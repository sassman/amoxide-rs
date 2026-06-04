use std::collections::BTreeMap;

use super::escape::escape_md_cell;
use super::LayerInputs;

/// Render the `## Variables` section as a single table with rows for the
/// variables that are actually defined; empty scopes contribute nothing.
///
/// Each name appears once with the scope that wins precedence
/// (project > profile (last in slice first) > global), mirroring the
/// `## Aliases` table. Returns an empty string when no scope defines any
/// variable — the whole section is then dropped from the snapshot.
pub fn render_variables(layers: &LayerInputs) -> String {
    // First-write-wins map keyed by name: walk highest-precedence scope first
    // and skip names we've already seen, so the winning scope is what lands.
    let mut rows: BTreeMap<String, (String, String)> = BTreeMap::new();

    let mut visit = |label: &str, vars: &crate::vars::VarSet| {
        for (name, value) in vars.iter() {
            rows.entry(name.as_str().to_string())
                .or_insert_with(|| (value.clone(), label.to_string()));
        }
    };

    visit("project", layers.project_vars);
    // Profiles: last in slice wins (matches the engine's merge order), so
    // walk them in reverse for first-write-wins.
    for layer in layers.profile_layers.iter().rev() {
        let label = format!("profile:{}", layer.name);
        visit(&label, &layer.vars);
    }
    visit("global", layers.global_vars);

    if rows.is_empty() {
        return String::new();
    }

    let mut out = String::from("## Variables\n\n");
    out.push_str("| name | value | from |\n");
    out.push_str("|------|-------|------|\n");
    for (name, (value, from)) in &rows {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            escape_md_cell(name),
            escape_md_cell(value),
            escape_md_cell(from),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
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
    fn variables_section_rendered_as_single_table_with_from_column() {
        let global = vset(&[("opt-flags", "-C opt-level=3")]);
        let project = VarSet::default();
        let empty_aliases = AliasSet::default();
        let empty_subs = SubcommandSet::new();
        let layers = layers_no_aliases(&global, &[], &project, &empty_aliases, &empty_subs);
        let out = render_variables(&layers);
        assert!(out.contains("## Variables"), "section header: {out}");
        let header = out.lines().find(|l| l.starts_with("| name")).unwrap();
        assert!(
            header.contains("from"),
            "table must carry a from column: {header}"
        );
        let row = out.lines().find(|l| l.contains("opt-flags")).unwrap();
        assert!(row.contains("-C opt-level=3"), "value: {row}");
        assert!(row.contains("global"), "from: {row}");
    }

    #[test]
    fn variables_section_skips_empty_scopes_entirely() {
        // profile:git has no vars; only global does. The output must not
        // contain a per-scope subheading or any `(none)` placeholder.
        let global = vset(&[("opt-flags", "-C opt-level=3")]);
        let empty_subs = SubcommandSet::new();
        let empty_aliases = AliasSet::default();
        let profiles = vec![ProfileLayer {
            name: "git".into(),
            aliases: AliasSet::default(),
            subcommands: SubcommandSet::new(),
            vars: VarSet::default(),
        }];
        let project = VarSet::default();
        let layers = layers_no_aliases(&global, &profiles, &project, &empty_aliases, &empty_subs);
        let out = render_variables(&layers);
        assert!(
            !out.contains("### profile:git"),
            "empty scope must not get a subsection: {out}"
        );
        assert!(
            !out.contains("(none)"),
            "no `(none)` placeholder for empty scope: {out}"
        );
        assert!(!out.contains("### "), "no `###` subsections at all: {out}");
    }

    #[test]
    fn variables_section_picks_highest_precedence_when_same_name_in_two_scopes() {
        // `foo` defined in global and profile:rust; profile wins, only one row.
        let global = vset(&[("foo", "global-value")]);
        let empty_subs = SubcommandSet::new();
        let empty_aliases = AliasSet::default();
        let profiles = vec![ProfileLayer {
            name: "rust".into(),
            aliases: AliasSet::default(),
            subcommands: SubcommandSet::new(),
            vars: vset(&[("foo", "rust-value")]),
        }];
        let project = VarSet::default();
        let layers = layers_no_aliases(&global, &profiles, &project, &empty_aliases, &empty_subs);
        let out = render_variables(&layers);
        let foo_rows: Vec<&str> = out.lines().filter(|l| l.contains("| foo ")).collect();
        assert_eq!(
            foo_rows.len(),
            1,
            "shadowed var must appear exactly once: {out}"
        );
        assert!(
            foo_rows[0].contains("rust-value"),
            "profile must win over global: {}",
            foo_rows[0]
        );
        assert!(
            foo_rows[0].contains("profile:rust"),
            "from must point at the winning scope: {}",
            foo_rows[0]
        );
        assert!(
            !out.contains("global-value"),
            "shadowed value must not appear at all: {out}"
        );
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
            unescaped_pipes, 4,
            "data row must have exactly 4 unescaped pipes (3 columns: leading + 2 separators + trailing): {data_line}"
        );
    }
}
