use std::path::{Path, PathBuf};

use anyhow::anyhow;

use crate::config::Config;
use crate::display::render_listing;
use crate::effects::Effect;
use crate::hook::generate_hook;
use crate::init::{generate_init, generate_reload};
use crate::project::ProjectAliases;
use crate::{profile, AliasSet, AliasTarget, Message, Profile, ProfileConfig};

pub struct UpdateResult {
    pub next: Option<Message>,
    pub effects: Vec<Effect>,
}

impl UpdateResult {
    pub fn new(message: Message, effects: &[Effect]) -> Self {
        Self {
            next: Some(message),
            effects: effects.to_vec(),
        }
    }

    pub fn with_effects(effects: &[Effect]) -> Self {
        Self {
            next: None,
            effects: effects.to_vec(),
        }
    }

    pub fn message(message: Message) -> Self {
        Self {
            next: Some(message),
            effects: vec![],
        }
    }

    pub fn effect(effect: Effect) -> Self {
        Self {
            next: None,
            effects: vec![effect],
        }
    }

    pub fn done() -> Self {
        Self {
            next: None,
            effects: vec![],
        }
    }
}

pub struct AppModel {
    pub config: Config,
    pub cwd: std::path::PathBuf,
    profile_config: ProfileConfig,
    project_aliases: Option<ProjectAliases>,
    project_path: Option<PathBuf>,
}

impl Default for AppModel {
    fn default() -> Self {
        let profile_config = ProfileConfig::load().unwrap();
        let config = Config::load().unwrap_or_default();
        let cwd = std::env::current_dir().unwrap_or_default();
        let project_path = ProjectAliases::find_path(&cwd).ok().flatten();
        let project_aliases = project_path
            .as_ref()
            .and_then(|p| ProjectAliases::load(p).ok());

        Self {
            config,
            cwd: std::env::current_dir().unwrap_or_default(),
            profile_config,
            project_aliases,
            project_path,
        }
    }
}

impl AppModel {
    pub fn new(config: Config, profile_config: ProfileConfig) -> Self {
        Self {
            config,
            cwd: std::env::current_dir().unwrap_or_default(),
            profile_config,
            project_aliases: None,
            project_path: None,
        }
    }

    pub fn with_cwd(mut self, cwd: std::path::PathBuf) -> Self {
        // Re-discover project aliases for the new cwd
        self.project_path = ProjectAliases::find_path(&cwd).ok().flatten();
        self.project_aliases = self
            .project_path
            .as_ref()
            .and_then(|p| ProjectAliases::load(p).ok());
        self.cwd = cwd;
        self
    }

    pub fn project_aliases(&self) -> Option<&ProjectAliases> {
        self.project_aliases.as_ref()
    }

    pub fn project_path(&self) -> Option<&Path> {
        self.project_path.as_deref()
    }

    /// Get project aliases' AliasSet, or empty default
    pub fn project_alias_set(&self) -> AliasSet {
        self.project_aliases
            .as_ref()
            .map(|p| p.aliases.clone())
            .unwrap_or_default()
    }

    /// Get or create the project path (for saving new .aliases files).
    /// If no .aliases exists, returns cwd/.aliases
    pub fn project_path_or_create(&self) -> PathBuf {
        self.project_path
            .clone()
            .unwrap_or_else(|| self.cwd.join(crate::project::ALIASES_FILE))
    }

    /// Merge aliases into project aliases and save.
    pub fn save_project_aliases(&mut self, aliases: AliasSet) -> anyhow::Result<()> {
        let path = self.project_path_or_create();
        let mut project = self.project_aliases.take().unwrap_or_default();
        project.merge_aliases(aliases);
        project.save(&path)?;
        self.project_aliases = Some(project);
        self.project_path = Some(path);
        Ok(())
    }

    pub fn profile_config_mut(&mut self) -> &mut ProfileConfig {
        &mut self.profile_config
    }

    pub fn profile_config(&self) -> &ProfileConfig {
        &self.profile_config
    }

    pub fn get_active_profiles(&self) -> Vec<&Profile> {
        self.config
            .active_profiles
            .iter()
            .filter_map(|name| self.profile_config.get_profile_by_name(name))
            .collect()
    }
}

pub fn update(model: &mut AppModel, message: Message) -> anyhow::Result<UpdateResult> {
    match message {
        Message::AddAlias(name, cmd, target, raw) => match target {
            AliasTarget::Global => {
                model.config.add_alias(name, cmd, raw);
                Ok(UpdateResult::effect(Effect::SaveConfig))
            }
            AliasTarget::Local => Ok(UpdateResult::effect(Effect::AddLocalAlias {
                name,
                cmd,
                raw,
            })),
            AliasTarget::ActiveProfile if model.config.active_profiles.is_empty() => {
                if model.project_path().is_some() {
                    Ok(UpdateResult::effect(Effect::AddLocalAlias {
                        name,
                        cmd,
                        raw,
                    }))
                } else {
                    model.config.add_alias(name, cmd, raw);
                    Ok(UpdateResult::effect(Effect::SaveConfig))
                }
            }
            target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                let profile = resolve_profile_mut(model, &target)?;
                profile.add_alias(name, cmd, raw)?;
                Ok(UpdateResult::effect(Effect::SaveProfiles))
            }
        },
        Message::RemoveAlias(name, target) => match target {
            AliasTarget::Global => {
                model.config.remove_alias(&name)?;
                Ok(UpdateResult::effect(Effect::SaveConfig))
            }
            AliasTarget::Local => Ok(UpdateResult::effect(Effect::RemoveLocalAlias { name })),
            AliasTarget::ActiveProfile if model.config.active_profiles.is_empty() => {
                if model.project_path().is_some() {
                    Ok(UpdateResult::effect(Effect::RemoveLocalAlias { name }))
                } else {
                    model.config.remove_alias(&name)?;
                    Ok(UpdateResult::effect(Effect::SaveConfig))
                }
            }
            target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                let profile = resolve_profile_mut(model, &target)?;
                profile.remove_alias(&name)?;
                Ok(UpdateResult::effect(Effect::SaveProfiles))
            }
        },
        Message::ListProfiles => {
            let rel_path = model
                .project_path()
                .map(|p| crate::dirs::relative_path(&model.cwd, p));
            let project = model
                .project_aliases()
                .filter(|p| !p.aliases.is_empty())
                .zip(rel_path.as_deref());
            let output = render_listing(
                &model.config.aliases,
                model.profile_config(),
                &model.config.active_profiles,
                project,
            );
            println!("{output}");
            Ok(UpdateResult::done())
        }
        Message::CreateProfile(name) => match model.profile_config_mut().add_profile(&name)? {
            profile::Response::ProfileAdded(_) => {
                if !model.config.active_profiles.contains(&name) {
                    model.config.active_profiles.push(name);
                }
                Ok(UpdateResult::effect(Effect::SaveProfiles))
            }
            profile::Response::ProfileActivated(_) => {
                if !model.config.active_profiles.contains(&name) {
                    model.config.active_profiles.push(name);
                }
                Ok(UpdateResult::done())
            }
        },
        Message::InitShell(shell) => {
            let resolved = model
                .profile_config()
                .resolve_active_aliases(&model.config.active_profiles);
            let output = generate_init(&shell, &model.config.aliases, &resolved);
            print!("{output}");
            Ok(UpdateResult::done())
        }
        Message::Reload(shell) => {
            let resolved = model
                .profile_config()
                .resolve_active_aliases(&model.config.active_profiles);
            let prev = std::env::var("_AM_ALIASES").ok();
            let output = generate_reload(&shell, &model.config.aliases, &resolved, prev.as_deref());
            if !output.is_empty() {
                print!("{output}");
            }
            Ok(UpdateResult::done())
        }
        Message::Hook(shell) => {
            let prev = std::env::var("_AM_PROJECT_ALIASES").ok();
            let output = generate_hook(&shell, &model.cwd, prev.as_deref())?;
            if !output.is_empty() {
                print!("{output}");
            }
            Ok(UpdateResult::done())
        }
        Message::ToggleProfiles(names) => {
            for name in &names {
                model
                    .profile_config()
                    .get_profile_by_name(name)
                    .ok_or(anyhow!("Profile '{name}' does not exist."))?;
            }
            let mut effects = Vec::new();
            for name in names {
                let was_active = model.config.is_active(&name);
                let alias_count = model
                    .profile_config()
                    .get_profile_by_name(&name)
                    .map(|p| p.aliases.iter().count())
                    .unwrap_or(0);
                model.config.toggle_profile(name.clone());
                let (action, verb) = if was_active {
                    ("deactivated", "unloaded")
                } else {
                    ("activated", "loaded")
                };
                effects.push(Effect::Print(format!(
                    "{name} {action}, {alias_count} aliases {verb}"
                )));
            }
            Ok(UpdateResult::with_effects(&effects))
        }
        Message::UseProfilesAt(names, priority) => {
            for name in &names {
                model
                    .profile_config()
                    .get_profile_by_name(name)
                    .ok_or(anyhow!("Profile '{name}' does not exist."))?;
            }
            let mut effects = Vec::new();
            for (i, name) in names.into_iter().enumerate() {
                let alias_count = model
                    .profile_config()
                    .get_profile_by_name(&name)
                    .map(|p| p.aliases.iter().count())
                    .unwrap_or(0);
                model.config.use_profile_at(name.clone(), priority + i);
                effects.push(Effect::Print(format!(
                    "{name} activated at position {}, {alias_count} aliases loaded",
                    priority + i
                )));
            }
            Ok(UpdateResult::with_effects(&effects))
        }
        Message::RemoveProfile(name) => {
            model.profile_config_mut().remove_profile(&name)?;
            model.config.active_profiles.retain(|p| p != &name);
            Ok(UpdateResult::effect(Effect::SaveProfiles))
        }
        Message::Import(payload) => {
            let mut effects = Vec::new();
            if let Some(aliases) = payload.global_aliases {
                model.config.merge_aliases(aliases);
                effects.push(Effect::SaveConfig);
            }
            for profile in payload.profiles {
                model.profile_config_mut().merge_profile(profile);
                if !effects.contains(&Effect::SaveProfiles) {
                    effects.push(Effect::SaveProfiles);
                }
            }
            // local_aliases are saved by the CLI layer (needs file path)
            Ok(UpdateResult::with_effects(&effects))
        }
    }
}

fn resolve_profile_mut<'a>(
    model: &'a mut AppModel,
    target: &AliasTarget,
) -> anyhow::Result<&'a mut Profile> {
    match target {
        AliasTarget::Profile(profile_name) => model
            .profile_config_mut()
            .get_profile_by_name_mut(profile_name)
            .ok_or_else(|| anyhow!("Profile not found: {profile_name}")),
        AliasTarget::ActiveProfile => {
            let active = model
                .config
                .active_profiles
                .last()
                .cloned()
                .ok_or_else(|| anyhow!("No active profile. Use -p <profile> or -g for global."))?;
            model
                .profile_config_mut()
                .get_profile_by_name_mut(&active)
                .ok_or_else(|| anyhow!("Active profile not found: {active}"))
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::effects::Effect;

    #[test]
    fn update_result_done_has_no_message_or_effects() {
        let r = UpdateResult::done();
        assert!(r.next.is_none());
        assert!(r.effects.is_empty());
    }

    #[test]
    fn update_result_message_has_message_and_no_effects() {
        let r = UpdateResult::message(Message::ListProfiles);
        assert!(r.next.is_some());
        assert!(r.effects.is_empty());
    }

    #[test]
    fn update_result_effect_has_one_effect_and_no_message() {
        let r = UpdateResult::effect(Effect::SaveConfig);
        assert!(r.next.is_none());
        assert_eq!(r.effects.len(), 1);
    }

    #[test]
    fn update_result_new_has_message_and_effects() {
        let r = UpdateResult::new(
            Message::ListProfiles,
            &[Effect::SaveConfig, Effect::SaveProfiles],
        );
        assert!(r.next.is_some());
        assert_eq!(r.effects.len(), 2);
    }

    #[test]
    fn update_result_with_effects_has_effects_and_no_message() {
        let r = UpdateResult::with_effects(&[Effect::SaveConfig, Effect::SaveProfiles]);
        assert!(r.next.is_none());
        assert_eq!(r.effects.len(), 2);
    }

    #[test]
    fn add_global_alias_mutates_config_and_returns_save_config() {
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::AddAlias("ll".into(), "ls -lha".into(), AliasTarget::Global, false),
        )
        .unwrap();

        assert_eq!(model.config.aliases.iter().count(), 1);
        assert_eq!(result.effects, vec![Effect::SaveConfig]);
        assert!(result.next.is_none());
    }

    #[test]
    fn add_alias_active_profile_no_profiles_no_local_falls_back_to_global() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config).with_cwd(dir.path().to_path_buf());

        let result = update(
            &mut model,
            Message::AddAlias(
                "ll".into(),
                "ls -lha".into(),
                AliasTarget::ActiveProfile,
                false,
            ),
        )
        .unwrap();

        assert_eq!(model.config.aliases.iter().count(), 1);
        assert_eq!(result.effects, vec![Effect::SaveConfig]);
    }

    #[test]
    fn remove_global_alias_mutates_config_and_returns_save_config() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::RemoveAlias("ll".into(), AliasTarget::Global),
        )
        .unwrap();

        assert!(model.config.aliases.is_empty());
        assert_eq!(result.effects, vec![Effect::SaveConfig]);
    }

    #[test]
    fn remove_alias_active_profile_no_profiles_no_local_falls_back_to_global() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config).with_cwd(dir.path().to_path_buf());

        let result = update(
            &mut model,
            Message::RemoveAlias("ll".into(), AliasTarget::ActiveProfile),
        )
        .unwrap();

        assert!(model.config.aliases.is_empty());
        assert_eq!(result.effects, vec![Effect::SaveConfig]);
    }

    #[test]
    fn add_local_alias_returns_add_local_effect() {
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::AddAlias("t".into(), "cargo test".into(), AliasTarget::Local, false),
        )
        .unwrap();

        assert_eq!(
            result.effects,
            vec![Effect::AddLocalAlias {
                name: "t".into(),
                cmd: "cargo test".into(),
                raw: false,
            }]
        );
        assert!(model.config.aliases.is_empty());
    }

    #[test]
    fn remove_local_alias_returns_remove_local_effect() {
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::RemoveAlias("t".into(), AliasTarget::Local),
        )
        .unwrap();

        assert_eq!(
            result.effects,
            vec![Effect::RemoveLocalAlias { name: "t".into() }]
        );
    }

    #[test]
    fn toggle_multiple_profiles_activates_all_in_order() {
        let config = Config::default();
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"git\"\n\n[[profiles]]\nname = \"rust\"\n")
                .unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::ToggleProfiles(vec!["git".into(), "rust".into()]),
        )
        .unwrap();

        assert_eq!(result.effects.len(), 2);
        assert!(matches!(&result.effects[0], Effect::Print(s) if s.contains("git") && s.contains("activated")));
        assert_eq!(
            model.config.active_profiles,
            vec!["git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn toggle_multiple_profiles_fails_if_any_missing() {
        let config = Config::default();
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"git\"\n").unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::ToggleProfiles(vec!["git".into(), "nope".into()]),
        );

        let err = result.err().expect("should be an error");
        assert!(err.to_string().contains("nope"));
        // git should NOT have been toggled since validation failed
        assert!(model.config.active_profiles.is_empty());
    }

    #[test]
    fn use_profiles_at_inserts_sequentially() {
        let config = Config::default();
        let profile_config: ProfileConfig = toml::from_str(
            "[[profiles]]\nname = \"git\"\n\n[[profiles]]\nname = \"rust\"\n\n[[profiles]]\nname = \"node\"\n",
        )
        .unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::UseProfilesAt(vec!["git".into(), "rust".into()], 1),
        )
        .unwrap();

        assert_eq!(result.effects.len(), 2);
        assert!(matches!(&result.effects[0], Effect::Print(s) if s.contains("git") && s.contains("position 1")));
        assert!(matches!(&result.effects[1], Effect::Print(s) if s.contains("rust") && s.contains("position 2")));
        assert_eq!(
            model.config.active_profiles,
            vec!["git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn use_profiles_at_inserts_at_offset() {
        let config = Config {
            active_profiles: vec!["node".to_string()],
            ..Config::default()
        };
        let profile_config: ProfileConfig = toml::from_str(
            "[[profiles]]\nname = \"git\"\n\n[[profiles]]\nname = \"rust\"\n\n[[profiles]]\nname = \"node\"\n",
        )
        .unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::UseProfilesAt(vec!["git".into(), "rust".into()], 2),
        )
        .unwrap();

        assert_eq!(result.effects.len(), 2);
        // node at 1, git at 2, rust at 3
        assert_eq!(
            model.config.active_profiles,
            vec!["node".to_string(), "git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn use_profiles_at_fails_if_any_missing() {
        let config = Config::default();
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"git\"\n").unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::UseProfilesAt(vec!["git".into(), "nope".into()], 1),
        );

        assert!(result.is_err());
        assert!(model.config.active_profiles.is_empty());
    }

    #[test]
    fn add_alias_to_named_profile() {
        let config = Config::default();
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"rust\"\n").unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::AddAlias(
                "ct".into(),
                "cargo test".into(),
                AliasTarget::Profile("rust".into()),
                false,
            ),
        )
        .unwrap();

        assert_eq!(result.effects, vec![Effect::SaveProfiles]);
        let profile = model.profile_config().get_profile_by_name("rust").unwrap();
        assert_eq!(profile.aliases.iter().count(), 1);
    }
}
