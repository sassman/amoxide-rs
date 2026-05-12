//! `am context` snapshot renderer.
//!
//! Sibling presentation module to `display.rs`. Drives from
//! `Precedence::resolve()` outcome plus the original layer inputs.
//! Output is markdown intended for AI coding assistants.

use std::path::Path;

use crate::alias::AliasSet;
use crate::precedence::{EffectiveEntry, EntryKind, OriginScope, ProfileLayer};
use crate::subcommand::SubcommandSet;
use crate::vars::VarSet;

/// Escape characters that would break a markdown table cell.
///
/// Currently only `|` is escaped (most common in shell command bodies). Other
/// markdown specials (backticks, asterisks, etc.) are left verbatim — they
/// render fine inside table cells and the spec calls for verbatim output.
fn escape_md_cell(s: &str) -> String {
    s.replace('|', r"\|")
}

/// Describes the active precedence chain for the preamble.
/// Order: highest precedence first.
#[derive(Debug, Clone)]
pub struct PrecedenceChain {
    pub layers: Vec<ChainLayer>,
}

#[derive(Debug, Clone)]
pub struct ChainLayer {
    pub scope: OriginScope,
    /// For profile layers: 1-based priority (higher = higher precedence above lower).
    /// For global/project: None.
    pub priority: Option<usize>,
}

/// Render the preamble. Leads with usage rules; format notes second.
pub fn render_preamble(cwd: &Path, chain: &PrecedenceChain) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# amoxide aliases (active set, cwd: {})\n",
        cwd.display()
    ));
    out.push_str("#\n");
    out.push_str("# ## How to use this snapshot\n");
    out.push_str("#\n");
    out.push_str(
        "# When the user mentions a name from the `Aliases` table below in any context —\n",
    );
    out.push_str(
        "# running a command, suggesting one, asking what it does — treat the `expands to`\n",
    );
    out.push_str("# value as the canonical form.\n");
    out.push_str("#\n");
    out.push_str(
        "# 1. Recognise aliases by name match. If the user's input contains a token that\n",
    );
    out.push_str(
        "#    matches a `name` from the table — including multi-word names with a space,\n",
    );
    out.push_str("#    like `git pl` — it is an alias. Expand it before running.\n");
    out.push_str("#\n");
    out.push_str("# 2. Subcommand aliases are deceptive. A name like `git pl` looks like a real\n");
    out.push_str("#    git subcommand but is not. Running `git pl` verbatim in a subshell fails\n");
    out.push_str("#    with `git: 'pl' is not a git command`. Always run the value from\n");
    out.push_str("#    `expands to` (`git pull --rebase`), never the alias text.\n");
    out.push_str("#\n");
    out.push_str(
        "# 3. Recover from `command not found` failures. If a shell command fails because\n",
    );
    out.push_str("#    the name is unknown, check this table — the user's shell sees the alias\n");
    out.push_str("#    but your subshell does not.\n");
    out.push_str("#\n");
    out.push_str("# 4. In chat, the user's vocabulary is fine. When suggesting commands in\n");
    out.push_str(
        "#    conversation, the short form (`git cm \"msg\"`) matches the user's mental\n",
    );
    out.push_str("#    model. When *running* it in a subshell, use the canonical form.\n");
    out.push_str("#\n");
    out.push_str("# ## Format\n");
    out.push_str("#\n");
    out.push_str(&format!(
        "# Precedence (highest first): {}\n",
        render_chain(chain)
    ));
    out.push_str("#\n");
    out.push_str("# Templates: {{N}} is a positional placeholder (1-indexed; `tag v1.0` expands {{1}} → `v1.0`).\n");
    out.push_str("# Variables: {{name}} tokens are already substituted in the table below.\n");
    out.push_str("#            The full variable table is in the `## Variables` section.\n");
    out
}

fn render_chain(chain: &PrecedenceChain) -> String {
    chain
        .layers
        .iter()
        .map(|l| match (&l.scope, l.priority) {
            (OriginScope::Project, _) => "project".to_string(),
            (OriginScope::Profile(name), Some(p)) => format!("profile({name}, prio {p})"),
            (OriginScope::Profile(name), None) => format!("profile({name})"),
            (OriginScope::Global, _) => "global".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" > ")
}

/// References to all the layered inputs that fed `Precedence::resolve()`.
/// Carried as borrows so the renderer can re-lookup origins and (later)
/// reconstruct shadow chains without re-resolving.
#[derive(Debug, Clone, Copy)]
pub struct LayerInputs<'a> {
    pub global_aliases: &'a AliasSet,
    pub global_subcommands: &'a SubcommandSet,
    pub global_vars: &'a VarSet,
    pub profile_layers: &'a [ProfileLayer],
    pub project_aliases: &'a AliasSet,
    pub project_subcommands: &'a SubcommandSet,
    pub project_vars: &'a VarSet,
}

/// Find the highest-precedence layer that defines `name`, returning its scope.
///
/// Precedence (highest first): project > profile (in slice order) > global.
fn lookup_origin(name: &str, layers: &LayerInputs) -> Option<OriginScope> {
    use crate::alias::AliasName;
    let key = AliasName::from(name);
    if layers.project_aliases.contains_key(&key) {
        return Some(OriginScope::Project);
    }
    for layer in layers.profile_layers {
        if layer.aliases.contains_key(&key) {
            return Some(OriginScope::Profile(layer.name.clone()));
        }
    }
    if layers.global_aliases.contains_key(&key) {
        return Some(OriginScope::Global);
    }
    None
}

/// Look up the origin of a subcommand entry. Key is the colon-joined
/// `program:short[:short...]` form used by `SubcommandSet`.
fn lookup_subcommand_origin(
    program: &str,
    short_key: &str,
    layers: &LayerInputs,
) -> Option<OriginScope> {
    let key = format!("{program}:{short_key}");
    if layers.project_subcommands.as_ref().contains_key(&key) {
        return Some(OriginScope::Project);
    }
    for layer in layers.profile_layers {
        if layer.subcommands.as_ref().contains_key(&key) {
            return Some(OriginScope::Profile(layer.name.clone()));
        }
    }
    if layers.global_subcommands.as_ref().contains_key(&key) {
        return Some(OriginScope::Global);
    }
    None
}

/// Render the `## Aliases` markdown table.
///
/// Rows sorted by name. Subcommand wrappers are flattened to one row per
/// (program, short) pair. Tracking-only `SubcommandKey` entries are skipped.
pub fn render_aliases_table(effective: &[EffectiveEntry], layers: &LayerInputs) -> String {
    struct Row {
        name: String,
        expansion: String,
        from: String,
    }

    let mut rows: Vec<Row> = Vec::new();
    for e in effective {
        match &e.kind {
            EntryKind::Alias(alias) => {
                let expansion = alias.command().to_string();
                let from = lookup_origin(&e.name, layers)
                    .map(|s| s.as_from_label())
                    .unwrap_or_else(|| {
                        debug_assert!(
                            false,
                            "alias '{}' in effective set but not present in any layer input",
                            e.name
                        );
                        "[bug: unknown origin]".to_string()
                    });
                rows.push(Row {
                    name: e.name.clone(),
                    expansion,
                    from,
                });
            }
            EntryKind::SubcommandWrapper {
                program, entries, ..
            } => {
                for sub in entries {
                    let name = format!("{program} {}", sub.short_subcommands.join(" "));
                    let expansion = format!("{program} {}", sub.long_subcommands.join(" "));
                    let short_key = sub.short_subcommands.join(":");
                    let from = lookup_subcommand_origin(program, &short_key, layers)
                        .map(|s| s.as_from_label())
                        .unwrap_or_else(|| {
                            debug_assert!(
                                false,
                                "subcommand '{} {}' in effective set but not present in any layer input",
                                program,
                                sub.short_subcommands.join(" ")
                            );
                            "[bug: unknown origin]".to_string()
                        });
                    rows.push(Row {
                        name,
                        expansion,
                        from,
                    });
                }
            }
            EntryKind::SubcommandKey { .. } => {
                // Tracking-only, never shown
            }
        }
    }

    rows.sort_by(|a, b| a.name.cmp(&b.name));

    let mut out = String::from("## Aliases\n\n");
    out.push_str("| name | expands to | from |\n");
    out.push_str("|------|------------|------|\n");
    for r in rows {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            escape_md_cell(&r.name),
            escape_md_cell(&r.expansion),
            escape_md_cell(&r.from),
        ));
    }
    out
}

/// Compute every (name, losing-scope, winning-scope) shadow triple from layer inputs.
///
/// Walks scopes in precedence order (highest first: project → profile (in slice
/// order) → global), collecting each scope that defines a given name. For each
/// name defined in 2+ scopes, the head of the list is the winner and every other
/// scope is a loser. Returns one triple per (name, loser).
fn collect_shadows(layers: &LayerInputs) -> Vec<(String, OriginScope, OriginScope)> {
    use crate::alias::AliasName;
    use std::collections::BTreeMap;

    let mut defs: BTreeMap<AliasName, Vec<OriginScope>> = BTreeMap::new();

    for (name, _) in layers.project_aliases.iter() {
        defs.entry(name.clone())
            .or_default()
            .push(OriginScope::Project);
    }
    for layer in layers.profile_layers {
        for (name, _) in layer.aliases.iter() {
            defs.entry(name.clone())
                .or_default()
                .push(OriginScope::Profile(layer.name.clone()));
        }
    }
    for (name, _) in layers.global_aliases.iter() {
        defs.entry(name.clone())
            .or_default()
            .push(OriginScope::Global);
    }

    let mut out = Vec::new();
    for (name, scopes) in defs {
        if scopes.len() < 2 {
            continue;
        }
        let winner = scopes[0].clone();
        for loser in scopes.iter().skip(1) {
            out.push((name.as_ref().to_string(), loser.clone(), winner.clone()));
        }
    }
    out
}

/// Render the brief `## Shadowed` section. Empty string if no shadows.
///
/// Groups names by (losing-scope, winning-scope) pair, names alphabetised
/// within each group. Includes a load-bearing pointer to `--verbose`.
pub fn render_shadow_brief(layers: &LayerInputs) -> String {
    let shadows = collect_shadows(layers);
    if shadows.is_empty() {
        return String::new();
    }

    use std::collections::BTreeMap;
    let mut groups: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for (name, loser, winner) in shadows {
        groups
            .entry((loser.as_from_label(), winner.as_from_label()))
            .or_default()
            .push(name);
    }

    let mut out = String::from("## Shadowed\n");
    for ((loser, winner), mut names) in groups {
        names.sort();
        out.push_str(&format!(
            "- {} — also defined in {loser}, overridden by {winner}\n",
            names.join(", ")
        ));
    }
    out.push('\n');
    out.push_str("(run `am context --verbose` for full definitions, origins, and shadow chains)\n");
    out
}

/// Render the `## Variables` section, or empty string if no scope has any var.
///
/// Per-scope subsections (`### project`, `### profile:<name>`, `### global`).
/// Empty scopes show `(none)` when at least one scope has a var, so the model
/// can confirm absence without re-running `am context`.
pub fn render_variables(layers: &LayerInputs) -> String {
    // Walk scopes in display order: project → profile (in priority/slice order) → global.
    let scopes: Vec<(String, &VarSet)> =
        std::iter::once(("project".to_string(), layers.project_vars))
            .chain(
                layers
                    .profile_layers
                    .iter()
                    .map(|l| (format!("profile:{}", l.name), &l.vars)),
            )
            .chain(std::iter::once(("global".to_string(), layers.global_vars)))
            .collect();

    if scopes.iter().all(|(_, v)| v.is_empty()) {
        return String::new();
    }

    let mut out = String::from("## Variables\n\n");
    for (label, vs) in &scopes {
        out.push_str(&format!("### {label}\n"));
        if vs.is_empty() {
            out.push_str("(none)\n\n");
        } else {
            out.push_str("| name | value |\n");
            out.push_str("|------|-------|\n");
            for (name, value) in vs.iter() {
                out.push_str(&format!(
                    "| {} | {} |\n",
                    escape_md_cell(name.as_str()),
                    escape_md_cell(value),
                ));
            }
            out.push('\n');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn chain(layers: Vec<ChainLayer>) -> PrecedenceChain {
        PrecedenceChain { layers }
    }

    #[test]
    fn preamble_starts_with_cwd_header() {
        let cwd = PathBuf::from("/tmp/project");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let out = render_preamble(&cwd, &c);
        assert!(
            out.starts_with("# amoxide aliases (active set, cwd: /tmp/project)\n"),
            "got: {out}"
        );
    }

    #[test]
    fn preamble_contains_all_four_usage_rules() {
        let cwd = PathBuf::from("/x");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let out = render_preamble(&cwd, &c);
        assert!(out.contains("1. Recognise aliases by name match"), "rule 1");
        assert!(
            out.contains("2. Subcommand aliases are deceptive"),
            "rule 2"
        );
        assert!(
            out.contains("3. Recover from `command not found`"),
            "rule 3"
        );
        assert!(
            out.contains("4. In chat, the user's vocabulary is fine"),
            "rule 4"
        );
    }

    #[test]
    fn preamble_renders_precedence_chain_in_order() {
        let cwd = PathBuf::from("/x");
        let c = chain(vec![
            ChainLayer {
                scope: OriginScope::Project,
                priority: None,
            },
            ChainLayer {
                scope: OriginScope::Profile("git".into()),
                priority: Some(2),
            },
            ChainLayer {
                scope: OriginScope::Profile("rust".into()),
                priority: Some(1),
            },
            ChainLayer {
                scope: OriginScope::Global,
                priority: None,
            },
        ]);
        let out = render_preamble(&cwd, &c);
        assert!(
            out.contains("project > profile(git, prio 2) > profile(rust, prio 1) > global"),
            "got: {out}"
        );
    }
}

#[cfg(test)]
mod aliases_tests {
    use super::*;
    use crate::alias::{AliasName, AliasSet, TomlAlias};
    use crate::precedence::{EffectiveEntry, EntryKind};
    use crate::subcommand::SubcommandSet;
    use crate::vars::VarSet;

    fn aset(pairs: &[(&str, &str)]) -> AliasSet {
        let mut s = AliasSet::default();
        for (n, c) in pairs {
            s.insert(AliasName::from(*n), TomlAlias::Command((*c).into()));
        }
        s
    }

    fn entry(name: &str, cmd: &str) -> EffectiveEntry {
        EffectiveEntry {
            name: name.into(),
            kind: EntryKind::Alias(TomlAlias::Command(cmd.into())),
            hash: "x".into(),
        }
    }

    #[test]
    fn aliases_table_sorted_by_name_with_origin_from_layers() {
        let global = aset(&[("ll", "ls -lha")]);
        let project = aset(&[("f", "cargo fmt")]);
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &[],
            project_aliases: &project,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let effective = vec![entry("ll", "ls -lha"), entry("f", "cargo fmt")];
        let out = render_aliases_table(&effective, &layers);
        let f_idx = out.find("| f ").unwrap();
        let ll_idx = out.find("| ll ").unwrap();
        assert!(f_idx < ll_idx, "rows must be alphabetical: {out}");
        assert!(out.contains("| project"), "f's from must be project: {out}");
        assert!(out.contains("| global"), "ll's from must be global: {out}");
    }

    #[test]
    fn aliases_table_skips_subcommand_key_entries() {
        let global = AliasSet::default();
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project = AliasSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &[],
            project_aliases: &project,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let effective = vec![EffectiveEntry {
            name: "git:pl".into(),
            kind: EntryKind::SubcommandKey {
                longs: vec!["pull".into(), "--rebase".into()],
            },
            hash: "x".into(),
        }];
        let out = render_aliases_table(&effective, &layers);
        assert!(
            !out.contains("git:pl"),
            "tracking entries must not appear: {out}"
        );
        assert!(
            !out.contains("| pl "),
            "tracking entries must not appear: {out}"
        );
    }

    #[test]
    fn aliases_table_flattens_subcommand_wrappers() {
        use crate::subcommand::SubcommandEntry;
        let global = AliasSet::default();
        let mut global_subs = SubcommandSet::new();
        global_subs
            .as_mut()
            .insert("git:pl".into(), Default::default());
        global_subs
            .as_mut()
            .insert("git:psh".into(), Default::default());
        let global_vars = VarSet::default();
        let project = AliasSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &[],
            project_aliases: &project,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let effective = vec![EffectiveEntry {
            name: "git".into(),
            kind: EntryKind::SubcommandWrapper {
                program: "git".into(),
                entries: vec![
                    SubcommandEntry {
                        program: "git".into(),
                        short_subcommands: vec!["pl".into()],
                        long_subcommands: vec!["pull --rebase".into()],
                    },
                    SubcommandEntry {
                        program: "git".into(),
                        short_subcommands: vec!["psh".into()],
                        long_subcommands: vec!["push".into()],
                    },
                ],
                base_cmd: None,
            },
            hash: "x".into(),
        }];
        let out = render_aliases_table(&effective, &layers);
        assert!(out.contains("| git pl "), "must flatten subcommand: {out}");
        assert!(
            out.contains("git pull --rebase"),
            "expansion must be full surface form: {out}"
        );
        assert!(out.contains("| git psh "), "must flatten subcommand: {out}");
        assert!(
            out.contains("| git push "),
            "expansion must be full surface form: {out}"
        );
        assert!(
            !out.contains("| git |"),
            "the wrapper row itself must not appear"
        );
    }

    #[test]
    fn aliases_table_escapes_pipe_characters_in_cells() {
        let global = aset(&[("filter", "rg foo | rg bar")]);
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project = AliasSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &[],
            project_aliases: &project,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let effective = vec![entry("filter", "rg foo | rg bar")];
        let out = render_aliases_table(&effective, &layers);
        assert!(
            out.contains(r"rg foo \| rg bar"),
            "pipe in expansion must be escaped as \\|: {out}"
        );
        // Column count check: every row must still have exactly 3 unescaped pipes (4 columns delimited by 3 separators when counting opens/closes).
        // Simpler: confirm we don't have 4 unescaped pipes on the row.
        let data_line = out.lines().find(|l| l.contains("filter")).unwrap();
        let unescaped_pipes = data_line
            .chars()
            .zip(std::iter::once(' ').chain(data_line.chars()))
            .filter(|(c, prev)| *c == '|' && *prev != '\\')
            .count();
        assert_eq!(unescaped_pipes, 4, "data row must have exactly 4 unescaped pipes (3 column boundaries + leading/trailing): {data_line}");
    }
}

#[cfg(test)]
mod variables_tests {
    use super::*;
    use crate::alias::AliasSet;
    use crate::precedence::ProfileLayer;
    use crate::subcommand::SubcommandSet;
    use crate::vars::{VarName, VarSet};

    fn vset(pairs: &[(&str, &str)]) -> VarSet {
        let mut s = VarSet::default();
        for (n, v) in pairs {
            s.insert(VarName::parse(n).unwrap(), (*v).to_string());
        }
        s
    }

    fn layers_no_aliases<'a>(
        global_vars: &'a VarSet,
        profiles: &'a [ProfileLayer],
        project_vars: &'a VarSet,
        empty_aliases: &'a AliasSet,
        empty_subs: &'a SubcommandSet,
    ) -> LayerInputs<'a> {
        LayerInputs {
            global_aliases: empty_aliases,
            global_subcommands: empty_subs,
            global_vars,
            profile_layers: profiles,
            project_aliases: empty_aliases,
            project_subcommands: empty_subs,
            project_vars,
        }
    }

    #[test]
    fn variables_section_omitted_when_no_vars_anywhere() {
        let global = VarSet::default();
        let project = VarSet::default();
        let empty_aliases = AliasSet::default();
        let empty_subs = SubcommandSet::new();
        let layers = layers_no_aliases(&global, &[], &project, &empty_aliases, &empty_subs);
        let out = render_variables(&layers);
        assert!(out.is_empty(), "expected empty, got: {out}");
    }

    #[test]
    fn variables_section_rendered_when_global_has_var() {
        let global = vset(&[("opt-flags", "-C opt-level=3")]);
        let project = VarSet::default();
        let empty_aliases = AliasSet::default();
        let empty_subs = SubcommandSet::new();
        let layers = layers_no_aliases(&global, &[], &project, &empty_aliases, &empty_subs);
        let out = render_variables(&layers);
        assert!(out.contains("## Variables"), "section header: {out}");
        assert!(out.contains("### global"), "global subsection: {out}");
        assert!(out.contains("opt-flags"), "var name: {out}");
        assert!(out.contains("-C opt-level=3"), "var value: {out}");
    }

    #[test]
    fn variables_section_shows_none_for_empty_scope_when_any_other_has_one() {
        let global = vset(&[("opt-flags", "-C opt-level=3")]);
        let empty_subs = SubcommandSet::new();
        let empty_aliases = AliasSet::default();
        let profile_vars = VarSet::default();
        let profiles = vec![ProfileLayer {
            name: "git".into(),
            aliases: AliasSet::default(),
            subcommands: SubcommandSet::new(),
            vars: profile_vars.clone(),
        }];
        let project = VarSet::default();
        let layers = layers_no_aliases(&global, &profiles, &project, &empty_aliases, &empty_subs);
        let out = render_variables(&layers);
        assert!(out.contains("### profile:git"), "profile subsection: {out}");
        assert!(out.contains("(none)"), "(none) for empty scope: {out}");
    }

    #[test]
    fn variables_section_escapes_pipe_characters_in_values() {
        let global = vset(&[("filter_cmd", "rg foo | rg bar")]);
        let empty_subs = SubcommandSet::new();
        let empty_aliases = AliasSet::default();
        let project = VarSet::default();
        let layers = layers_no_aliases(&global, &[], &project, &empty_aliases, &empty_subs);
        let out = render_variables(&layers);
        assert!(
            out.contains(r"rg foo \| rg bar"),
            "pipe in value must be escaped as \\|: {out}"
        );
        let data_line = out.lines().find(|l| l.contains("filter_cmd")).unwrap();
        let unescaped_pipes = data_line
            .chars()
            .zip(std::iter::once(' ').chain(data_line.chars()))
            .filter(|(c, prev)| *c == '|' && *prev != '\\')
            .count();
        assert_eq!(
            unescaped_pipes, 3,
            "data row must have exactly 3 unescaped pipes (2 column boundaries + outer): {data_line}"
        );
    }
}

#[cfg(test)]
mod shadow_brief_tests {
    use super::*;
    use crate::alias::{AliasName, AliasSet, TomlAlias};
    use crate::precedence::ProfileLayer;
    use crate::subcommand::SubcommandSet;
    use crate::vars::VarSet;

    fn aset(pairs: &[(&str, &str)]) -> AliasSet {
        let mut s = AliasSet::default();
        for (n, c) in pairs {
            s.insert(AliasName::from(*n), TomlAlias::Command((*c).into()));
        }
        s
    }

    #[test]
    fn shadow_brief_empty_when_no_shadows() {
        let global = AliasSet::default();
        let project = aset(&[("f", "cargo fmt")]);
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &[],
            project_aliases: &project,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let out = render_shadow_brief(&layers);
        assert!(out.is_empty(), "no shadows means no section: {out}");
    }

    #[test]
    fn shadow_brief_groups_by_loser_winner_pair() {
        // `f`, `t` both in profile:rust and project. project wins.
        let global = AliasSet::default();
        let project = aset(&[("f", "cargo fmt"), ("t", "cargo test")]);
        let rust = ProfileLayer {
            name: "rust".into(),
            aliases: aset(&[("f", "cargo fmt --check"), ("t", "cargo nextest")]),
            subcommands: SubcommandSet::new(),
            vars: VarSet::default(),
        };
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let profile_slice = vec![rust];
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &profile_slice,
            project_aliases: &project,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let out = render_shadow_brief(&layers);
        assert!(out.contains("## Shadowed"), "section header: {out}");
        assert!(
            out.contains("f, t — also defined in profile:rust, overridden by project"),
            "grouped names with loser/winner: {out}"
        );
        assert!(
            out.contains("(run `am context --verbose`"),
            "must include verbose pointer: {out}"
        );
    }

    #[test]
    fn shadow_brief_multiple_losing_scopes_emit_separate_bullets() {
        // `x` exists in BOTH global and profile:git. profile:git wins.
        // Project doesn't define x at all.
        let global = aset(&[("x", "global-x")]);
        let git = ProfileLayer {
            name: "git".into(),
            aliases: aset(&[("x", "git-x")]),
            subcommands: SubcommandSet::new(),
            vars: VarSet::default(),
        };
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project_aliases = AliasSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let profile_slice = vec![git];
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &profile_slice,
            project_aliases: &project_aliases,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let out = render_shadow_brief(&layers);
        // x → loser global, winner profile:git
        assert!(
            out.contains("x — also defined in global, overridden by profile:git"),
            "expected grouping: {out}"
        );
    }
}
