use crate::model::{AliasId, Column, ConfirmAction, Mode, MoveDestination, NodeKind, TransferMode, TuiMessage, TuiModel};
use amoxide::AliasName;

pub fn handle(model: &mut TuiModel, msg: TuiMessage) {
    match msg {
        TuiMessage::EnterMoveMode => {
            if model.mode != Mode::Normal {
                return;
            }
            // If nothing selected, auto-select the alias under cursor
            if model.selected.is_empty() {
                if let Some(node) = model.tree.get(model.cursor) {
                    if node.kind.is_selectable() {
                        if let Some(ref id) = node.alias_id {
                            model.selected.insert(id.clone());
                        }
                    }
                }
            }
            if !model.selected.is_empty() {
                model.mode = Mode::Transfer(TransferMode::Move);
                model.active_column = Column::Right;
            }
        }
        TuiMessage::EnterCopyMode => {
            if model.mode != Mode::Normal {
                return;
            }
            if model.selected.is_empty() {
                if let Some(node) = model.tree.get(model.cursor) {
                    if node.kind.is_selectable() {
                        if let Some(ref id) = node.alias_id {
                            model.selected.insert(id.clone());
                        }
                    }
                }
            }
            if !model.selected.is_empty() {
                model.mode = Mode::Transfer(TransferMode::Copy);
                model.active_column = Column::Right;
            }
        }
        TuiMessage::CancelTransfer => {
            model.selected.clear();
            model.mode = Mode::Normal;
            model.active_column = Column::Left;
        }
        TuiMessage::ExecuteTransfer => {
            if matches!(model.mode, Mode::Transfer(_)) && model.active_column == Column::Right {
                execute_transfer(model);
            }
        }
        _ => {}
    }
}

fn execute_transfer(model: &mut TuiModel) {
    let transfer_mode = match &model.mode {
        Mode::Transfer(tm) => tm.clone(),
        _ => return,
    };

    let dest_node = match model.dest_tree.get(model.dest_cursor) {
        Some(node) => node.clone(),
        None => return,
    };

    let destination = match dest_node.kind {
        NodeKind::GlobalHeader => MoveDestination::Global,
        NodeKind::ProjectHeader => MoveDestination::Project,
        NodeKind::ProfileHeader => MoveDestination::Profile(dest_node.label.clone()),
        _ => return,
    };

    // Filter out aliases that are already at the destination (same-source transfers).
    let aliases_to_transfer: Vec<AliasId> = model
        .selected
        .iter()
        .filter(|id| !is_same_source(id, &destination))
        .cloned()
        .collect();

    if aliases_to_transfer.is_empty() {
        // All selected aliases are already at the destination — treat as no-op.
        model.selected.clear();
        model.mode = Mode::Normal;
        model.active_column = Column::Left;
        return;
    }

    // Check for collisions: aliases that already exist at the destination.
    let collisions: Vec<AliasId> = aliases_to_transfer
        .iter()
        .filter(|id| alias_exists_at_dest(model, id, &destination))
        .cloned()
        .collect();

    if collisions.is_empty() {
        let lib_dest = match &destination {
            MoveDestination::Global => amoxide::AliasTarget::Global,
            MoveDestination::Project => amoxide::AliasTarget::Local,
            MoveDestination::Profile(n) => amoxide::AliasTarget::Profile(n.clone()),
        };
        let msg = match transfer_mode {
            TransferMode::Move => amoxide::Message::MoveAliases {
                aliases: aliases_to_transfer.iter().cloned().collect(),
                to: lib_dest,
            },
            TransferMode::Copy => amoxide::Message::CopyAliases {
                aliases: aliases_to_transfer.iter().cloned().collect(),
                to: lib_dest,
            },
        };
        let _ = super::delegation::dispatch(model, msg);
        model.selected.clear();
        model.mode = Mode::Normal;
        model.active_column = Column::Left;
    } else {
        model.mode = Mode::Confirm(ConfirmAction::OverwriteAliases {
            aliases: aliases_to_transfer,
            destination,
            transfer_mode,
        });
    }
}

pub(super) fn is_same_source(id: &AliasId, dest: &MoveDestination) -> bool {
    match (id, dest) {
        (AliasId::Global { .. }, MoveDestination::Global) => true,
        (AliasId::Project { .. }, MoveDestination::Project) => true,
        (AliasId::Profile { profile_name, .. }, MoveDestination::Profile(dest_name)) => {
            profile_name == dest_name
        }
        _ => false,
    }
}

pub(super) fn alias_exists_at_dest(model: &TuiModel, id: &AliasId, dest: &MoveDestination) -> bool {
    let alias_name_str = match id {
        AliasId::Global { alias_name }
        | AliasId::Profile { alias_name, .. }
        | AliasId::Project { alias_name } => alias_name.as_str(),
    };
    let key = AliasName::from(alias_name_str);
    match dest {
        MoveDestination::Global => model.app_model.config.aliases.contains_key(&key),
        MoveDestination::Project => model
            .app_model
            .project_aliases()
            .is_some_and(|p| p.aliases.contains_key(&key)),
        MoveDestination::Profile(name) => model
            .app_model
            .profile_config()
            .get_profile_by_name(name)
            .is_some_and(|p| p.aliases.contains_key(&key)),
    }
}
