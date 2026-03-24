use anyhow::anyhow;

use crate::config::Config;
use crate::display::render_listing;
use crate::hook::generate_hook;
use crate::init::generate_init;
use crate::{profile, AddAliasProfile, Message, Profile, ProfileConfig};

pub struct AppModel {
    pub config: Config,
    profile_config: ProfileConfig,
}

impl Default for AppModel {
    fn default() -> Self {
        let mut profile_config = ProfileConfig::load().unwrap();
        if profile_config.get_default_profile().is_none() {
            profile_config.add_default_profile().unwrap();
        }

        let config = Config::load().unwrap_or_default();

        Self {
            config,
            profile_config,
        }
    }
}

impl AppModel {
    pub fn profile_config_mut(&mut self) -> &mut ProfileConfig {
        &mut self.profile_config
    }

    pub fn profile_config(&self) -> &ProfileConfig {
        &self.profile_config
    }

    pub fn get_active_profile(&self) -> &Profile {
        self.profile_config
            .get_profile_by_name(&self.config.active_profile)
            .unwrap_or_else(|| {
                self.profile_config
                    .get_default_profile()
                    .expect("default profile must exist")
            })
    }
}

pub fn update(model: &mut AppModel, message: Message) -> anyhow::Result<Option<Message>> {
    match message {
        Message::DoNothing => Ok(None),
        Message::AddAlias(name, cmd, target) => {
            let profile = match target {
                AddAliasProfile::Profile(profile_name) => model
                    .profile_config_mut()
                    .get_profile_by_name_mut(&profile_name)
                    .ok_or_else(|| anyhow!("Profile not found: {profile_name}"))?,
                AddAliasProfile::ActiveProfile => {
                    let active = model.config.active_profile.clone();
                    model
                        .profile_config_mut()
                        .get_profile_by_name_mut(&active)
                        .ok_or_else(|| anyhow!("Active profile not found: {active}"))?
                }
            };
            profile.add_alias(name, cmd)?;
            Ok(Some(Message::SaveProfiles))
        }
        Message::RemoveAlias(name, target) => {
            let profile = match target {
                AddAliasProfile::Profile(profile_name) => model
                    .profile_config_mut()
                    .get_profile_by_name_mut(&profile_name)
                    .ok_or_else(|| anyhow!("Profile not found: {profile_name}"))?,
                AddAliasProfile::ActiveProfile => {
                    let active = model.config.active_profile.clone();
                    model
                        .profile_config_mut()
                        .get_profile_by_name_mut(&active)
                        .ok_or_else(|| anyhow!("Active profile not found: {active}"))?
                }
            };
            profile.remove_alias(&name)?;
            Ok(Some(Message::SaveProfiles))
        }
        Message::ListProfiles => {
            let cwd = std::env::current_dir()?;
            let output = render_listing(model.profile_config(), &model.config.active_profile, &cwd);
            println!("{output}");
            Ok(None)
        }
        Message::CreateOrUpdateProfile(name, inherits) => {
            match model.profile_config_mut().add_profile(&name, &inherits)? {
                profile::Response::ProfileAdded(_) => {
                    model.config.active_profile = name;
                    Ok(Some(Message::SaveProfiles))
                }
                profile::Response::ProfileActivated(_) => {
                    model.config.active_profile = name;
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
            let profile = model.get_active_profile();
            let output = generate_init(&shell, profile);
            print!("{output}");
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
        Message::ActivateProfile(name) => {
            let _ = model
                .profile_config()
                .get_profile_by_name(&name)
                .ok_or(anyhow!("Profile '{name}' does not exist."))?;
            model.config.active_profile = name;
            Ok(None)
        }
        Message::RemoveProfile(name) => {
            model.profile_config_mut().remove_profile(&name)?;
            // If the removed profile was active, fall back to default
            if model.config.active_profile == name {
                model.config.active_profile = "default".to_string();
            }
            Ok(Some(Message::SaveProfiles))
        }
    }
}
