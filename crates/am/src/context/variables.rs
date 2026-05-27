use crate::vars::VarSet;

use super::escape::escape_md_cell;
use super::LayerInputs;

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
