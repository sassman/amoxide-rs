use amoxide::effects::Effect;
use amoxide::update::UpdateError;
use amoxide::Message;

use crate::model::TuiModel;

/// Execute a library message against the model.
/// On success, executes all effects and rebuilds the tree.
/// On `UpdateError`, stores a TUI-friendly message in `status_line` and returns Ok(()).
pub fn dispatch(model: &mut TuiModel, msg: Message) -> anyhow::Result<()> {
    let result = match amoxide::update::update(&mut model.app_model, msg) {
        Ok(r) => r,
        Err(e) => {
            model.status_line = Some(tui_error_message(&e));
            return Ok(());
        }
    };

    execute_effects(model, &result.effects)?;

    if let Some(follow_up) = result.next {
        return dispatch(model, follow_up);
    }

    model.rebuild_tree();
    Ok(())
}

fn execute_effects(model: &mut TuiModel, effects: &[Effect]) -> anyhow::Result<()> {
    for effect in effects {
        match effect {
            Effect::Print(text) => {
                model.status_line = Some(text.clone());
            }
            other => {
                amoxide::execute_effect(&mut model.app_model, other)?;
            }
        }
    }
    Ok(())
}

fn tui_error_message(err: &UpdateError) -> String {
    match err {
        UpdateError::ProjectNotTrusted { .. } => {
            "press 't' to trust this project".to_string()
        }
        UpdateError::AliasNotFound { name, .. } => format!("alias '{name}' not found"),
        UpdateError::ProfileNotFound { name } => format!("profile '{name}' not found"),
        UpdateError::NoProjectFile => "no .aliases file in this tree".to_string(),
        UpdateError::Other(e) => format!("error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amoxide::{AliasTarget, Config, Message, ProfileConfig};
    use amoxide::update::AppModel;
    use crate::model::{Column, Mode, TuiModel};
    use std::collections::BTreeSet;

    fn make_model() -> TuiModel {
        let app_model = AppModel::new(Config::default(), ProfileConfig::default());
        let tree = crate::tree::build_tree(&app_model);
        let dest_tree = crate::tree::build_dest_tree(&app_model);
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
    fn dispatch_successful_mutation_rebuilds_tree() {
        let mut model = make_model();
        dispatch(
            &mut model,
            Message::AddAlias("ll".into(), "ls -lha".into(), AliasTarget::Global, false),
        )
        .unwrap();

        assert!(!model.tree.is_empty());
        assert!(model.status_line.is_none());
    }

    #[test]
    fn dispatch_project_not_trusted_sets_status_line() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        std::fs::write(&aliases_path, "[aliases]\nt = \"cargo test\"\n").unwrap();
        let mut security = amoxide::security::SecurityConfig::default();
        security.untrust(&aliases_path);
        let app_model = AppModel::new_with_security(
            Config::default(),
            ProfileConfig::default(),
            security,
        )
        .with_cwd(dir.path().to_path_buf());
        let tree = crate::tree::build_tree(&app_model);
        let dest_tree = crate::tree::build_dest_tree(&app_model);
        let mut model = TuiModel {
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
        };

        dispatch(
            &mut model,
            Message::AddAlias("t".into(), "cargo test".into(), AliasTarget::Local, false),
        )
        .unwrap();

        assert!(model.status_line.is_some());
        let msg = model.status_line.as_ref().unwrap();
        assert!(msg.contains("trust"), "expected trust message, got: {msg}");
    }

    #[test]
    fn dispatch_print_effect_goes_to_status_line() {
        let mut model = make_model();
        // CreateProfile returns no Print effect, just SaveProfiles — but ToggleProfiles emits Print
        let profile_config: amoxide::ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"git\"\n").unwrap();
        let app_model = AppModel::new(Config::default(), profile_config);
        let tree = crate::tree::build_tree(&app_model);
        let dest_tree = crate::tree::build_dest_tree(&app_model);
        model.app_model = app_model;
        model.tree = tree;
        model.dest_tree = dest_tree;

        dispatch(
            &mut model,
            Message::ToggleProfiles(vec!["git".into()]),
        )
        .unwrap();

        // ToggleProfiles emits a Print effect with activation message
        assert!(model.status_line.is_some());
        let msg = model.status_line.as_ref().unwrap();
        assert!(msg.contains("git"), "expected profile name in status, got: {msg}");
    }
}
