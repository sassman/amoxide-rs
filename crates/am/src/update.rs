use anyhow::anyhow;

use crate::config::Config;
use crate::display::render_listing;
use crate::hook::generate_hook;
use crate::init::{generate_init, generate_reload};
use crate::{profile, AddAliasProfile, Message, Profile, ProfileConfig};

pub struct AppModel {
    pub config: Config,
    profile_config: ProfileConfig,
}

impl Default for AppModel {
    fn default() -> Self {
        let profile_config = ProfileConfig::load().unwrap();
        let config = Config::load().unwrap_or_default();

        Self {
            config,
            profile_config,
        }
    }
}

impl AppModel {
    pub fn new(config: Config, profile_config: ProfileConfig) -> Self {
        Self {
            config,
            profile_config,
        }
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

pub fn update(model: &mut AppModel, message: Message) -> anyhow::Result<Option<Message>> {
    match message {
        Message::DoNothing => Ok(None),
        Message::AddAlias(name, cmd, target, raw) => {
            let profile = resolve_profile_mut(model, &target)?;
            profile.add_alias(name, cmd, raw)?;
            Ok(Some(Message::SaveProfiles))
        }
        Message::RemoveAlias(name, target) => {
            let profile = resolve_profile_mut(model, &target)?;
            profile.remove_alias(&name)?;
            Ok(Some(Message::SaveProfiles))
        }
        Message::ListProfiles => {
            let cwd = std::env::current_dir()?;
            let output = render_listing(
                &model.config.aliases,
                model.profile_config(),
                &model.config.active_profiles,
                &cwd,
            );
            println!("{output}");
            Ok(None)
        }
        Message::CreateProfile(name) => {
            match model.profile_config_mut().add_profile(&name)? {
                profile::Response::ProfileAdded(_) => {
                    if !model.config.active_profiles.contains(&name) {
                        model.config.active_profiles.push(name);
                    }
                    Ok(Some(Message::SaveProfiles))
                }
                profile::Response::ProfileActivated(_) => {
                    if !model.config.active_profiles.contains(&name) {
                        model.config.active_profiles.push(name);
                    }
                    Ok(None)
                }
            }
        }
        Message::SaveProfiles => {
            model.profile_config().save()?;
            Ok(None)
        }
        Message::SaveConfig => {
            model.config.save()?;
            Ok(None)
        }
        Message::InitShell(shell) => {
            let resolved = model
                .profile_config()
                .resolve_active_aliases(&model.config.active_profiles);
            let output = generate_init(&shell, &model.config.aliases, &resolved);
            print!("{output}");
            Ok(None)
        }
        Message::Reload(shell) => {
            let resolved = model
                .profile_config()
                .resolve_active_aliases(&model.config.active_profiles);
            let prev = std::env::var("_AM_ALIASES").ok();
            let output =
                generate_reload(&shell, &model.config.aliases, &resolved, prev.as_deref());
            if !output.is_empty() {
                print!("{output}");
            }
            Ok(None)
        }
        Message::Hook(shell) => {
            let cwd = std::env::current_dir()?;
            let prev = std::env::var("_AM_PROJECT_ALIASES").ok();
            let output = generate_hook(&shell, &cwd, prev.as_deref())?;
            if !output.is_empty() {
                print!("{output}");
            }
            Ok(None)
        }
        Message::ToggleProfile(name) => {
            let _ = model
                .profile_config()
                .get_profile_by_name(&name)
                .ok_or(anyhow!("Profile '{name}' does not exist."))?;
            model.config.toggle_profile(name);
            Ok(None)
        }
        Message::UseProfileAt(name, priority) => {
            let _ = model
                .profile_config()
                .get_profile_by_name(&name)
                .ok_or(anyhow!("Profile '{name}' does not exist."))?;
            model.config.use_profile_at(name, priority);
            Ok(None)
        }
        Message::RemoveProfile(name) => {
            model.profile_config_mut().remove_profile(&name)?;
            model.config.active_profiles.retain(|p| p != &name);
            Ok(Some(Message::SaveProfiles))
        }
    }
}

fn resolve_profile_mut<'a>(
    model: &'a mut AppModel,
    target: &AddAliasProfile,
) -> anyhow::Result<&'a mut Profile> {
    match target {
        AddAliasProfile::Profile(profile_name) => model
            .profile_config_mut()
            .get_profile_by_name_mut(profile_name)
            .ok_or_else(|| anyhow!("Profile not found: {profile_name}")),
        AddAliasProfile::ActiveProfile => {
            let active = model.config.active_profiles.last().cloned().ok_or_else(|| {
                anyhow!("No active profile. Use -p <profile> or -g for global.")
            })?;
            model
                .profile_config_mut()
                .get_profile_by_name_mut(&active)
                .ok_or_else(|| anyhow!("Active profile not found: {active}"))
        }
    }
}
