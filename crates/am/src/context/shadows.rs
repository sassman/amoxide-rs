use crate::precedence::OriginScope;

use super::escape::escape_md_cell;
use super::LayerInputs;

/// Compute every (name, losing-scope, winning-scope) shadow triple from layer inputs.
///
/// Walks scopes in precedence order (highest first: project → profile
/// (slice-end first) → global), collecting each scope that defines a given
/// name. For each name defined in 2+ scopes, the head of the list is the
/// winner and every other scope is a loser. Returns one triple per
/// (name, loser).
fn collect_shadows(layers: &LayerInputs) -> Vec<(String, OriginScope, OriginScope)> {
    use crate::alias::AliasName;
    use std::collections::BTreeMap;

    let mut defs: BTreeMap<AliasName, Vec<OriginScope>> = BTreeMap::new();

    for (name, _) in layers.project_aliases.iter() {
        defs.entry(name.clone())
            .or_default()
            .push(OriginScope::Project);
    }
    for layer in layers.profile_layers.iter().rev() {
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

    let mut rows: Vec<(
        String,
        String,
        std::borrow::Cow<'static, str>,
        std::borrow::Cow<'static, str>,
    )> = Vec::new();
    for (name, loser, winner) in shadows {
        let expansion = lookup_expansion_at(&name, &loser, layers).unwrap_or_else(|| {
            panic!(
                "shadowed alias '{}' has no expansion in its declaring scope {:?}",
                name, loser
            )
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

    use std::borrow::Cow;
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<(Cow<'static, str>, Cow<'static, str>), Vec<String>> = BTreeMap::new();
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

#[cfg(test)]
mod brief_tests {
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
    fn shadow_brief_with_two_profiles_sharing_name_attributes_last_in_slice_as_winner() {
        // Engine: profile_layers[i+1] wins over [i]. With [git, rust] both
        // defining `x`, rust wins. The brief should report git as loser and
        // rust as winner — not the other way around.
        let global = AliasSet::default();
        let git = ProfileLayer {
            name: "git".into(),
            aliases: aset(&[("x", "git-x")]),
            subcommands: SubcommandSet::new(),
            vars: VarSet::default(),
        };
        let rust = ProfileLayer {
            name: "rust".into(),
            aliases: aset(&[("x", "rust-x")]),
            subcommands: SubcommandSet::new(),
            vars: VarSet::default(),
        };
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project = AliasSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let profile_slice = vec![git, rust];
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
        assert!(
            out.contains("x — also defined in profile:git, overridden by profile:rust"),
            "rust wins (later in slice), git is shadowed: {out}"
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
mod verbose_tests {
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
