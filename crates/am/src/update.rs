use anyhow::anyhow;

use crate::config::Config;
use crate::hook::generate_hook;
use crate::init::generate_init;
use crate::shell::Shells;
use crate::{profile, AddAliasProfile, Message, Profile, ProfileConfig, TomlAlias};

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
        Message::ListProfiles => {
            let active_name = &model.config.active_profile;
            let shell = Shells::Fish.as_shell();
            for profile in model.profile_config().iter() {
                let Profile {
                    name,
                    inherits,
                    aliases,
                } = profile;
                let is_active = if name == active_name { " (active)" } else { "" };

                if let Some(inherits) = inherits {
                    println!("# [profile: {name}, extends: {inherits}]{is_active}");
                } else {
                    println!("# [profile: {name}]{is_active}");
                }
                if aliases.is_empty() {
                    println!("  # No aliases");
                    continue;
                }
                for (alias_name, command) in aliases.iter() {
                    let n = alias_name.as_ref();
                    let alias = match &command {
                        TomlAlias::Detailed(details) => shell.alias(n, &details.command),
                        TomlAlias::Command(cmd) => shell.alias(n, cmd),
                    };
                    println!("  {alias}");
                }
            }
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
    }
}
