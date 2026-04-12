use crate::model::{
    AliasId, Column, ConfirmAction, Mode, MoveDestination, NodeKind, TuiMessage, TuiModel,
};

pub fn handle(model: &mut TuiModel, msg: TuiMessage) {
    match msg {
        TuiMessage::DeleteItem => {
            if model.mode != Mode::Normal {
                return;
            }
            let node = match model.tree.get(model.cursor) {
                Some(n) => n.clone(),
                None => return,
            };
            match node.kind {
                NodeKind::AliasItem => {
                    if let Some(id) = node.alias_id {
                        let lib_target = match &id {
                            AliasId::Global { .. } => amoxide::AliasTarget::Global,
                            AliasId::Profile { profile_name, .. } => {
                                amoxide::AliasTarget::Profile(profile_name.clone())
                            }
                            AliasId::Project { .. } => amoxide::AliasTarget::Local,
                            AliasId::Subcommand { .. } => return,
                        };
                        let _ = super::delegation::dispatch(
                            model,
                            amoxide::Message::RemoveAlias(id.name().to_string(), lib_target),
                        );
                    }
                }
                NodeKind::SubcommandItem => {
                    if let Some(AliasId::Subcommand { scope, key }) = node.alias_id {
                        let lib_target = match &scope {
                            amoxide::SubcommandScope::Global => amoxide::AliasTarget::Global,
                            amoxide::SubcommandScope::Profile(n) => {
                                amoxide::AliasTarget::Profile(n.clone())
                            }
                            amoxide::SubcommandScope::Project => amoxide::AliasTarget::Local,
                        };
                        let _ = super::delegation::dispatch(
                            model,
                            amoxide::Message::RemoveSubcommandAlias(key, lib_target),
                        );
                    }
                }
                NodeKind::SubcommandGroupNode => {
                    let prefix = derive_key_prefix_from_cursor(model);
                    let keys_to_remove: Vec<String> = {
                        let lib_target = derive_target_from_cursor(model);
                        get_subcommand_set(model, &lib_target)
                            .keys()
                            .filter(|k| k.starts_with(&prefix))
                            .cloned()
                            .collect()
                    };
                    for key in keys_to_remove {
                        let _ = super::delegation::dispatch(
                            model,
                            amoxide::Message::RemoveSubcommandAlias(
                                key,
                                derive_target_from_cursor(model),
                            ),
                        );
                    }
                }
                NodeKind::SubcommandProgramHeader => {
                    let program = node
                        .label
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    let prog_prefix = format!("{program}:");
                    let keys_to_remove: Vec<String> = {
                        let lib_target = derive_target_from_cursor(model);
                        get_subcommand_set(model, &lib_target)
                            .keys()
                            .filter(|k| k.starts_with(&prog_prefix))
                            .cloned()
                            .collect()
                    };
                    for key in keys_to_remove {
                        let _ = super::delegation::dispatch(
                            model,
                            amoxide::Message::RemoveSubcommandAlias(
                                key,
                                derive_target_from_cursor(model),
                            ),
                        );
                    }
                }
                NodeKind::ProfileHeader => {
                    model.mode = Mode::Confirm(ConfirmAction::DeleteProfile(node.label.clone()));
                }
                _ => {}
            }
        }
        TuiMessage::ConfirmYes => {
            let action = match &model.mode {
                Mode::Confirm(a) => a.clone(),
                _ => return,
            };
            match action {
                ConfirmAction::DeleteProfile(name) => {
                    let _ =
                        super::delegation::dispatch(model, amoxide::Message::RemoveProfile(name));
                }
                ConfirmAction::OverwriteAliases {
                    aliases,
                    destination,
                    transfer_mode,
                } => {
                    let lib_dest = match &destination {
                        MoveDestination::Global => amoxide::AliasTarget::Global,
                        MoveDestination::Project => amoxide::AliasTarget::Local,
                        MoveDestination::Profile(n) => amoxide::AliasTarget::Profile(n.clone()),
                    };
                    super::transfer::dispatch_transfer(model, &aliases, &transfer_mode, lib_dest);
                    model.selected.clear();
                    model.active_column = Column::Left;
                }
            }
            model.mode = Mode::Normal;
        }
        TuiMessage::ConfirmNo => {
            model.mode = Mode::Normal;
        }
        TuiMessage::UseProfile => {
            if model.mode != Mode::Normal {
                return;
            }
            let node = match model.tree.get(model.cursor) {
                Some(n) => n.clone(),
                None => return,
            };
            if node.kind == NodeKind::ProfileHeader {
                let _ = super::delegation::dispatch(
                    model,
                    amoxide::Message::ToggleProfiles(vec![node.label.clone()]),
                );
            }
        }
        TuiMessage::UseProfileWithPriority(n) => {
            if model.mode != Mode::Normal {
                return;
            }
            let node = match model.tree.get(model.cursor) {
                Some(n_node) => n_node.clone(),
                None => return,
            };
            if node.kind == NodeKind::ProfileHeader {
                let _ = super::delegation::dispatch(
                    model,
                    amoxide::Message::UseProfilesAt(vec![node.label.clone()], n),
                );
            }
        }
        _ => {}
    }
}

fn derive_target_from_cursor(model: &TuiModel) -> amoxide::AliasTarget {
    for i in (0..=model.cursor).rev() {
        match &model.tree[i].kind {
            NodeKind::GlobalHeader => return amoxide::AliasTarget::Global,
            NodeKind::ProjectHeader => return amoxide::AliasTarget::Local,
            NodeKind::ProfileHeader => {
                return amoxide::AliasTarget::Profile(model.tree[i].label.clone())
            }
            _ => {}
        }
    }
    amoxide::AliasTarget::Global
}

fn derive_key_prefix_from_cursor(model: &TuiModel) -> String {
    let prog_idx = (0..=model.cursor)
        .rev()
        .find(|&i| model.tree[i].kind == NodeKind::SubcommandProgramHeader);
    let Some(pidx) = prog_idx else {
        return String::new();
    };
    let program = model.tree[pidx]
        .label
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string();
    let mut segments = vec![program];
    for node in &model.tree[pidx + 1..=model.cursor] {
        match node.kind {
            NodeKind::SubcommandGroupNode => segments.push(node.label.clone()),
            _ => break,
        }
    }
    segments.join(":")
}

fn get_subcommand_set<'a>(
    model: &'a TuiModel,
    target: &amoxide::AliasTarget,
) -> &'a amoxide::SubcommandSet {
    static EMPTY: std::sync::LazyLock<amoxide::SubcommandSet> =
        std::sync::LazyLock::new(amoxide::SubcommandSet::new);
    match target {
        amoxide::AliasTarget::Global => &model.app_model.config.subcommands,
        amoxide::AliasTarget::Local => model
            .app_model
            .project_aliases()
            .map(|p| &p.subcommands)
            .unwrap_or(&EMPTY),
        amoxide::AliasTarget::Profile(name) => model
            .app_model
            .profile_config()
            .get_profile_by_name(name)
            .map(|p| &p.subcommands)
            .unwrap_or(&EMPTY),
        _ => &model.app_model.config.subcommands,
    }
}
