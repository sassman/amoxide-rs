#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    SaveConfig,
    SaveProfiles,
    AddLocalAlias {
        name: String,
        cmd: String,
        raw: bool,
    },
    RemoveLocalAlias {
        name: String,
    },
    Print(String),
    SaveSecurity,
}

use crate::update::AppModel;

/// Execute a single I/O effect against the model.
/// Does NOT handle `Effect::Print` — callers render that in their own way.
pub fn execute_effect(model: &mut AppModel, effect: &Effect) -> anyhow::Result<()> {
    match effect {
        Effect::SaveConfig => model.save_config()?,
        Effect::SaveProfiles => model.save_profiles()?,
        Effect::SaveSecurity => model.save_security()?,
        Effect::AddLocalAlias { name, cmd, raw } => {
            model.save_project_aliases_add(name, cmd, *raw)?;
        }
        Effect::RemoveLocalAlias { name } => {
            model.save_project_aliases_remove(name)?;
        }
        Effect::Print(_) => {} // caller's responsibility
    }
    Ok(())
}

#[cfg(all(test, feature = "test-util"))]
mod tests {
    use super::*;
    use crate::update::AppModel;

    #[test]
    fn execute_effect_save_config_writes_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let mut model = AppModel::load_from(dir.path().to_path_buf());
        model.config.add_alias("ll".into(), "ls -lha".into(), false);
        execute_effect(&mut model, &Effect::SaveConfig).unwrap();

        let saved = crate::config::Config::load_from(dir.path()).unwrap();
        assert_eq!(saved.aliases.iter().count(), 1);
    }

    #[test]
    fn execute_effect_print_is_noop() {
        let dir = tempfile::tempdir().unwrap();
        let mut model = AppModel::load_from(dir.path().to_path_buf());
        // Should not panic or error
        execute_effect(&mut model, &Effect::Print("hello".into())).unwrap();
    }

    #[test]
    fn execute_effect_add_local_alias_updates_project_trust() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        std::fs::write(&aliases_path, "[aliases]\n").unwrap();

        // Trust the file first
        let mut sec = crate::security::SecurityConfig::default();
        let hash = crate::trust::compute_file_hash(&aliases_path).unwrap();
        sec.trust(&aliases_path, &hash);
        let mut model = AppModel::new_with_security(
            crate::Config::default(),
            crate::ProfileConfig::default(),
            sec,
        )
        .with_cwd(dir.path().to_path_buf());

        execute_effect(
            &mut model,
            &Effect::AddLocalAlias {
                name: "t".into(),
                cmd: "cargo test".into(),
                raw: false,
            },
        )
        .unwrap();

        let key = crate::AliasName::from("t");
        assert!(model.project_aliases().unwrap().aliases.contains_key(&key));
    }
}
