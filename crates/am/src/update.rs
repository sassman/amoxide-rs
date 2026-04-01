use anyhow::anyhow;

use crate::config::Config;
use crate::display::render_listing;
use crate::effects::Effect;
use crate::hook::generate_hook;
use crate::init::{generate_init, generate_reload};
use crate::project::ProjectAliases;
use crate::{profile, AliasTarget, Message, Profile, ProfileConfig};

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
}

impl Default for AppModel {
    fn default() -> Self {
        let profile_config = ProfileConfig::load().unwrap();
        let config = Config::load().unwrap_or_default();

        Self {
            config,
            cwd: std::env::current_dir().unwrap_or_default(),
            profile_config,
        }
    }
}

impl AppModel {
    pub fn new(config: Config, profile_config: ProfileConfig) -> Self {
        Self {
            config,
            cwd: std::env::current_dir().unwrap_or_default(),
            profile_config,
        }
    }

    pub fn with_cwd(mut self, cwd: std::path::PathBuf) -> Self {
        self.cwd = cwd;
        self
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
                match ProjectAliases::find_local_path_in(&model.cwd) {
                    Some(_) => Ok(UpdateResult::effect(Effect::AddLocalAlias {
                        name,
                        cmd,
                        raw,
                    })),
                    None => {
                        model.config.add_alias(name, cmd, raw);
                        Ok(UpdateResult::effect(Effect::SaveConfig))
                    }
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
                match ProjectAliases::find_local_path_in(&model.cwd) {
                    Some(_) => Ok(UpdateResult::effect(Effect::RemoveLocalAlias { name })),
                    None => {
                        model.config.remove_alias(&name)?;
                        Ok(UpdateResult::effect(Effect::SaveConfig))
                    }
                }
            }
            target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                let profile = resolve_profile_mut(model, &target)?;
                profile.remove_alias(&name)?;
                Ok(UpdateResult::effect(Effect::SaveProfiles))
            }
        },
        Message::ListProfiles => {
            let output = render_listing(
                &model.config.aliases,
                model.profile_config(),
                &model.config.active_profiles,
                &model.cwd,
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
        Message::ToggleProfile(name) => {
            let _ = model
                .profile_config()
                .get_profile_by_name(&name)
                .ok_or(anyhow!("Profile '{name}' does not exist."))?;
            model.config.toggle_profile(name);
            Ok(UpdateResult::done())
        }
        Message::UseProfileAt(name, priority) => {
            let _ = model
                .profile_config()
                .get_profile_by_name(&name)
                .ok_or(anyhow!("Profile '{name}' does not exist."))?;
            model.config.use_profile_at(name, priority);
            Ok(UpdateResult::done())
        }
        Message::RemoveProfile(name) => {
            model.profile_config_mut().remove_profile(&name)?;
            model.config.active_profiles.retain(|p| p != &name);
            Ok(UpdateResult::effect(Effect::SaveProfiles))
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
        let mut model =
            AppModel::new(config, profile_config).with_cwd(dir.path().to_path_buf());

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
        let mut model =
            AppModel::new(config, profile_config).with_cwd(dir.path().to_path_buf());

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
