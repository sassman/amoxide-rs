use crate::config::LogVerbosity;
use crate::sync_outcome::SyncOutcome;

#[derive(Debug, Clone, PartialEq)]
pub enum Echo {
    Silent,
    Line(String),
}

impl Echo {
    /// Choose output based on verbosity. Closures are lazy — no work done for suppressed tiers.
    pub fn from_verbosity(
        verbosity: &LogVerbosity,
        short: impl FnOnce() -> String,
        verbose: impl FnOnce() -> String,
    ) -> Self {
        match verbosity {
            LogVerbosity::Off => Self::Silent,
            LogVerbosity::Short => Self::Line(short()),
            LogVerbosity::Verbose => Self::Line(verbose()),
        }
    }

    /// Functional shell output — always emitted regardless of verbosity.
    pub fn always(s: String) -> Self {
        if s.is_empty() {
            Self::Silent
        } else {
            Self::Line(s)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Effect {
    SaveConfig,
    SaveSession,
    SaveProfiles,
    AddLocalAlias {
        name: String,
        cmd: String,
        raw: bool,
    },
    RemoveLocalAlias {
        name: String,
    },
    AddLocalSubcommand {
        key: String,
        long_subcommands: Vec<String>,
    },
    RemoveLocalSubcommand {
        key: String,
    },
    Print(String),
    PrintLines(Vec<Echo>),
    RenderSync(SyncOutcome),
    SaveSecurity,
}

use crate::update::AppModel;

/// Execute a single I/O effect against the model.
/// Does NOT handle `Effect::Print` — callers render that in their own way.
pub fn execute_effect(model: &mut AppModel, effect: &Effect) -> anyhow::Result<()> {
    match effect {
        Effect::SaveConfig => model.save_config()?,
        Effect::SaveSession => model.save_session()?,
        Effect::SaveProfiles => model.save_profiles()?,
        Effect::SaveSecurity => model.save_security()?,
        Effect::AddLocalAlias { name, cmd, raw } => {
            model.save_project_aliases_add(name, cmd, *raw)?;
        }
        Effect::RemoveLocalAlias { name } => {
            model.save_project_aliases_remove(name)?;
        }
        Effect::AddLocalSubcommand {
            key,
            long_subcommands,
        } => {
            model.save_project_subcommand_add(key, long_subcommands)?;
        }
        Effect::RemoveLocalSubcommand { key } => {
            model.save_project_subcommand_remove(key)?;
        }
        Effect::Print(_) => {} // caller's responsibility
        Effect::PrintLines(_) => {} // caller's responsibility, like Print
        Effect::RenderSync(_) => {} // caller's responsibility
    }
    Ok(())
}

#[cfg(test)]
mod echo_tests {
    use super::*;
    use crate::config::LogVerbosity;

    #[test]
    fn echo_from_verbosity_off_returns_silent() {
        let echo = Echo::from_verbosity(
            &LogVerbosity::Off,
            || "short".to_string(),
            || "verbose".to_string(),
        );
        assert!(matches!(echo, Echo::Silent));
    }

    #[test]
    fn echo_from_verbosity_short_returns_short_line() {
        let echo = Echo::from_verbosity(
            &LogVerbosity::Short,
            || "short msg".to_string(),
            || panic!("verbose closure should not be called"),
        );
        assert!(matches!(echo, Echo::Line(s) if s == "short msg"));
    }

    #[test]
    fn echo_from_verbosity_verbose_returns_verbose_line() {
        let echo = Echo::from_verbosity(
            &LogVerbosity::Verbose,
            || panic!("short closure should not be called"),
            || "verbose msg".to_string(),
        );
        assert!(matches!(echo, Echo::Line(s) if s == "verbose msg"));
    }

    #[test]
    fn echo_always_empty_is_silent() {
        assert!(matches!(Echo::always(String::new()), Echo::Silent));
    }

    #[test]
    fn echo_always_non_empty_is_line() {
        assert!(matches!(Echo::always("hello".into()), Echo::Line(s) if s == "hello"));
    }
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
