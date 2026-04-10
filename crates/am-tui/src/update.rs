use crate::model::{
    AliasField, AliasId, AliasTarget, Column, ConfirmAction, Mode, MoveDestination, NodeKind,
    TextInputState, TransferMode, TreeNode, TuiMessage, TuiModel,
};
use amoxide::AliasName;

pub fn update(model: &mut TuiModel, msg: TuiMessage) {
    match msg {
        TuiMessage::CursorDown => {
            // Extract cursor and tree length before any mutable borrow.
            let (cursor, tree_len, next) = {
                let tree = active_tree(model);
                let cursor = *active_cursor(model);
                let next = (cursor + 1..tree.len()).find(|&i| tree[i].kind.is_navigable());
                (cursor, tree.len(), next)
            };
            let _ = (cursor, tree_len); // suppress unused-variable warnings
            if let Some(next_idx) = next {
                *active_cursor_mut(model) = next_idx;
            }
        }
        TuiMessage::CursorUp => {
            let (cursor, prev) = {
                let tree = active_tree(model);
                let cursor = *active_cursor(model);
                let prev = (0..cursor).rev().find(|&i| tree[i].kind.is_navigable());
                (cursor, prev)
            };
            let _ = cursor;
            if let Some(prev_idx) = prev {
                *active_cursor_mut(model) = prev_idx;
            }
        }
        TuiMessage::JumpTop => {
            let first = {
                let tree = active_tree(model);
                (0..tree.len()).find(|&i| tree[i].kind.is_navigable())
            };
            if let Some(first_idx) = first {
                *active_cursor_mut(model) = first_idx;
            }
        }
        TuiMessage::JumpBottom => {
            let last = {
                let tree = active_tree(model);
                (0..tree.len()).rev().find(|&i| tree[i].kind.is_navigable())
            };
            if let Some(last_idx) = last {
                *active_cursor_mut(model) = last_idx;
            }
        }
        TuiMessage::ToggleSelect => {
            if model.mode != Mode::Normal && !matches!(model.mode, Mode::Transfer(_)) {
                return;
            }
            if let Some(node) = model.tree.get(model.cursor).cloned() {
                if node.kind.is_selectable() {
                    // Single alias toggle
                    if let Some(ref id) = node.alias_id {
                        if !model.selected.remove(id) {
                            model.selected.insert(id.clone());
                        }
                    }
                } else {
                    // Header: toggle all aliases inside this container
                    let child_ids = collect_child_aliases(model, model.cursor);
                    if !child_ids.is_empty() {
                        // If any are selected, deselect all. Otherwise select all.
                        let any_selected = child_ids.iter().any(|id| model.selected.contains(id));
                        if any_selected {
                            for id in &child_ids {
                                model.selected.remove(id);
                            }
                        } else {
                            for id in child_ids {
                                model.selected.insert(id);
                            }
                        }
                    }
                }
            }
        }
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
        TuiMessage::SwitchColumn => {
            if matches!(model.mode, Mode::Transfer(_)) {
                model.active_column = match model.active_column {
                    Column::Left => Column::Right,
                    Column::Right => Column::Left,
                };
            }
        }
        TuiMessage::ExecuteTransfer => {
            if matches!(model.mode, Mode::Transfer(_)) && model.active_column == Column::Right {
                execute_transfer(model);
            }
        }
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
                        let _ = crate::delegation::dispatch(
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
                    let _ = crate::delegation::dispatch(model, amoxide::Message::RemoveProfile(name));
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
                    let _ = crate::delegation::dispatch(model, msg);
                    model.selected.clear();
                    model.active_column = Column::Left;
                }
            }
            model.mode = Mode::Normal;
        }
        TuiMessage::ConfirmNo => {
            model.mode = Mode::Normal;
        }
        TuiMessage::StartCreateProfile => {
            if model.mode == Mode::Normal {
                model.mode = Mode::TextInput(TextInputState::NewProfile(String::new()));
            }
        }
        TuiMessage::StartAddAlias => {
            if model.mode != Mode::Normal {
                return;
            }
            // Determine target scope from cursor position
            let target = resolve_alias_target(model);
            if let Some(target) = target {
                model.mode = Mode::TextInput(TextInputState::NewAlias {
                    name: String::new(),
                    command: String::new(),
                    active_field: AliasField::Name,
                    target,
                });
            }
        }
        TuiMessage::TextInputChar(c) => {
            if let Mode::TextInput(ref mut state) = model.mode {
                match state {
                    TextInputState::NewProfile(buf) => {
                        buf.push(c);
                    }
                    TextInputState::NewAlias {
                        name,
                        command,
                        active_field,
                        ..
                    } => match active_field {
                        AliasField::Name => {
                            name.push(c);
                        }
                        AliasField::Command => {
                            command.push(c);
                        }
                    },
                    TextInputState::EditProfile { name, error, .. } => {
                        *error = None;
                        name.push(c);
                    }
                    TextInputState::EditAlias {
                        name,
                        command,
                        active_field,
                        error,
                        ..
                    } => {
                        *error = None;
                        match active_field {
                            AliasField::Name => name.push(c),
                            AliasField::Command => command.push(c),
                        }
                    }
                }
            }
        }
        TuiMessage::TextInputBackspace => {
            if let Mode::TextInput(ref mut state) = model.mode {
                match state {
                    TextInputState::NewProfile(buf) => {
                        buf.pop();
                    }
                    TextInputState::NewAlias {
                        name,
                        command,
                        active_field,
                        ..
                    } => match active_field {
                        AliasField::Name => {
                            name.pop();
                        }
                        AliasField::Command => {
                            command.pop();
                        }
                    },
                    TextInputState::EditProfile { name, error, .. } => {
                        *error = None;
                        name.pop();
                    }
                    TextInputState::EditAlias {
                        name,
                        command,
                        active_field,
                        error,
                        ..
                    } => {
                        *error = None;
                        match active_field {
                            AliasField::Name => {
                                name.pop();
                            }
                            AliasField::Command => {
                                command.pop();
                            }
                        }
                    }
                }
            }
        }
        TuiMessage::TextInputSwitchField => {
            if let Mode::TextInput(
                TextInputState::NewAlias { active_field, .. }
                | TextInputState::EditAlias { active_field, .. },
            ) = &mut model.mode
            {
                *active_field = match active_field {
                    AliasField::Name => AliasField::Command,
                    AliasField::Command => AliasField::Name,
                };
            }
        }
        TuiMessage::TextInputConfirm => {
            let state = match &model.mode {
                Mode::TextInput(s) => s.clone(),
                _ => return,
            };
            match state {
                TextInputState::NewProfile(name) => {
                    if name.is_empty() {
                        return;
                    }
                    if model
                        .app_model
                        .profile_config()
                        .get_profile_by_name(&name)
                        .is_some()
                    {
                        return;
                    }
                    let _ = crate::delegation::dispatch(model, amoxide::Message::CreateProfile(name));
                    model.mode = Mode::Normal;
                }
                TextInputState::NewAlias {
                    name,
                    command,
                    target,
                    ..
                } => {
                    let name = name.trim().to_string();
                    let command = command.trim().to_string();
                    if name.is_empty() || command.is_empty() {
                        return;
                    }
                    let lib_target = match &target {
                        AliasTarget::Global => amoxide::AliasTarget::Global,
                        AliasTarget::Profile(n) => amoxide::AliasTarget::Profile(n.clone()),
                        AliasTarget::Project => amoxide::AliasTarget::Local,
                    };
                    let _ = crate::delegation::dispatch(
                        model,
                        amoxide::Message::AddAlias(name, command, lib_target, false),
                    );
                    model.mode = Mode::Normal;
                }
                TextInputState::EditProfile {
                    original_name,
                    name,
                    ..
                } => {
                    let name = name.trim().to_string();
                    if name.is_empty() {
                        return;
                    }
                    if name == original_name {
                        model.mode = Mode::Normal;
                        return;
                    }
                    if model
                        .app_model
                        .profile_config()
                        .get_profile_by_name(&name)
                        .is_some()
                    {
                        if let Mode::TextInput(TextInputState::EditProfile { error, .. }) =
                            &mut model.mode
                        {
                            *error = Some(format!("profile '{}' already exists", name));
                        }
                        return;
                    }
                    let _ = crate::delegation::dispatch(
                        model,
                        amoxide::Message::RenameProfile {
                            old_name: original_name,
                            new_name: name,
                        },
                    );
                    model.mode = Mode::Normal;
                }
                TextInputState::EditAlias {
                    alias_id,
                    name,
                    command,
                    ..
                } => {
                    let new_name = name.trim().to_string();
                    let new_command = command.trim().to_string();
                    if new_name.is_empty() || new_command.is_empty() {
                        return;
                    }
                    let original_name = match &alias_id {
                        AliasId::Global { alias_name }
                        | AliasId::Profile { alias_name, .. }
                        | AliasId::Project { alias_name } => alias_name.clone(),
                    };
                    // No change — just exit
                    if new_name == original_name {
                        let key = AliasName::from(original_name.as_str());
                        let original_command = match &alias_id {
                            AliasId::Global { .. } => model
                                .app_model
                                .config
                                .aliases
                                .get(&key)
                                .map(|a| a.command().to_string()),
                            AliasId::Profile { profile_name, .. } => model
                                .app_model
                                .profile_config()
                                .get_profile_by_name(profile_name)
                                .and_then(|p| p.aliases.get(&key).map(|a| a.command().to_string())),
                            AliasId::Project { .. } => model
                                .app_model
                                .project_aliases()
                                .and_then(|p| p.aliases.get(&key).map(|a| a.command().to_string())),
                        };
                        if original_command.as_deref() == Some(new_command.as_str()) {
                            model.mode = Mode::Normal;
                            return;
                        }
                    }
                    // Name collision check
                    if new_name != original_name {
                        let key = AliasName::from(new_name.as_str());
                        let exists = match &alias_id {
                            AliasId::Global { .. } => {
                                model.app_model.config.aliases.contains_key(&key)
                            }
                            AliasId::Profile { profile_name, .. } => model
                                .app_model
                                .profile_config()
                                .get_profile_by_name(profile_name)
                                .is_some_and(|p| p.aliases.contains_key(&key)),
                            AliasId::Project { .. } => model
                                .app_model
                                .project_aliases()
                                .is_some_and(|p| p.aliases.contains_key(&key)),
                        };
                        if exists {
                            if let Mode::TextInput(TextInputState::EditAlias { error, .. }) =
                                &mut model.mode
                            {
                                *error = Some(format!(
                                    "name '{}' already exists in this scope",
                                    new_name
                                ));
                            }
                            return;
                        }
                    }
                    // Apply via dispatch
                    let lib_target = match &alias_id {
                        AliasId::Global { .. } => amoxide::AliasTarget::Global,
                        AliasId::Profile { profile_name, .. } => {
                            amoxide::AliasTarget::Profile(profile_name.clone())
                        }
                        AliasId::Project { .. } => amoxide::AliasTarget::Local,
                    };
                    let _ = crate::delegation::dispatch(
                        model,
                        amoxide::Message::UpdateAlias {
                            target: lib_target,
                            old_name: original_name,
                            new_name,
                            new_command,
                            raw: false,
                        },
                    );
                    model.mode = Mode::Normal;
                }
            }
        }
        TuiMessage::TextInputCancel => {
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
                let _ = crate::delegation::dispatch(
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
                let _ = crate::delegation::dispatch(
                    model,
                    amoxide::Message::UseProfilesAt(vec![node.label.clone()], n),
                );
            }
        }
        TuiMessage::EditItem => {
            if model.mode != Mode::Normal {
                return;
            }
            let node = match model.tree.get(model.cursor) {
                Some(n) => n.clone(),
                None => return,
            };
            match node.kind {
                NodeKind::ProfileHeader => {
                    model.mode = Mode::TextInput(TextInputState::EditProfile {
                        original_name: node.label.clone(),
                        name: node.label.clone(),
                        error: None,
                    });
                }
                NodeKind::AliasItem => {
                    if let (Some(id), Some(cmd)) =
                        (node.alias_id.clone(), node.alias_command.clone())
                    {
                        model.mode = Mode::TextInput(TextInputState::EditAlias {
                            alias_id: id,
                            name: node.label.clone(),
                            command: cmd,
                            active_field: AliasField::Name,
                            error: None,
                        });
                    }
                }
                _ => {}
            }
        }
        TuiMessage::ToggleTrust => {
            if model.mode != Mode::Normal {
                return;
            }
            let node = match model.tree.get(model.cursor) {
                Some(n) => n.clone(),
                None => return,
            };
            if node.kind != NodeKind::ProjectHeader {
                return;
            }
            let is_trusted = model
                .app_model
                .project_trust()
                .map(|t| t.is_trusted())
                .unwrap_or(false);
            let msg = if is_trusted {
                amoxide::Message::Untrust { forget: false }
            } else {
                amoxide::Message::Trust
            };
            let _ = crate::delegation::dispatch(model, msg);
            // rebuild_tree is called by dispatch on success
        }
        _ => {} // remaining messages (Quit, Resize) handled at the app layer
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
        let _ = crate::delegation::dispatch(model, msg);
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

fn is_same_source(id: &AliasId, dest: &MoveDestination) -> bool {
    match (id, dest) {
        (AliasId::Global { .. }, MoveDestination::Global) => true,
        (AliasId::Project { .. }, MoveDestination::Project) => true,
        (AliasId::Profile { profile_name, .. }, MoveDestination::Profile(dest_name)) => {
            profile_name == dest_name
        }
        _ => false,
    }
}

fn alias_exists_at_dest(model: &TuiModel, id: &AliasId, dest: &MoveDestination) -> bool {
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

/// Collect all AliasItem ids that belong to the container at `header_idx`.
/// Walks forward from the header, collecting consecutive AliasItem nodes.
fn collect_child_aliases(model: &TuiModel, header_idx: usize) -> Vec<AliasId> {
    let mut ids = Vec::new();
    for node in model.tree.iter().skip(header_idx + 1) {
        match node.kind {
            NodeKind::AliasItem => {
                if let Some(ref id) = node.alias_id {
                    ids.push(id.clone());
                }
            }
            // Stop at the next header — those aliases belong to a different container
            _ => break,
        }
    }
    ids
}

/// Determine the alias target scope from the current cursor position.
/// Returns None if cursor is not on a node that implies a scope.
fn resolve_alias_target(model: &TuiModel) -> Option<AliasTarget> {
    let node = model.tree.get(model.cursor)?;
    match &node.kind {
        NodeKind::GlobalHeader => Some(AliasTarget::Global),
        NodeKind::ProjectHeader => Some(AliasTarget::Project),
        NodeKind::ProfileHeader => Some(AliasTarget::Profile(node.label.clone())),
        NodeKind::AliasItem => {
            // Derive target from the alias's scope
            match &node.alias_id {
                Some(AliasId::Global { .. }) => Some(AliasTarget::Global),
                Some(AliasId::Profile { profile_name, .. }) => {
                    Some(AliasTarget::Profile(profile_name.clone()))
                }
                Some(AliasId::Project { .. }) => Some(AliasTarget::Project),
                None => None,
            }
        }
    }
}

fn active_tree(model: &TuiModel) -> &[TreeNode] {
    match model.active_column {
        Column::Left => &model.tree,
        Column::Right => &model.dest_tree,
    }
}

fn active_cursor(model: &TuiModel) -> &usize {
    match model.active_column {
        Column::Left => &model.cursor,
        Column::Right => &model.dest_cursor,
    }
}

fn active_cursor_mut(model: &mut TuiModel) -> &mut usize {
    match model.active_column {
        Column::Left => &mut model.cursor,
        Column::Right => &mut model.dest_cursor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Mode, TuiMessage, TuiModel};
    use crate::tree::{build_dest_tree_from_parts, build_tree_from_parts};
    use amoxide::update::AppModel;
    use amoxide::{Config, ProfileConfig};
    use std::collections::BTreeSet;

    fn test_model(profiles_toml: &str) -> TuiModel {
        let profiles: ProfileConfig = toml::from_str(profiles_toml).unwrap();
        let config = Config::default();
        let app_model = AppModel::new(config, profiles);
        let tree = build_tree_from_parts(
            &app_model.config.aliases,
            app_model.profile_config(),
            &app_model.config.active_profiles,
            None,
        );
        let dest_tree = build_dest_tree_from_parts(
            &app_model.config.aliases,
            app_model.profile_config(),
            &app_model.config.active_profiles,
            false,
        );
        TuiModel {
            app_model,
            tree,
            cursor: 0,
            selected: BTreeSet::new(),
            mode: Mode::Normal,
            dest_tree,
            dest_cursor: 0,
            active_column: Column::Left,
            scroll_offset: 0,
            status_line: None,
        }
    }

    #[test]
    fn test_cursor_down_moves_to_next_navigable() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
            gp = "git push"
        "#,
        );
        update(&mut model, TuiMessage::CursorDown);
        assert_eq!(model.cursor, 1);
    }

    #[test]
    fn test_cursor_up_at_top_stays() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
        "#,
        );
        model.cursor = 0;
        update(&mut model, TuiMessage::CursorUp);
        assert_eq!(model.cursor, 0);
    }

    #[test]
    fn test_cursor_down_at_bottom_stays() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let last = model.tree.len() - 1;
        model.cursor = last;
        update(&mut model, TuiMessage::CursorDown);
        assert_eq!(model.cursor, last);
    }

    #[test]
    fn test_toggle_select_on_alias() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let alias_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::AliasItem)
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        assert_eq!(model.selected.len(), 1);
        update(&mut model, TuiMessage::ToggleSelect);
        assert!(model.selected.is_empty());
    }

    #[test]
    fn test_toggle_select_on_header_is_noop() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        model.cursor = 0;
        update(&mut model, TuiMessage::ToggleSelect);
        assert!(model.selected.is_empty());
    }

    #[test]
    fn test_jump_top_and_bottom() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "a"
            [profiles.aliases]
            x = "cmd1"

            [[profiles]]
            name = "b"
            [profiles.aliases]
            y = "cmd2"
        "#,
        );
        update(&mut model, TuiMessage::JumpBottom);
        assert!(model.cursor > 0);
        update(&mut model, TuiMessage::JumpTop);
        assert_eq!(model.cursor, 0);
    }

    #[test]
    fn test_enter_move_mode_with_selection() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let alias_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::AliasItem)
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        update(&mut model, TuiMessage::EnterMoveMode);
        assert_eq!(model.mode, Mode::Transfer(TransferMode::Move));
        assert_eq!(model.active_column, Column::Right);
    }

    #[test]
    fn test_enter_move_mode_without_selection_is_noop() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        update(&mut model, TuiMessage::EnterMoveMode);
        assert_eq!(model.mode, Mode::Normal);
    }

    #[test]
    fn test_cancel_move_clears_selection() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let alias_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::AliasItem)
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        update(&mut model, TuiMessage::EnterMoveMode);
        update(&mut model, TuiMessage::CancelTransfer);
        assert_eq!(model.mode, Mode::Normal);
        assert!(model.selected.is_empty());
        assert_eq!(model.active_column, Column::Left);
    }

    #[test]
    fn test_switch_column_in_move_mode() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let alias_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::AliasItem)
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        update(&mut model, TuiMessage::EnterMoveMode);
        assert_eq!(model.active_column, Column::Right);
        update(&mut model, TuiMessage::SwitchColumn);
        assert_eq!(model.active_column, Column::Left);
        update(&mut model, TuiMessage::SwitchColumn);
        assert_eq!(model.active_column, Column::Right);
    }

    #[test]
    fn test_switch_column_in_normal_mode_is_noop() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
        "#,
        );
        update(&mut model, TuiMessage::SwitchColumn);
        assert_eq!(model.active_column, Column::Left);
    }

    #[test]
    fn test_same_source_move_is_noop() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let alias_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::AliasItem)
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        update(&mut model, TuiMessage::EnterMoveMode);
        let git_idx = model
            .dest_tree
            .iter()
            .position(|n| n.label == "git")
            .unwrap();
        model.dest_cursor = git_idx;
        update(&mut model, TuiMessage::ExecuteTransfer);
        assert_eq!(model.mode, Mode::Normal);
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("git")
            .unwrap()
            .aliases
            .iter()
            .any(|(n, _)| n.as_ref() == "gs"));
    }

    // --- Task 8: Delete + Confirm ---

    #[test]
    fn test_delete_alias() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        // cursor index 1 should be the alias item under the "git" profile header
        let alias_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::AliasItem)
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::DeleteItem);
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("git")
            .unwrap()
            .aliases
            .is_empty());
    }

    #[test]
    fn test_delete_profile_enters_confirm() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let header_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "git")
            .unwrap();
        model.cursor = header_idx;
        update(&mut model, TuiMessage::DeleteItem);
        assert_eq!(
            model.mode,
            Mode::Confirm(ConfirmAction::DeleteProfile("git".to_string()))
        );
    }

    #[test]
    fn test_confirm_yes_deletes_profile() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#,
        );
        // Set up confirm mode for deleting "git"
        model.mode = Mode::Confirm(ConfirmAction::DeleteProfile("git".to_string()));
        update(&mut model, TuiMessage::ConfirmYes);
        assert_eq!(model.mode, Mode::Normal);
        assert_eq!(model.app_model.profile_config().len(), 1);
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("git")
            .is_none());
    }

    #[test]
    fn test_confirm_no_cancels_delete() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
        "#,
        );
        model.mode = Mode::Confirm(ConfirmAction::DeleteProfile("git".to_string()));
        update(&mut model, TuiMessage::ConfirmNo);
        assert_eq!(model.mode, Mode::Normal);
        // Profile must still exist
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("git")
            .is_some());
    }

    // --- Task 9: Create Profile + Set Active ---

    #[test]
    fn test_start_create_profile_enters_text_input() {
        let mut model = test_model("profiles = []");
        update(&mut model, TuiMessage::StartCreateProfile);
        assert_eq!(
            model.mode,
            Mode::TextInput(TextInputState::NewProfile(String::new()))
        );
    }

    #[test]
    fn test_text_input_confirm_creates_profile() {
        let mut model = test_model("profiles = []");
        update(&mut model, TuiMessage::StartCreateProfile);
        for c in "newprof".chars() {
            update(&mut model, TuiMessage::TextInputChar(c));
        }
        update(&mut model, TuiMessage::TextInputConfirm);
        assert_eq!(model.mode, Mode::Normal);
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("newprof")
            .is_some());
    }

    #[test]
    fn test_text_input_cancel() {
        let mut model = test_model("profiles = []");
        update(&mut model, TuiMessage::StartCreateProfile);
        for c in "newprof".chars() {
            update(&mut model, TuiMessage::TextInputChar(c));
        }
        update(&mut model, TuiMessage::TextInputCancel);
        assert_eq!(model.mode, Mode::Normal);
        // Profile should NOT have been created
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("newprof")
            .is_none());
    }

    #[test]
    fn test_text_input_empty_confirm_is_noop() {
        let mut model = test_model("profiles = []");
        update(&mut model, TuiMessage::StartCreateProfile);
        // Confirm immediately without typing anything
        update(&mut model, TuiMessage::TextInputConfirm);
        // Mode must remain TextInput (empty buffer — no-op)
        assert_eq!(
            model.mode,
            Mode::TextInput(TextInputState::NewProfile(String::new()))
        );
        assert_eq!(model.app_model.profile_config().len(), 0);
    }

    #[test]
    fn test_toggle_active_profile() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"

            [[profiles]]
            name = "rust"
        "#,
        );
        let rust_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "rust")
            .unwrap();
        model.cursor = rust_idx;
        update(&mut model, TuiMessage::UseProfile);
        assert!(model.app_model.config.is_active("rust"));
        // Toggle again to deactivate
        // After rebuild, find rust again (position may have changed)
        let rust_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "rust")
            .unwrap();
        model.cursor = rust_idx;
        update(&mut model, TuiMessage::UseProfile);
        assert!(!model.app_model.config.is_active("rust"));
    }

    #[test]
    fn test_use_profile_with_priority() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"

            [[profiles]]
            name = "rust"
        "#,
        );
        // Activate git first
        let git_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "git")
            .unwrap();
        model.cursor = git_idx;
        update(&mut model, TuiMessage::UseProfile);
        assert!(model.app_model.config.is_active("git"));

        // Now activate rust at priority 1
        let rust_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "rust")
            .unwrap();
        model.cursor = rust_idx;
        update(&mut model, TuiMessage::UseProfileWithPriority(1));
        assert_eq!(
            model.app_model.config.active_profiles,
            vec!["rust".to_string(), "git".to_string()]
        );
    }

    // --- Copy-to feature ---

    #[test]
    fn test_enter_copy_mode_with_selection() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let alias_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::AliasItem)
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        update(&mut model, TuiMessage::EnterCopyMode);
        assert_eq!(model.mode, Mode::Transfer(TransferMode::Copy));
        assert_eq!(model.active_column, Column::Right);
    }

    #[test]
    fn test_copy_preserves_source() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "rust"
        "#,
        );
        let alias_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::AliasItem)
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        update(&mut model, TuiMessage::EnterCopyMode);
        let rust_idx = model
            .dest_tree
            .iter()
            .position(|n| n.label == "rust")
            .unwrap();
        model.dest_cursor = rust_idx;
        update(&mut model, TuiMessage::ExecuteTransfer);
        // Source still has the alias
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("git")
            .unwrap()
            .aliases
            .iter()
            .any(|(n, _)| n.as_ref() == "gs"));
        // Destination also has it
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("rust")
            .unwrap()
            .aliases
            .iter()
            .any(|(n, _)| n.as_ref() == "gs"));
    }

    #[test]
    fn test_copy_same_source_is_noop() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let alias_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::AliasItem)
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        update(&mut model, TuiMessage::EnterCopyMode);
        let git_idx = model
            .dest_tree
            .iter()
            .position(|n| n.label == "git")
            .unwrap();
        model.dest_cursor = git_idx;
        update(&mut model, TuiMessage::ExecuteTransfer);
        assert_eq!(model.mode, Mode::Normal);
    }

    // --- Edit feature ---

    #[test]
    fn test_edit_profile_enters_text_input() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
        "#,
        );
        let header_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "git")
            .unwrap();
        model.cursor = header_idx;
        update(&mut model, TuiMessage::EditItem);
        assert_eq!(
            model.mode,
            Mode::TextInput(TextInputState::EditProfile {
                original_name: "git".to_string(),
                name: "git".to_string(),
                error: None,
            })
        );
    }

    #[test]
    fn test_edit_profile_rename() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let header_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "git")
            .unwrap();
        model.cursor = header_idx;
        update(&mut model, TuiMessage::EditItem);
        for _ in 0..3 {
            update(&mut model, TuiMessage::TextInputBackspace);
        }
        for c in "vcs".chars() {
            update(&mut model, TuiMessage::TextInputChar(c));
        }
        update(&mut model, TuiMessage::TextInputConfirm);
        assert_eq!(model.mode, Mode::Normal);
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("vcs")
            .is_some());
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("git")
            .is_none());
    }

    #[test]
    fn test_edit_profile_duplicate_name_rejected() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"

            [[profiles]]
            name = "rust"
        "#,
        );
        let header_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "git")
            .unwrap();
        model.cursor = header_idx;
        update(&mut model, TuiMessage::EditItem);
        for _ in 0..3 {
            update(&mut model, TuiMessage::TextInputBackspace);
        }
        for c in "rust".chars() {
            update(&mut model, TuiMessage::TextInputChar(c));
        }
        update(&mut model, TuiMessage::TextInputConfirm);
        match &model.mode {
            Mode::TextInput(TextInputState::EditProfile { error, .. }) => {
                assert!(error.is_some());
            }
            other => panic!("expected EditProfile with error, got {other:?}"),
        }
    }

    #[test]
    fn test_edit_alias_changes_command() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#,
        );
        let alias_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::AliasItem)
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::EditItem);
        update(&mut model, TuiMessage::TextInputSwitchField);
        for _ in 0..10 {
            update(&mut model, TuiMessage::TextInputBackspace);
        }
        for c in "git status -sb".chars() {
            update(&mut model, TuiMessage::TextInputChar(c));
        }
        update(&mut model, TuiMessage::TextInputConfirm);
        assert_eq!(model.mode, Mode::Normal);
        let profile = model
            .app_model
            .profile_config()
            .get_profile_by_name("git")
            .unwrap();
        let key = AliasName::from("gs");
        assert_eq!(
            profile.aliases.get(&key).unwrap().command(),
            "git status -sb"
        );
    }

    #[test]
    fn test_edit_alias_name_collision_rejected() {
        let mut model = test_model(
            r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
            gp = "git push"
        "#,
        );
        let alias_idx = model
            .tree
            .iter()
            .position(|n| {
                n.kind == NodeKind::AliasItem
                    && n.alias_id
                        == Some(AliasId::Profile {
                            profile_name: "git".to_string(),
                            alias_name: "gs".to_string(),
                        })
            })
            .unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::EditItem);
        update(&mut model, TuiMessage::TextInputBackspace);
        update(&mut model, TuiMessage::TextInputBackspace);
        for c in "gp".chars() {
            update(&mut model, TuiMessage::TextInputChar(c));
        }
        update(&mut model, TuiMessage::TextInputConfirm);
        match &model.mode {
            Mode::TextInput(TextInputState::EditAlias { error, .. }) => {
                assert!(error.is_some());
            }
            other => panic!("expected EditAlias with error, got {other:?}"),
        }
    }

    #[test]
    fn test_edit_on_global_header_is_noop() {
        let mut model = test_model("profiles = []");
        model
            .app_model
            .config
            .add_alias("ll".into(), "ls -la".into(), false);
        model.rebuild_tree();
        assert_eq!(model.tree[model.cursor].kind, NodeKind::GlobalHeader);
        update(&mut model, TuiMessage::EditItem);
        assert_eq!(model.mode, Mode::Normal);
    }
}
