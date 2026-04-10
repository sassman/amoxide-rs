use crate::model::{
    AliasId, Column, ConfirmAction, Mode, MoveDestination, NodeKind, TransferMode, TuiMessage,
    TuiModel,
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
                        };
                        let _ = super::delegation::dispatch(
                            model,
                            amoxide::Message::RemoveAlias(id.name().to_string(), lib_target),
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
                    let _ = super::delegation::dispatch(model, amoxide::Message::RemoveProfile(name));
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
                    let msg = match &transfer_mode {
                        TransferMode::Move => amoxide::Message::MoveAliases {
                            aliases: aliases.iter().cloned().collect(),
                            to: lib_dest,
                        },
                        TransferMode::Copy => amoxide::Message::CopyAliases {
                            aliases: aliases.iter().cloned().collect(),
                            to: lib_dest,
                        },
                    };
                    let _ = super::delegation::dispatch(model, msg);
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
