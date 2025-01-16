use anyhow::anyhow;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};

use crate::dirs::config_dir;
use crate::setup::fish::init_shell_code;
use crate::shell::{Shell, Shells};
use crate::{profile, AddAliasProfile, Message, Profile, ProfileConfig, TomlAlias};

pub struct AppModel {
    pub state: PeristedState,
    profile_config: Option<ProfileConfig>,
    shell: Shells,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PeristedState {
    active_profile: usize,
}

impl PeristedState {
    pub fn save(&self, session_key: &str) -> crate::Result<()> {
        let path = config_dir().join(format!("{session_key}.toml"));
        debug!("Saving state to: {}", path.display());
        let data = toml::to_string(self)?;
        std::fs::write(path, data)?;

        Ok(())
    }

    pub fn load(session_key: &str) -> crate::Result<Self> {
        let path = config_dir().join(format!("{session_key}.toml"));
        debug!("Loading state from: {}", path.display());
        if !path.exists() {
            debug!(
                "Session state file {} did not exist, make sure you run `am env <shell> | source`",
                path.display()
            );
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path)?;
        let state = toml::from_str(&data)?;

        Ok(state)
    }
}

impl Default for AppModel {
    fn default() -> Self {
        Self {
            state: PeristedState::default(),
            profile_config: None,
            shell: Shells::Fish,
        }
    }
}

impl AppModel {
    fn profile_config_mut(&mut self) -> &mut ProfileConfig {
        if self.profile_config.is_none() {
            self.profile_config = Some(ProfileConfig::load().unwrap());
        }
        self.profile_config.as_mut().unwrap()
    }

    fn profile_config(&mut self) -> &ProfileConfig {
        if self.profile_config.is_none() {
            // this is actually not good practice, but for now it's fine
            self.profile_config = Some(ProfileConfig::load().unwrap());
        }
        self.profile_config.as_ref().unwrap()
    }

    fn get_active_profile(&mut self) -> &Profile {
        let active = self.state.active_profile;
        self.profile_config().get_profile(active).unwrap()
    }
}

pub fn update<'a>(model: &mut AppModel, message: Message) -> anyhow::Result<Option<Message<'a>>> {
    match message {
        Message::DoNothing => Ok(None),
        Message::AddAlias(name, cmd, profile) => {
            let config = model.profile_config_mut();
            let profile = match profile {
                AddAliasProfile::Profile(profile_name) => config
                    .get_profile_by_name_mut(&profile_name)
                    .ok_or_else(|| anyhow::anyhow!("Profile not found: {profile_name}"))?,
                AddAliasProfile::ActiveProfile => {
                    let active_profile = model.state.active_profile;
                    let config = model.profile_config_mut();

                    let profile = match config.get_profile_mut(active_profile) {
                        Some(profile) => profile,
                        None => match config.get_default_profile_mut() {
                            Some(profile) => profile,
                            None => {
                                config.add_default_profile()?;
                                config.get_default_profile_mut().unwrap()
                            }
                        },
                    };

                    profile
                }
            };

            profile.add_alias(name, cmd)?;
            Ok(Some(Message::SaveProfiles))
        }
        Message::AddProfile(_, _) => todo!(),
        Message::SetEnv(_) => todo!(),
        Message::ListProfiles => {
            warn!("todo: grab the active shell from the app model, hardcoding to fish for now");
            let active_profile = model.get_active_profile().name.to_owned();
            let shell = crate::shell::Fish::default();
            for profile in model.profile_config().iter() {
                let Profile {
                    name,
                    inherits,
                    aliases,
                } = profile;
                let is_active = if name == &active_profile {
                    " **) Active"
                } else {
                    ""
                };

                if let Some(inherits) = inherits {
                    println!("# [profile: {name}, extends: {inherits}]{is_active}");
                } else {
                    println!("# [profile: {name}]{is_active}");
                }
                let Some(aliases) = aliases.as_ref() else {
                    println!("  # No aliases");
                    continue;
                };
                for (alias_name, command) in aliases.iter() {
                    let name = alias_name.as_ref();
                    let alias = match &command {
                        TomlAlias::Detailed(details) => shell.alias(name, &details.command),
                        TomlAlias::Command(command) => shell.alias(name, command),
                    };
                    println!("  {alias}");
                }
            }
            Ok(None)
        }
        Message::CreateOrUpdateProfile(name, inherits) => {
            match model.profile_config_mut().add_profile(name, inherits)? {
                profile::Response::ProfileAdded(i) => {
                    model.state.active_profile = i;
                    debug!("Profile added: {}", i);
                    // maybe there is a better way than doing this sort of upcall
                    Ok(Some(Message::SaveProfiles))
                }
                profile::Response::ProfileActivated(i) => {
                    model.state.active_profile = i;
                    debug!("Profile activated: {}", i);
                    Ok(None)
                }
            }
        }
        Message::SaveProfiles => {
            model.profile_config().save()?;
            Ok(None)
        }
        Message::InitShell(shell) => {
            let active_profile = model.get_active_profile();

            match shell {
                Shells::Fish => {
                    init_shell_code(&active_profile.name);
                }
                _ => unimplemented!("InitShell for shell: {shell}"),
            }

            Ok(None)
        }
        Message::SetShell(shell) => {
            model.shell = shell.clone();
            Ok(None)
        }
        Message::ActivateProfile(name) => {
            info!("Message::ActivateProfile({name})");
            // checking the profile by name exists
            let _ = model
                .profile_config()
                .get_profile_by_name(&name)
                .ok_or(anyhow!("Profile {name} does not exist."))?;

            // setting the active profile by index
            let i = model
                .profile_config()
                .iter()
                .enumerate()
                .filter(|i| i.1.name.eq(name))
                .next()
                .map(|i| i.0)
                .unwrap();

            model.state.active_profile = i;

            Ok(None)
        }
        Message::ListActiveAliases => {
            let shell = model.shell.clone().as_shell();
            let active_profile = model.get_active_profile();

            for (alias_name, alias_details) in active_profile.aliases.as_ref().unwrap().iter() {
                let name = alias_name.as_ref();
                let alias = match &alias_details {
                    TomlAlias::Detailed(details) => shell.alias(name, &details.command),
                    TomlAlias::Command(command) => shell.alias(name, command),
                };
                println!("{alias}");
            }

            Ok(None)
        }
        Message::RestoreState(session_key) => {
            info!("restoring state from session key: {session_key}");
            model.state = PeristedState::load(session_key)?;
            Ok(None)
        }
        Message::SaveState(session_key) => {
            info!("Saving state for session key: {session_key}");
            model.state.save(session_key)?;
            Ok(None)
        }
    }
}
