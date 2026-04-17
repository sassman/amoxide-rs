use crate::model::{
    AliasId, Column, ConfirmAction, Mode, MoveDestination, NodeKind, TransferMode, TuiMessage,
    TuiModel,
};
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
        TuiMessage::ExecuteTransfer
            if matches!(model.mode, Mode::Transfer(_))
                && model.active_column == Column::Right =>
        {
            execute_transfer(model);
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
        dispatch_transfer(model, &aliases_to_transfer, &transfer_mode, lib_dest);
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

/// Dispatch the appropriate transfer messages for a mixed set of alias IDs.
/// Subcommand aliases are grouped by source scope and dispatched via
/// Copy/MoveSubcommandAliases; regular aliases use Copy/MoveAliases.
pub(super) fn dispatch_transfer(
    model: &mut crate::model::TuiModel,
    aliases: &[AliasId],
    transfer_mode: &TransferMode,
    lib_dest: amoxide::AliasTarget,
) {
    // Separate subcommand IDs from regular IDs
    let (subcmd_ids, alias_ids): (Vec<_>, Vec<_>) = aliases
        .iter()
        .cloned()
        .partition(|id| matches!(id, AliasId::Subcommand { .. }));

    // For subcommand IDs, group by source scope and dispatch one message per source
    use std::collections::BTreeMap;
    let mut by_source: BTreeMap<String, (amoxide::AliasTarget, Vec<String>)> = BTreeMap::new();
    for id in &subcmd_ids {
        if let AliasId::Subcommand { scope, key } = id {
            let from_target = subcmd_scope_to_target(scope);
            let source_key = format!("{scope:?}");
            by_source
                .entry(source_key)
                .or_insert_with(|| (from_target, Vec::new()))
                .1
                .push(key.clone());
        }
    }
    for (_, (from_target, keys)) in by_source {
        let to = match &lib_dest {
            amoxide::AliasTarget::Global => amoxide::AliasTarget::Global,
            amoxide::AliasTarget::Local => amoxide::AliasTarget::Local,
            amoxide::AliasTarget::Profile(n) => amoxide::AliasTarget::Profile(n.clone()),
            amoxide::AliasTarget::ActiveProfile => amoxide::AliasTarget::ActiveProfile,
        };
        let msg = match transfer_mode {
            TransferMode::Move => amoxide::Message::MoveSubcommandAliases {
                keys,
                from: from_target,
                to,
            },
            TransferMode::Copy => amoxide::Message::CopySubcommandAliases {
                keys,
                from: from_target,
                to,
            },
        };
        let _ = super::delegation::dispatch(model, msg);
    }

    // Regular aliases
    if !alias_ids.is_empty() {
        let to = lib_dest;
        let msg = match transfer_mode {
            TransferMode::Move => amoxide::Message::MoveAliases {
                aliases: alias_ids,
                to,
            },
            TransferMode::Copy => amoxide::Message::CopyAliases {
                aliases: alias_ids,
                to,
            },
        };
        let _ = super::delegation::dispatch(model, msg);
    }
}

fn subcmd_scope_to_target(scope: &amoxide::SubcommandScope) -> amoxide::AliasTarget {
    match scope {
        amoxide::SubcommandScope::Global => amoxide::AliasTarget::Global,
        amoxide::SubcommandScope::Profile(n) => amoxide::AliasTarget::Profile(n.clone()),
        amoxide::SubcommandScope::Project => amoxide::AliasTarget::Local,
    }
}

pub(super) fn is_same_source(id: &AliasId, dest: &MoveDestination) -> bool {
    match (id, dest) {
        (AliasId::Global { .. }, MoveDestination::Global) => true,
        (AliasId::Project { .. }, MoveDestination::Project) => true,
        (AliasId::Profile { profile_name, .. }, MoveDestination::Profile(dest_name)) => {
            profile_name == dest_name
        }
        (AliasId::Subcommand { scope, .. }, MoveDestination::Global) => {
            matches!(scope, amoxide::SubcommandScope::Global)
        }
        (AliasId::Subcommand { scope, .. }, MoveDestination::Project) => {
            matches!(scope, amoxide::SubcommandScope::Project)
        }
        (AliasId::Subcommand { scope, .. }, MoveDestination::Profile(dest_name)) => {
            matches!(scope, amoxide::SubcommandScope::Profile(n) if n == dest_name)
        }
        _ => false,
    }
}

pub(super) fn alias_exists_at_dest(model: &TuiModel, id: &AliasId, dest: &MoveDestination) -> bool {
    match id {
        AliasId::Subcommand { key, .. } => match dest {
            MoveDestination::Global => model.app_model.config.subcommands.contains_key(key),
            MoveDestination::Project => model
                .app_model
                .project_aliases()
                .is_some_and(|p| p.subcommands.contains_key(key)),
            MoveDestination::Profile(name) => model
                .app_model
                .profile_config()
                .get_profile_by_name(name)
                .is_some_and(|p| p.subcommands.contains_key(key)),
        },
        _ => {
            let alias_name_str = match id {
                AliasId::Global { alias_name }
                | AliasId::Profile { alias_name, .. }
                | AliasId::Project { alias_name } => alias_name.as_str(),
                AliasId::Subcommand { .. } => unreachable!(),
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
    }
}
