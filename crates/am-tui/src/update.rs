use crate::model::{AliasField, AliasId, AliasTarget, Column, ConfirmAction, Mode, MoveDestination, NodeKind, TextInputState, TreeNode, TuiMessage, TuiModel};
use amoxide::{AliasName, TomlAlias, ProjectAliases};

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
            if model.mode != Mode::Normal && model.mode != Mode::Moving {
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
                model.mode = Mode::Moving;
                model.active_column = Column::Right;
            }
        }
        TuiMessage::CancelMove => {
            model.selected.clear();
            model.mode = Mode::Normal;
            model.active_column = Column::Left;
        }
        TuiMessage::SwitchColumn => {
            if model.mode == Mode::Moving {
                model.active_column = match model.active_column {
                    Column::Left => Column::Right,
                    Column::Right => Column::Left,
                };
            }
        }
        TuiMessage::ExecuteMove => {
            if model.mode == Mode::Moving && model.active_column == Column::Right {
                execute_move(model);
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
                    if let Some(ref id) = node.alias_id {
                        delete_alias(model, id);
                        save_all(model);
                        model.rebuild_tree();
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
                    let _ = model.app_model.profile_config_mut().remove_profile(&name);
                    if model.app_model.config.active_profile.as_deref() == Some(name.as_str()) {
                        model.app_model.config.active_profile = None;
                    }
                    save_all(model);
                    model.rebuild_tree();
                }
                ConfirmAction::OverwriteAliases { aliases, destination } => {
                    do_move(model, &aliases, &destination);
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
                    TextInputState::NewProfile(buf) => { buf.push(c); }
                    TextInputState::NewAlias { name, command, active_field, .. } => {
                        match active_field {
                            AliasField::Name => { name.push(c); }
                            AliasField::Command => { command.push(c); }
                        }
                    }
                }
            }
        }
        TuiMessage::TextInputBackspace => {
            if let Mode::TextInput(ref mut state) = model.mode {
                match state {
                    TextInputState::NewProfile(buf) => { buf.pop(); }
                    TextInputState::NewAlias { name, command, active_field, .. } => {
                        match active_field {
                            AliasField::Name => { name.pop(); }
                            AliasField::Command => { command.pop(); }
                        }
                    }
                }
            }
        }
        TuiMessage::TextInputSwitchField => {
            if let Mode::TextInput(TextInputState::NewAlias { active_field, .. }) = &mut model.mode {
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
                    if model.app_model.profile_config().get_profile_by_name(&name).is_some() {
                        return;
                    }
                    let _ = model.app_model.profile_config_mut().add_profile(&name, &None);
                    save_all(model);
                    model.rebuild_tree();
                    model.mode = Mode::Normal;
                }
                TextInputState::NewAlias { name, command, target, .. } => {
                    let name = name.trim().to_string();
                    let command = command.trim().to_string();
                    if name.is_empty() || command.is_empty() {
                        return;
                    }
                    match &target {
                        AliasTarget::Global => {
                            model.app_model.config.add_alias(name, command, false);
                        }
                        AliasTarget::Profile(profile_name) => {
                            if let Some(p) = model.app_model.profile_config_mut().get_profile_by_name_mut(profile_name) {
                                let _ = p.add_alias(name, command, false);
                            }
                        }
                        AliasTarget::Project => {
                            let project = model.project_aliases.get_or_insert_with(ProjectAliases::default);
                            project.add_alias(name, command, false);
                            if model.project_path.is_none() {
                                model.project_path = Some(std::env::current_dir().unwrap_or_default().join(".aliases"));
                            }
                        }
                    }
                    save_all(model);
                    model.rebuild_tree();
                    model.mode = Mode::Normal;
                }
            }
        }
        TuiMessage::TextInputCancel => {
            model.mode = Mode::Normal;
        }
        TuiMessage::SetActive => {
            if model.mode != Mode::Normal {
                return;
            }
            let node = match model.tree.get(model.cursor) {
                Some(n) => n.clone(),
                None => return,
            };
            if node.kind == NodeKind::ProfileHeader {
                model.app_model.config.active_profile = Some(node.label.clone());
                save_all(model);
                model.rebuild_tree();
            }
        }
        _ => {} // remaining messages (Quit, Resize) handled at the app layer
    }
}

fn execute_move(model: &mut TuiModel) {
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

    // Filter out aliases that are already at the destination (same-source moves).
    let aliases_to_move: Vec<AliasId> = model
        .selected
        .iter()
        .filter(|id| !is_same_source(id, &destination))
        .cloned()
        .collect();

    if aliases_to_move.is_empty() {
        // All selected aliases are already at the destination — treat as no-op.
        model.selected.clear();
        model.mode = Mode::Normal;
        model.active_column = Column::Left;
        return;
    }

    // Check for collisions: aliases that already exist at the destination.
    let collisions: Vec<AliasId> = aliases_to_move
        .iter()
        .filter(|id| alias_exists_at_dest(model, id, &destination))
        .cloned()
        .collect();

    if collisions.is_empty() {
        do_move(model, &aliases_to_move, &destination);
    } else {
        model.mode = Mode::Confirm(ConfirmAction::OverwriteAliases {
            aliases: aliases_to_move,
            destination,
        });
    }
}

fn delete_alias(model: &mut TuiModel, alias_id: &AliasId) {
    match alias_id {
        AliasId::Global { alias_name } => {
            let _ = model.app_model.config.remove_alias(alias_name);
        }
        AliasId::Profile { profile_name, alias_name } => {
            if let Some(p) = model.app_model.profile_config_mut().get_profile_by_name_mut(profile_name) {
                let _ = p.remove_alias(alias_name);
            }
        }
        AliasId::Project { alias_name } => {
            if let Some(ref mut p) = model.project_aliases {
                let key = amoxide::AliasName::from(alias_name.as_str());
                p.aliases.remove(&key);
            }
        }
    }
    model.selected.remove(alias_id);
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
            .project_aliases
            .as_ref()
            .is_some_and(|p| p.aliases.contains_key(&key)),
        MoveDestination::Profile(name) => model
            .app_model
            .profile_config()
            .get_profile_by_name(name)
            .is_some_and(|p| p.aliases.contains_key(&key)),
    }
}

fn do_move(model: &mut TuiModel, aliases: &[AliasId], dest: &MoveDestination) {
    for alias_id in aliases {
        move_single_alias(model, alias_id, dest);
    }
    save_all(model);
    model.selected.clear();
    model.mode = Mode::Normal;
    model.active_column = Column::Left;
    model.rebuild_tree();
}

fn move_single_alias(model: &mut TuiModel, alias_id: &AliasId, dest: &MoveDestination) {
    // Read the alias from its source.
    let (alias_name_str, toml_alias) = match alias_id {
        AliasId::Global { alias_name } => {
            let key = AliasName::from(alias_name.as_str());
            let alias = model.app_model.config.aliases.get(&key).cloned();
            (alias_name.clone(), alias)
        }
        AliasId::Profile {
            profile_name,
            alias_name,
        } => {
            let key = AliasName::from(alias_name.as_str());
            let alias = model
                .app_model
                .profile_config()
                .get_profile_by_name(profile_name)
                .and_then(|p| p.aliases.get(&key).cloned());
            (alias_name.clone(), alias)
        }
        AliasId::Project { alias_name } => {
            let key = AliasName::from(alias_name.as_str());
            let alias = model
                .project_aliases
                .as_ref()
                .and_then(|p| p.aliases.get(&key).cloned());
            (alias_name.clone(), alias)
        }
    };

    let Some(toml_alias) = toml_alias else {
        return;
    };

    let command = toml_alias.command().to_string();
    let raw = matches!(&toml_alias, TomlAlias::Detailed(d) if d.raw);

    // Remove from source.
    match alias_id {
        AliasId::Global { alias_name } => {
            let _ = model.app_model.config.remove_alias(alias_name);
        }
        AliasId::Profile {
            profile_name,
            alias_name,
        } => {
            if let Some(profile) = model
                .app_model
                .profile_config_mut()
                .get_profile_by_name_mut(profile_name)
            {
                let _ = profile.remove_alias(alias_name);
            }
        }
        AliasId::Project { alias_name } => {
            if let Some(proj) = model.project_aliases.as_mut() {
                let key = AliasName::from(alias_name.as_str());
                proj.aliases.remove(&key);
            }
        }
    }

    // Add to destination.
    match dest {
        MoveDestination::Global => {
            model
                .app_model
                .config
                .add_alias(alias_name_str, command, raw);
        }
        MoveDestination::Project => {
            if let Some(proj) = model.project_aliases.as_mut() {
                proj.add_alias(alias_name_str, command, raw);
            } else {
                // Create project aliases if they don't exist yet.
                let mut proj = ProjectAliases::default();
                proj.add_alias(alias_name_str, command, raw);
                model.project_aliases = Some(proj);
            }
        }
        MoveDestination::Profile(profile_name) => {
            if let Some(profile) = model
                .app_model
                .profile_config_mut()
                .get_profile_by_name_mut(profile_name)
            {
                let _ = profile.add_alias(alias_name_str, command, raw);
            }
        }
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
                Some(AliasId::Profile { profile_name, .. }) => Some(AliasTarget::Profile(profile_name.clone())),
                Some(AliasId::Project { .. }) => Some(AliasTarget::Project),
                None => None,
            }
        }
    }
}

fn save_all(model: &TuiModel) {
    if let Some(ref dir) = model.config_dir {
        let _ = model.app_model.config.save_to(dir);
        let _ = model.app_model.profile_config().save_to(dir);
    } else {
        let _ = model.app_model.config.save();
        let _ = model.app_model.profile_config().save();
    }
    if let (Some(proj), Some(path)) = (&model.project_aliases, &model.project_path) {
        let _ = proj.save(path);
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
    use amoxide::{Config, ProfileConfig};
    use amoxide::update::AppModel;
    use std::collections::BTreeSet;

    fn test_model(profiles_toml: &str) -> TuiModel {
        let profiles: ProfileConfig = toml::from_str(profiles_toml).unwrap();
        let config = Config::default();
        let app_model = AppModel::new(config, profiles);
        let tree = build_tree_from_parts(
            &app_model.config.aliases,
            app_model.profile_config(),
            None,
            None,
        );
        let dest_tree = build_dest_tree_from_parts(
            &app_model.config.aliases,
            app_model.profile_config(),
            None,
            false,
        );
        // Use a temp dir so save_all() never touches real config files
        let temp_dir = std::env::temp_dir().join(format!("am-tui-test-{}", std::process::id()));
        TuiModel {
            app_model,
            project_aliases: None,
            project_path: None,
            config_dir: Some(temp_dir),
            tree,
            cursor: 0,
            selected: BTreeSet::new(),
            mode: Mode::Normal,
            dest_tree,
            dest_cursor: 0,
            active_column: Column::Left,
            scroll_offset: 0,
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
        let alias_idx = model.tree.iter().position(|n| n.kind == NodeKind::AliasItem).unwrap();
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
        let alias_idx = model.tree.iter().position(|n| n.kind == NodeKind::AliasItem).unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        update(&mut model, TuiMessage::EnterMoveMode);
        assert_eq!(model.mode, Mode::Moving);
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
        let alias_idx = model.tree.iter().position(|n| n.kind == NodeKind::AliasItem).unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        update(&mut model, TuiMessage::EnterMoveMode);
        update(&mut model, TuiMessage::CancelMove);
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
        let alias_idx = model.tree.iter().position(|n| n.kind == NodeKind::AliasItem).unwrap();
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
        let alias_idx = model.tree.iter().position(|n| n.kind == NodeKind::AliasItem).unwrap();
        model.cursor = alias_idx;
        update(&mut model, TuiMessage::ToggleSelect);
        update(&mut model, TuiMessage::EnterMoveMode);
        let git_idx = model
            .dest_tree
            .iter()
            .position(|n| n.label == "git")
            .unwrap();
        model.dest_cursor = git_idx;
        update(&mut model, TuiMessage::ExecuteMove);
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
        assert_eq!(model.mode, Mode::TextInput(TextInputState::NewProfile(String::new())));
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
        assert_eq!(model.mode, Mode::TextInput(TextInputState::NewProfile(String::new())));
        assert_eq!(model.app_model.profile_config().len(), 0);
    }

    #[test]
    fn test_set_active_profile() {
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
        update(&mut model, TuiMessage::SetActive);
        assert_eq!(model.app_model.config.active_profile.as_deref(), Some("rust"));
    }
}
