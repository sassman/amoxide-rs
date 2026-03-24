use crate::model::{Column, Mode, TreeNode, TuiMessage, TuiModel};

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
            if let Some(node) = model.tree.get(model.cursor) {
                if node.kind.is_selectable() {
                    if let Some(ref id) = node.alias_id {
                        if !model.selected.remove(id) {
                            model.selected.insert(id.clone());
                        }
                    }
                }
            }
        }
        _ => {} // other messages handled in later tasks
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
    use am::{Config, ProfileConfig};
    use am::update::AppModel;
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
        TuiModel {
            app_model,
            project_aliases: None,
            project_path: None,
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
        model.cursor = 1;
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
}
