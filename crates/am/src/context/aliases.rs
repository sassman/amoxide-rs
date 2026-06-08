use crate::precedence::{EffectiveEntry, EntryKind, OriginScope};
use crate::Described;

use super::escape::escape_md_cell;
use super::LayerInputs;

/// Find the highest-precedence layer that defines `name`, returning its scope.
///
/// Precedence (highest first): project > profile (slice-end first) > global.
/// Profiles later in `profile_layers` win over earlier ones, matching the
/// merge order in `Precedence::resolve()`.
fn lookup_origin(name: &str, layers: &LayerInputs) -> Option<OriginScope> {
    use crate::alias::AliasName;
    let key = AliasName::from(name);
    if layers.project_aliases.contains_key(&key) {
        return Some(OriginScope::Project);
    }
    for layer in layers.profile_layers.iter().rev() {
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
    for layer in layers.profile_layers.iter().rev() {
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
///
/// The `description` column is only emitted when at least one row carries
/// a description — keeps the snapshot minimal for users who haven't set
/// any, while surfacing the user's own intent hints where they exist.
pub fn render_aliases_table(effective: &[EffectiveEntry], layers: &LayerInputs) -> String {
    struct Row {
        name: String,
        expansion: String,
        from: std::borrow::Cow<'static, str>,
        description: Option<String>,
    }

    let mut rows: Vec<Row> = Vec::new();
    for e in effective {
        match &e.kind {
            EntryKind::Alias(alias) => {
                let expansion = alias.command().to_string();
                let from = lookup_origin(&e.name, layers)
                    .unwrap_or_else(|| {
                        panic!(
                            "alias '{}' in effective set but not present in any layer input",
                            e.name
                        )
                    })
                    .as_from_label();
                rows.push(Row {
                    name: e.name.clone(),
                    expansion,
                    from,
                    description: alias.description().map(str::to_owned),
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
                        .unwrap_or_else(|| {
                            panic!(
                                "subcommand '{} {}' in effective set but not present in any layer input",
                                program,
                                sub.short_subcommands.join(" ")
                            )
                        })
                        .as_from_label();
                    rows.push(Row {
                        name,
                        expansion,
                        from,
                        description: sub.description.clone(),
                    });
                }
            }
            EntryKind::SubcommandKey { .. } => {
                // Tracking-only, never shown
            }
        }
    }

    rows.sort_by(|a, b| a.name.cmp(&b.name));

    let any_description = rows.iter().any(|r| r.description.is_some());

    let mut out = String::from("## Aliases\n\n");
    if any_description {
        out.push_str("| name | expands to | from | description |\n");
        out.push_str("|------|------------|------|-------------|\n");
        for r in rows {
            out.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                escape_md_cell(&r.name),
                escape_md_cell(&r.expansion),
                escape_md_cell(&r.from),
                escape_md_cell(r.description.as_deref().unwrap_or("")),
            ));
        }
    } else {
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
    }
    out
}

#[cfg(test)]
mod tests {
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
        use crate::subcommand::{SubcommandEntry, TomlSubcommand};
        let global = AliasSet::default();
        let mut global_subs = SubcommandSet::new();
        global_subs
            .as_mut()
            .insert("git:pl".into(), TomlSubcommand::Expansion(vec![]));
        global_subs
            .as_mut()
            .insert("git:psh".into(), TomlSubcommand::Expansion(vec![]));
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
                        description: None,
                    },
                    SubcommandEntry {
                        program: "git".into(),
                        short_subcommands: vec!["psh".into()],
                        long_subcommands: vec!["push".into()],
                        description: None,
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
    fn aliases_table_attributes_alias_to_last_profile_in_slice_when_two_profiles_share_name() {
        // Engine treats profile_layers[i+1] as higher precedence than [i].
        // With layers [git, rust] both defining `x`, the winner is rust.
        // The `from` cell must say `profile:rust`, not `profile:git`.
        use crate::precedence::ProfileLayer;
        let global = AliasSet::default();
        let project = AliasSet::default();
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
        let profiles = vec![git, rust];
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &profiles,
            project_aliases: &project,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let effective = vec![entry("x", "rust-x")];
        let out = render_aliases_table(&effective, &layers);
        let data_line = out.lines().find(|l| l.contains("| x ")).unwrap();
        assert!(
            data_line.contains("profile:rust"),
            "winner is rust (last in slice); got: {data_line}"
        );
        assert!(
            !data_line.contains("profile:git"),
            "loser must not be attributed; got: {data_line}"
        );
    }

    #[test]
    fn aliases_table_omits_description_column_when_no_row_has_one() {
        let global = aset(&[("ll", "ls -lha")]);
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
        let effective = vec![entry("ll", "ls -lha")];
        let out = render_aliases_table(&effective, &layers);
        let header = out.lines().find(|l| l.starts_with("| name")).unwrap();
        assert!(
            !header.contains("description"),
            "header must stay 3-column when no descriptions are present: {header}"
        );
    }

    #[test]
    fn aliases_table_surfaces_description_for_aliases_when_any_row_has_one() {
        use crate::alias::AliasDetail;
        let mut global = AliasSet::default();
        global.insert(
            AliasName::from("t"),
            TomlAlias::Detailed(AliasDetail {
                command: "cargo test --all-features".into(),
                description: Some("run the full test matrix".into()),
                raw: false,
            }),
        );
        // Second alias with no description — must still render with an empty
        // description cell, not be dropped or break column alignment.
        global.insert(AliasName::from("ll"), TomlAlias::Command("ls -lha".into()));
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
        let effective = vec![
            EffectiveEntry {
                name: "t".into(),
                kind: EntryKind::Alias(global.as_ref().get(&AliasName::from("t")).unwrap().clone()),
                hash: "x".into(),
            },
            EffectiveEntry {
                name: "ll".into(),
                kind: EntryKind::Alias(
                    global.as_ref().get(&AliasName::from("ll")).unwrap().clone(),
                ),
                hash: "x".into(),
            },
        ];
        let out = render_aliases_table(&effective, &layers);
        let header = out.lines().find(|l| l.starts_with("| name")).unwrap();
        assert!(
            header.contains("description"),
            "header must include description column when any row has one: {header}"
        );
        let t_row = out.lines().find(|l| l.contains("| t ")).unwrap();
        assert!(
            t_row.contains("run the full test matrix"),
            "described alias must carry its description in the row: {t_row}"
        );
        let ll_row = out.lines().find(|l| l.contains("| ll ")).unwrap();
        // 4 columns ⇒ 5 unescaped pipes per row (leading + 3 separators + trailing).
        let unescaped_pipes = ll_row
            .chars()
            .zip(std::iter::once(' ').chain(ll_row.chars()))
            .filter(|(c, prev)| *c == '|' && *prev != '\\')
            .count();
        assert_eq!(
            unescaped_pipes, 5,
            "row without a description must still emit an empty 4th cell: {ll_row}"
        );
    }

    #[test]
    fn aliases_table_surfaces_subcommand_descriptions() {
        use crate::subcommand::{SubcommandEntry, TomlSubcommand};
        let global = AliasSet::default();
        let mut global_subs = SubcommandSet::new();
        global_subs
            .as_mut()
            .insert("git:pl".into(), TomlSubcommand::Expansion(vec![]));
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
                entries: vec![SubcommandEntry {
                    program: "git".into(),
                    short_subcommands: vec!["pl".into()],
                    long_subcommands: vec!["pull --rebase".into()],
                    description: Some("rebase-based pull, never merge".into()),
                }],
                base_cmd: None,
            },
            hash: "x".into(),
        }];
        let out = render_aliases_table(&effective, &layers);
        assert!(
            out.contains("rebase-based pull, never merge"),
            "subcommand description must surface in the row: {out}"
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
