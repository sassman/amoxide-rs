//! `am context` snapshot renderer.
//!
//! Sibling presentation module to `display.rs`. Drives from
//! `Precedence::resolve()` outcome plus the original layer inputs.
//! Output is markdown intended for AI coding assistants.

use std::path::Path;

use indoc::formatdoc;

use crate::alias::AliasSet;
use crate::precedence::{
    Diagnostic, EffectiveEntry, EntryKind, OriginScope, ProfileLayer, ResolveOutcome,
};
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
pub fn render_preamble(
    cwd: &Path,
    chain: &PrecedenceChain,
    project_trust_notice: Option<&ProjectTrustNotice>,
) -> String {
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
    out.push_str("# 5. Aliases reveal user preference — match them by intent, not just by name.\n");
    out.push_str("#    - If the user describes an action in plain language (\"test the code\",\n");
    out.push_str("#      \"format\", \"install\") rather than typing an alias name, scan the\n");
    out.push_str("#      `expands to` column for a command that covers the intent and run\n");
    out.push_str("#      *that* exact form. The flags are deliberate (`cargo test\n");
    out.push_str("#      --all-features --verbose`, not bare `cargo test`); reaching for a\n");
    out.push_str("#      vanilla command throws away the user's choices.\n");
    if project_trust_notice.is_some() {
        out.push_str("#    - A project `.aliases` file in scope but untrusted (see\n");
        out.push_str("#      `## Project aliases (not loaded)` below) is the same signal — the\n");
        out.push_str("#      user has even more refined preferences for this directory, you\n");
        out.push_str("#      just can't see them yet. Treat that as added weight on the\n");
        out.push_str("#      `am trust` ask.\n");
    }
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

/// Look up the command body of an alias by name in a specific scope.
fn lookup_expansion_at(name: &str, scope: &OriginScope, layers: &LayerInputs) -> Option<String> {
    use crate::alias::AliasName;
    let key = AliasName::from(name);
    let set = match scope {
        OriginScope::Project => layers.project_aliases,
        OriginScope::Global => layers.global_aliases,
        OriginScope::Profile(name) => {
            let layer = layers.profile_layers.iter().find(|l| &l.name == name)?;
            &layer.aliases
        }
    };
    set.get(&key).map(|a| a.command().to_string())
}

/// Render the verbose `## Shadowed` table. Empty string if no shadows.
///
/// Four columns: name, expansion (at the loser's scope), from (loser), shadowed by (winner).
/// One row per `(name, defining-scope)` pair from the shadowed set, ordered first
/// by name then by ascending precedence (lowest-priority definition first).
pub fn render_shadow_verbose(layers: &LayerInputs) -> String {
    let shadows = collect_shadows(layers);
    if shadows.is_empty() {
        return String::new();
    }

    let mut rows: Vec<(String, String, String, String)> = Vec::new();
    for (name, loser, winner) in shadows {
        let expansion = lookup_expansion_at(&name, &loser, layers).unwrap_or_else(|| {
            debug_assert!(
                false,
                "shadowed alias '{}' has no expansion in its declaring scope {:?}",
                name, loser
            );
            "[bug: missing expansion]".to_string()
        });
        rows.push((
            name,
            expansion,
            loser.as_from_label(),
            winner.as_from_label(),
        ));
    }
    rows.sort_by(|a, b| a.0.cmp(&b.0));

    let mut out = String::from("## Shadowed\n\n");
    out.push_str("| name | expansion | from | shadowed by |\n");
    out.push_str("|------|-----------|------|-------------|\n");
    for (name, exp, from, by) in rows {
        out.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            escape_md_cell(&name),
            escape_md_cell(&exp),
            escape_md_cell(&from),
            escape_md_cell(&by),
        ));
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

/// Render the verbose-only `## Invalid` section. Empty if no diagnostics.
///
/// Drives off `ResolveOutcome.diagnostics` (the rendered, ready-to-print form)
/// rather than re-formatting `InvalidEntry` independently. One bullet per
/// diagnostic.
pub fn render_invalid(diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return String::new();
    }
    let mut out = String::from("## Invalid\n");
    for d in diagnostics {
        out.push_str(&format!("- {}\n", d.message));
    }
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

/// A discovered `.aliases` file that is not currently loaded due to its
/// trust state. Carried into the snapshot so the agent can prompt the user
/// to review the file instead of silently working from an incomplete picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectTrustNotice {
    pub path: std::path::PathBuf,
    pub reason: ProjectTrustReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectTrustReason {
    /// File exists but has never been reviewed by the user.
    Unknown,
    /// User previously declined to trust this file.
    Untrusted,
    /// File contents changed since the user last trusted it.
    Tampered,
}

impl ProjectTrustReason {
    fn label(self) -> &'static str {
        match self {
            Self::Unknown => "not yet reviewed",
            Self::Untrusted => "previously untrusted",
            Self::Tampered => "modified since last trusted",
        }
    }
}

/// Render the `## Project aliases` section. Empty string when there is no
/// such file in scope or the file is trusted (its aliases are already in
/// the main table).
///
/// The body tells the *agent* to ask the *user* once at session start to
/// run `am trust`. `am trust` itself is the interactive review tool: it
/// shows the file contents and asks for confirmation. The agent's only job
/// here is to surface the choice to the user — once — before acting on the
/// alias table, which is partial while the project file is untrusted.
///
/// `alias_names` is every name currently in the effective alias table,
/// listed verbatim in the notice so the agent sees concretely which entries
/// the untrusted project file could override.
pub fn render_project_trust_notice(
    notice: Option<&ProjectTrustNotice>,
    alias_names: &[&str],
) -> String {
    let Some(notice) = notice else {
        return String::new();
    };
    let name_list = if alias_names.is_empty() {
        String::from("(the table is currently empty)")
    } else {
        alias_names
            .iter()
            .map(|n| format!("`{n}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    formatdoc! {r#"
        ## Project aliases (not loaded)

        > [!WARNING]
        > Your view of aliases here is incomplete. The table below is missing the project file's entries, any of which can shadow what you see.

        A project `.aliases` file at `{path}` is in scope but not loaded ({reason}). Its entries shadow every alias of the same name at priority 0 — above all profiles and globals — so any of these table names, plus any the project file may add, could resolve to a different command here:

        {name_list}

        **At session start, ask the user once** — unless they have already declined trust in this session, in which case skip this whole section:

        > "A project `.aliases` file here isn't trusted yet. Run `am trust` in your shell — it'll show you the contents and ask for your confirmation."

        Then wait for their answer before acting on any alias.

        **Hand-off rules**

        - Don't read `.aliases` yourself — `am trust` is the review surface; reading it shortcuts the interactive flow.
        - Don't run `am trust` in a subshell — it's interactive and hangs on the prompt. Run it in the user's terminal.
    "#,
        path = notice.path.display(),
        reason = notice.reason.label(),
        name_list = name_list,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub verbose: bool,
}

/// Top-level orchestrator for the `am context` snapshot.
///
/// Wires every section into one markdown blob:
///   preamble → project-trust-notice → aliases → variables → shadowed → invalid
///
/// All sections except preamble and aliases are conditional. Invalid is
/// verbose-only. The project-trust-notice appears when an `.aliases` file
/// exists in scope but is not loaded, and is emitted *before* the alias
/// table so the agent reads the gating rule before treating the table as
/// authoritative.
pub fn render(
    cwd: &Path,
    chain: &PrecedenceChain,
    outcome: &ResolveOutcome,
    layers: &LayerInputs,
    project_trust_notice: Option<&ProjectTrustNotice>,
    opts: RenderOptions,
) -> String {
    let mut out = String::new();
    out.push_str(&render_preamble(cwd, chain, project_trust_notice));
    out.push('\n');

    // Effective set for `am context` is added ∪ unchanged
    let effective: Vec<_> = outcome
        .diff
        .added
        .iter()
        .chain(outcome.diff.unchanged.iter())
        .cloned()
        .collect();

    let mut alias_names: Vec<String> = Vec::new();
    for e in &effective {
        match &e.kind {
            EntryKind::Alias(_) => alias_names.push(e.name.clone()),
            EntryKind::SubcommandWrapper {
                program, entries, ..
            } => {
                for sub in entries {
                    alias_names.push(format!("{program} {}", sub.short_subcommands.join(" ")));
                }
            }
            EntryKind::SubcommandKey { .. } => {
                // Tracking-only, not user-facing.
            }
        }
    }
    alias_names.sort();
    let name_refs: Vec<&str> = alias_names.iter().map(String::as_str).collect();
    let trust_notice = render_project_trust_notice(project_trust_notice, &name_refs);
    if !trust_notice.is_empty() {
        out.push_str(&trust_notice);
        out.push('\n');
    }

    out.push_str(&render_aliases_table(&effective, layers));
    out.push('\n');

    let vars = render_variables(layers);
    if !vars.is_empty() {
        out.push_str(&vars);
        out.push('\n');
    }

    let shadow = if opts.verbose {
        render_shadow_verbose(layers)
    } else {
        render_shadow_brief(layers)
    };
    if !shadow.is_empty() {
        out.push_str(&shadow);
        out.push('\n');
    }

    if opts.verbose {
        let invalid = render_invalid(&outcome.diagnostics);
        if !invalid.is_empty() {
            out.push_str(&invalid);
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
        let out = render_preamble(&cwd, &c, None);
        assert!(
            out.starts_with("# amoxide aliases (active set, cwd: /tmp/project)\n"),
            "got: {out}"
        );
    }

    #[test]
    fn preamble_contains_all_five_usage_rules() {
        let cwd = PathBuf::from("/x");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let out = render_preamble(&cwd, &c, None);
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
        assert!(out.contains("5. Aliases reveal user preference"), "rule 5");
        assert!(
            out.contains("describes an action in plain language"),
            "rule 5 first bullet (intent-match)"
        );
    }

    #[test]
    fn preamble_omits_rule_5_trust_bullet_when_no_notice() {
        let cwd = PathBuf::from("/x");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let out = render_preamble(&cwd, &c, None);
        assert!(
            !out.contains("`am trust` ask"),
            "rule 5 trust bullet should be absent when no untrusted project notice: {out}"
        );
    }

    #[test]
    fn preamble_emits_rule_5_trust_bullet_when_notice_present() {
        let cwd = PathBuf::from("/x");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let notice = ProjectTrustNotice {
            path: "/x/.aliases".into(),
            reason: ProjectTrustReason::Untrusted,
        };
        let out = render_preamble(&cwd, &c, Some(&notice));
        assert!(
            out.contains("`am trust` ask"),
            "rule 5 trust bullet should appear when notice present: {out}"
        );
        assert!(
            out.contains("`## Project aliases (not loaded)` below"),
            "rule 5 trust bullet points forward to the trust-notice section: {out}"
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
        let out = render_preamble(&cwd, &c, None);
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

#[cfg(test)]
mod shadow_verbose_tests {
    use super::*;
    use crate::alias::{AliasName, AliasSet, TomlAlias};
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
    fn shadow_verbose_empty_when_no_shadows() {
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
        let out = render_shadow_verbose(&layers);
        assert!(out.is_empty(), "no shadows means no section: {out}");
    }

    #[test]
    fn shadow_verbose_renders_full_table() {
        let global = aset(&[("f", "cargo fmt --check")]);
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
        let out = render_shadow_verbose(&layers);
        assert!(out.contains("## Shadowed"), "header: {out}");
        assert!(
            out.contains("| name | expansion | from | shadowed by |"),
            "column header: {out}"
        );
        assert!(
            out.contains("| f | cargo fmt --check | global | project |"),
            "shadow row: {out}"
        );
        // No verbose-pointer in verbose mode
        assert!(
            !out.contains("`am context --verbose`"),
            "no pointer in verbose mode: {out}"
        );
    }

    #[test]
    fn shadow_verbose_escapes_pipes_in_cells() {
        let global = aset(&[("f", "echo a | rg b")]);
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
        let out = render_shadow_verbose(&layers);
        assert!(
            out.contains(r"echo a \| rg b"),
            "pipe in expansion must be escaped: {out}"
        );
    }
}

#[cfg(test)]
mod project_trust_notice_tests {
    use super::*;

    #[test]
    fn no_notice_when_none() {
        let out = render_project_trust_notice(None, &[]);
        assert!(out.is_empty(), "expected empty: {out}");
    }

    #[test]
    fn notice_names_path_reason_session_start_ask_and_alias_list() {
        let notice = ProjectTrustNotice {
            path: "/x/.aliases".into(),
            reason: ProjectTrustReason::Untrusted,
        };
        let out = render_project_trust_notice(Some(&notice), &["i", "t", "git cm"]);
        assert!(
            out.contains("## Project aliases (not loaded)"),
            "header: {out}"
        );
        assert!(
            out.contains("> [!WARNING]"),
            "leads with a markdown-native warning admonition: {out}"
        );
        assert!(
            out.contains("Your view of aliases here is incomplete"),
            "warning body names the failure mode: {out}"
        );
        assert!(out.contains("/x/.aliases"), "path: {out}");
        assert!(out.contains("previously untrusted"), "reason label: {out}");
        assert!(
            out.contains("At session start, ask the user once"),
            "asks once at session start (not action-gated): {out}"
        );
        assert!(
            out.contains("already declined trust in this session"),
            "respects a prior decline inline with the ask: {out}"
        );
        assert!(
            out.contains("Run `am trust` in your shell"),
            "natural prompt hands off to user terminal: {out}"
        );
        assert!(
            out.contains("`i`") && out.contains("`t`") && out.contains("`git cm`"),
            "every passed alias name appears verbatim: {out}"
        );
        assert!(
            out.contains("**Hand-off rules**"),
            "groups the don'ts under a labelled bullet block: {out}"
        );
        assert!(
            out.contains("Don't read `.aliases` yourself"),
            "forbids reading the file ourselves: {out}"
        );
        assert!(
            out.contains("Don't run `am trust` in a subshell"),
            "forbids subshell execution: {out}"
        );
    }

    #[test]
    fn notice_handles_empty_alias_list_gracefully() {
        let notice = ProjectTrustNotice {
            path: "/x/.aliases".into(),
            reason: ProjectTrustReason::Unknown,
        };
        let out = render_project_trust_notice(Some(&notice), &[]);
        assert!(
            out.contains("the table is currently empty"),
            "fallback phrasing when no aliases in scope: {out}"
        );
    }

    #[test]
    fn notice_reason_labels_per_trust_state() {
        for (reason, label) in [
            (ProjectTrustReason::Unknown, "not yet reviewed"),
            (ProjectTrustReason::Untrusted, "previously untrusted"),
            (ProjectTrustReason::Tampered, "modified since last trusted"),
        ] {
            let notice = ProjectTrustNotice {
                path: "/p/.aliases".into(),
                reason,
            };
            let out = render_project_trust_notice(Some(&notice), &[]);
            assert!(
                out.contains(label),
                "label '{label}' missing for {reason:?}: {out}"
            );
        }
    }
}

#[cfg(test)]
mod invalid_tests {
    use super::*;

    #[test]
    fn invalid_section_empty_when_no_diagnostics() {
        let out = render_invalid(&[]);
        assert!(out.is_empty(), "no diagnostics means no section: {out}");
    }

    #[test]
    fn invalid_section_renders_diagnostic_messages() {
        let diags = vec![Diagnostic {
            message: "alias `cc` in profile:compile_help references undefined vars: opt-flags"
                .into(),
        }];
        let out = render_invalid(&diags);
        assert!(out.contains("## Invalid"), "section header: {out}");
        assert!(
            out.contains("alias `cc` in profile:compile_help"),
            "message text: {out}"
        );
        assert!(
            out.contains("undefined vars: opt-flags"),
            "message text: {out}"
        );
    }

    #[test]
    fn invalid_section_emits_one_bullet_per_diagnostic() {
        let diags = vec![
            Diagnostic {
                message: "first message".into(),
            },
            Diagnostic {
                message: "second message".into(),
            },
        ];
        let out = render_invalid(&diags);
        // Count bullet lines (lines starting with "- ")
        let bullets = out.lines().filter(|l| l.starts_with("- ")).count();
        assert_eq!(bullets, 2, "expected 2 bullets, got: {out}");
        assert!(out.contains("- first message"));
        assert!(out.contains("- second message"));
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;
    use crate::alias::{AliasName, AliasSet, TomlAlias};
    use crate::precedence::{
        Diagnostic, EffectiveEntry, EntryKind, PrecedenceDiff, ResolveOutcome,
    };
    use crate::subcommand::SubcommandSet;
    use crate::vars::VarSet;
    use std::path::PathBuf;

    fn aset(pairs: &[(&str, &str)]) -> AliasSet {
        let mut s = AliasSet::default();
        for (n, c) in pairs {
            s.insert(AliasName::from(*n), TomlAlias::Command((*c).into()));
        }
        s
    }

    #[test]
    fn render_assembles_all_sections_in_order_for_brief() {
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
        let chain = PrecedenceChain {
            layers: vec![
                ChainLayer {
                    scope: OriginScope::Project,
                    priority: None,
                },
                ChainLayer {
                    scope: OriginScope::Global,
                    priority: None,
                },
            ],
        };

        let effective = vec![
            EffectiveEntry {
                name: "f".into(),
                kind: EntryKind::Alias(TomlAlias::Command("cargo fmt".into())),
                hash: "x".into(),
            },
            EffectiveEntry {
                name: "ll".into(),
                kind: EntryKind::Alias(TomlAlias::Command("ls -lha".into())),
                hash: "y".into(),
            },
        ];
        let outcome = ResolveOutcome {
            diff: PrecedenceDiff {
                added: effective,
                ..Default::default()
            },
            diagnostics: vec![],
        };

        let out = render(
            &PathBuf::from("/tmp/x"),
            &chain,
            &outcome,
            &layers,
            None,
            RenderOptions { verbose: false },
        );

        // Section presence
        assert!(out.contains("# amoxide aliases"), "preamble present: {out}");
        assert!(out.contains("## Aliases"), "aliases header present: {out}");
        // No variables section (none defined); note the preamble comment references
        // "## Variables" so we check for the standalone section header line.
        assert!(
            !out.contains("\n## Variables\n"),
            "no vars means no section: {out}"
        );
        // No shadowed section (no shadows)
        assert!(!out.contains("## Shadowed"), "no shadows: {out}");
        // No invalid section (brief mode)
        assert!(!out.contains("## Invalid"), "brief omits invalid: {out}");

        // Section order
        let preamble_idx = out.find("# amoxide aliases").unwrap();
        let aliases_idx = out.find("## Aliases").unwrap();
        assert!(preamble_idx < aliases_idx, "preamble before aliases");
    }

    #[test]
    fn render_includes_invalid_only_in_verbose_mode() {
        let global = AliasSet::default();
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project_aliases = AliasSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &[],
            project_aliases: &project_aliases,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let chain = PrecedenceChain {
            layers: vec![ChainLayer {
                scope: OriginScope::Global,
                priority: None,
            }],
        };
        let outcome = ResolveOutcome {
            diff: PrecedenceDiff::default(),
            diagnostics: vec![Diagnostic {
                message: "alias `cc` missing vars: x".into(),
            }],
        };

        let brief = render(
            &PathBuf::from("/x"),
            &chain,
            &outcome,
            &layers,
            None,
            RenderOptions { verbose: false },
        );
        let verbose = render(
            &PathBuf::from("/x"),
            &chain,
            &outcome,
            &layers,
            None,
            RenderOptions { verbose: true },
        );
        assert!(
            !brief.contains("## Invalid"),
            "brief hides invalid: {brief}"
        );
        assert!(
            verbose.contains("## Invalid"),
            "verbose shows invalid: {verbose}"
        );
        assert!(verbose.contains("alias `cc` missing vars: x"));
    }
}
