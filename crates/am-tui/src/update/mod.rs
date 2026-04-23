pub mod delegation;
mod navigation;
mod profile_actions;
mod selection;
mod text_input;
mod transfer;
mod trust;

use crate::model::{TuiMessage, TuiModel};

pub fn update(model: &mut TuiModel, msg: TuiMessage) {
    // Clear status line on each keypress
    model.status_line = None;

    match msg {
        TuiMessage::CursorUp
        | TuiMessage::CursorDown
        | TuiMessage::JumpTop
        | TuiMessage::JumpBottom
        | TuiMessage::SwitchColumn => navigation::handle(model, msg),

        TuiMessage::ToggleSelect => selection::handle_toggle(model),

        TuiMessage::StartCreateProfile
        | TuiMessage::StartAddAlias
        | TuiMessage::EditItem
        | TuiMessage::TextInputChar(_)
        | TuiMessage::TextInputBackspace
        | TuiMessage::TextInputConfirm
        | TuiMessage::TextInputCancel
        | TuiMessage::TextInputSwitchField
        | TuiMessage::TextInputSwitchFieldBack
        | TuiMessage::TextInputCursorLeft
        | TuiMessage::TextInputCursorRight => text_input::handle(model, msg),

        TuiMessage::EnterMoveMode
        | TuiMessage::EnterCopyMode
        | TuiMessage::ExecuteTransfer
        | TuiMessage::CancelTransfer => transfer::handle(model, msg),

        TuiMessage::DeleteItem
        | TuiMessage::ConfirmYes
        | TuiMessage::ConfirmNo
        | TuiMessage::UseProfile
        | TuiMessage::UseProfileWithPriority(_) => profile_actions::handle(model, msg),

        TuiMessage::ToggleTrust => trust::handle(model),

        TuiMessage::Quit | TuiMessage::Resize(_, _) => {} // handled at app layer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        AliasField, AliasId, AliasTarget, ConfirmAction, Mode, NodeKind, SubcommandField,
        TextInputState, TransferMode, TuiMessage, TuiModel,
    };
    use crate::tree::{build_dest_tree_from_parts, build_tree_from_parts};
    use amoxide::update::AppModel;
    use amoxide::{AliasName, Config, ProfileConfig};
    use std::collections::BTreeSet;

    fn test_model(profiles_toml: &str) -> TuiModel {
        let profiles: ProfileConfig = toml::from_str(profiles_toml).unwrap();
        let config = Config::default();
        let app_model = AppModel::new(config, profiles);
        let tree = build_tree_from_parts(
            &app_model.config.aliases,
            &app_model.config.subcommands,
            app_model.profile_config(),
            &app_model.session.active_profiles,
            None,
            None,
        );
        let dest_tree = build_dest_tree_from_parts(
            &app_model.config.aliases,
            app_model.profile_config(),
            &app_model.session.active_profiles,
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
            active_column: crate::model::Column::Left,
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
        assert_eq!(model.active_column, crate::model::Column::Right);
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
        assert_eq!(model.active_column, crate::model::Column::Left);
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
        assert_eq!(model.active_column, crate::model::Column::Right);
        update(&mut model, TuiMessage::SwitchColumn);
        assert_eq!(model.active_column, crate::model::Column::Left);
        update(&mut model, TuiMessage::SwitchColumn);
        assert_eq!(model.active_column, crate::model::Column::Right);
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
        assert_eq!(model.active_column, crate::model::Column::Left);
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
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("git")
            .is_some());
    }

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
        update(&mut model, TuiMessage::TextInputConfirm);
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
        assert!(model.app_model.session.is_active("rust"));
        let rust_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "rust")
            .unwrap();
        model.cursor = rust_idx;
        update(&mut model, TuiMessage::UseProfile);
        assert!(!model.app_model.session.is_active("rust"));
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
        let git_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "git")
            .unwrap();
        model.cursor = git_idx;
        update(&mut model, TuiMessage::UseProfile);
        assert!(model.app_model.session.is_active("git"));

        let rust_idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::ProfileHeader && n.label == "rust")
            .unwrap();
        model.cursor = rust_idx;
        update(&mut model, TuiMessage::UseProfileWithPriority(1));
        assert_eq!(
            model.app_model.session.active_profiles,
            vec!["rust".to_string(), "git".to_string()]
        );
    }

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
        assert_eq!(model.active_column, crate::model::Column::Right);
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
        assert!(model
            .app_model
            .profile_config()
            .get_profile_by_name("git")
            .unwrap()
            .aliases
            .iter()
            .any(|(n, _)| n.as_ref() == "gs"));
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

    #[test]
    fn test_start_add_alias_on_subcommand_program_header_opens_subcmd_editor() {
        let mut model = test_model("profiles = []");
        model
            .app_model
            .config
            .subcommands
            .as_mut()
            .insert("jj:ab".into(), vec!["abandon".into()]);
        model.rebuild_tree();
        let idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::SubcommandProgramHeader)
            .unwrap();
        model.cursor = idx;
        update(&mut model, TuiMessage::StartAddAlias);
        assert!(
            matches!(
                model.mode,
                Mode::TextInput(TextInputState::SubcommandInput { .. })
            ),
            "expected SubcommandInput mode, got {:?}",
            model.mode
        );
    }

    #[test]
    fn test_new_alias_tab_with_colon_morphs_to_subcommand_editor() {
        let mut model = test_model("profiles = []");
        model.mode = Mode::TextInput(TextInputState::NewAlias {
            name: "git:".into(),
            command: String::new(),
            active_field: AliasField::Name,
            cursor: 4,
            target: AliasTarget::Global,
        });
        update(&mut model, TuiMessage::TextInputSwitchField);
        match &model.mode {
            Mode::TextInput(TextInputState::SubcommandInput { program, pairs, .. }) => {
                assert_eq!(program, "git");
                assert_eq!(pairs.len(), 1);
                assert_eq!(pairs[0].0, ""); // short token is empty (nothing after the colon)
            }
            other => panic!("expected SubcommandInput, got {other:?}"),
        }
    }

    #[test]
    fn test_new_alias_tab_with_colon_and_short_token() {
        let mut model = test_model("profiles = []");
        model.mode = Mode::TextInput(TextInputState::NewAlias {
            name: "git:co".into(),
            command: String::new(),
            active_field: AliasField::Name,
            cursor: 6,
            target: AliasTarget::Global,
        });
        update(&mut model, TuiMessage::TextInputSwitchField);
        match &model.mode {
            Mode::TextInput(TextInputState::SubcommandInput {
                program,
                pairs,
                cursor,
                ..
            }) => {
                assert_eq!(program, "git");
                assert_eq!(pairs[0].0, "co");
                assert_eq!(*cursor, "co".len());
            }
            other => panic!("expected SubcommandInput, got {other:?}"),
        }
    }

    #[test]
    fn test_new_alias_tab_without_colon_switches_to_command_field() {
        let mut model = test_model("profiles = []");
        model.mode = Mode::TextInput(TextInputState::NewAlias {
            name: "ll".into(),
            command: String::new(),
            active_field: AliasField::Name,
            cursor: 2,
            target: AliasTarget::Global,
        });
        update(&mut model, TuiMessage::TextInputSwitchField);
        match &model.mode {
            Mode::TextInput(TextInputState::NewAlias { active_field, .. }) => {
                assert_eq!(*active_field, AliasField::Command);
            }
            other => panic!("expected NewAlias on Command field, got {other:?}"),
        }
    }

    #[test]
    fn test_subcmd_add_pair_extends_pairs() {
        // Tab on the Long field of a complete pair should add a new pair.
        let mut model = test_model("profiles = []");
        model.mode = Mode::TextInput(TextInputState::SubcommandInput {
            program: "jj".into(),
            pairs: vec![("ab".into(), "abandon".into())],
            active_pair: 0,
            active_field: SubcommandField::Long,
            cursor: "abandon".len(),
            target: AliasTarget::Global,
            original_key: None,
        });
        update(&mut model, TuiMessage::TextInputSwitchField);
        match &model.mode {
            Mode::TextInput(TextInputState::SubcommandInput {
                pairs,
                active_pair,
                active_field,
                ..
            }) => {
                assert_eq!(pairs.len(), 2);
                assert_eq!(*active_pair, 1);
                assert_eq!(*active_field, SubcommandField::Short);
            }
            other => panic!("expected SubcommandInput, got {other:?}"),
        }
    }

    #[test]
    fn test_tab_on_empty_long_does_not_add_pair() {
        // Tab on an empty Long field should not add a new pair.
        let mut model = test_model("profiles = []");
        model.mode = Mode::TextInput(TextInputState::SubcommandInput {
            program: "jj".into(),
            pairs: vec![("ab".into(), "".into())],
            active_pair: 0,
            active_field: SubcommandField::Long,
            cursor: 0,
            target: AliasTarget::Global,
            original_key: None,
        });
        update(&mut model, TuiMessage::TextInputSwitchField);
        match &model.mode {
            Mode::TextInput(TextInputState::SubcommandInput {
                pairs, active_pair, ..
            }) => {
                assert_eq!(pairs.len(), 1, "no new pair should be added");
                assert_eq!(*active_pair, 0);
            }
            other => panic!("expected SubcommandInput, got {other:?}"),
        }
    }

    #[test]
    fn test_esc_removes_empty_last_pair() {
        // Esc when the last pair is empty should remove it, not exit.
        let mut model = test_model("profiles = []");
        model.mode = Mode::TextInput(TextInputState::SubcommandInput {
            program: "jj".into(),
            pairs: vec![("ab".into(), "abandon".into()), ("".into(), "".into())],
            active_pair: 1,
            active_field: SubcommandField::Short,
            cursor: 0,
            target: AliasTarget::Global,
            original_key: None,
        });
        update(&mut model, TuiMessage::TextInputCancel);
        match &model.mode {
            Mode::TextInput(TextInputState::SubcommandInput {
                pairs, active_pair, ..
            }) => {
                assert_eq!(pairs.len(), 1, "empty last pair should be removed");
                assert_eq!(*active_pair, 0);
            }
            other => panic!("expected SubcommandInput after first Esc, got {other:?}"),
        }
        // Second Esc exits
        update(&mut model, TuiMessage::TextInputCancel);
        assert_eq!(model.mode, Mode::Normal);
    }

    #[test]
    fn test_subcmd_confirm_dispatches_add() {
        let mut model = test_model("profiles = []");
        model.mode = Mode::TextInput(TextInputState::SubcommandInput {
            program: "jj".into(),
            pairs: vec![("ab".into(), "abandon".into())],
            active_pair: 0,
            active_field: SubcommandField::Long,
            cursor: "abandon".len(),
            target: AliasTarget::Global,
            original_key: None,
        });
        update(&mut model, TuiMessage::TextInputConfirm);
        assert_eq!(model.mode, Mode::Normal);
        assert!(model
            .app_model
            .config
            .subcommands
            .as_ref()
            .contains_key("jj:ab"));
    }

    fn make_subcmd_model(keys: &[(&str, &[&str])]) -> TuiModel {
        let mut config = amoxide::Config::default();
        for (key, longs) in keys {
            config.subcommands.as_mut().insert(
                key.to_string(),
                longs.iter().map(|s| s.to_string()).collect(),
            );
        }
        let app_model = AppModel::new(config, ProfileConfig::default());
        let tree = build_tree_from_parts(
            &app_model.config.aliases,
            &app_model.config.subcommands,
            app_model.profile_config(),
            &app_model.session.active_profiles,
            None,
            None,
        );
        let dest_tree = build_dest_tree_from_parts(
            &app_model.config.aliases,
            app_model.profile_config(),
            &app_model.session.active_profiles,
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
            active_column: crate::model::Column::Left,
            scroll_offset: 0,
            status_line: None,
        }
    }

    #[test]
    fn test_delete_subcommand_item() {
        let mut model = make_subcmd_model(&[("jj:ab", &["abandon"])]);
        let idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::SubcommandItem)
            .unwrap();
        model.cursor = idx;
        update(&mut model, TuiMessage::DeleteItem);
        assert!(!model
            .app_model
            .config
            .subcommands
            .as_ref()
            .contains_key("jj:ab"));
    }

    #[test]
    fn test_delete_subcommand_program_header_removes_all_keys() {
        let mut model =
            make_subcmd_model(&[("jj:ab", &["abandon"]), ("jj:b:l", &["branch", "list"])]);
        let idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::SubcommandProgramHeader)
            .unwrap();
        model.cursor = idx;
        update(&mut model, TuiMessage::DeleteItem);
        assert!(model.app_model.config.subcommands.is_empty());
    }

    #[test]
    fn test_toggle_select_on_subcommand_item_adds_to_selected() {
        let mut model = make_subcmd_model(&[("jj:ab", &["abandon"])]);
        let idx = model
            .tree
            .iter()
            .position(|n| n.kind == NodeKind::SubcommandItem)
            .unwrap();
        model.cursor = idx;
        update(&mut model, TuiMessage::ToggleSelect);
        assert_eq!(model.selected.len(), 1);
        let id = model.selected.iter().next().unwrap();
        assert!(matches!(id, AliasId::Subcommand { .. }));
    }
}
