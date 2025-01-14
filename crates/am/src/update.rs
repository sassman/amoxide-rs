use log::debug;

use crate::setup::fish::{
    init_shell_code, setup_fish_config_file, setup_fish_functions_for_profile,
};
use crate::shell::{Shell, Shells};
use crate::{profile, AddAliasProfile, Message, Profile, ProfileConfig, TomlAlias};

#[derive(Default)]
pub struct AppModel {
    active_profile: usize,
    profile_config: Option<ProfileConfig>,
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
}

pub fn update<'a>(model: &mut AppModel, message: Message) -> anyhow::Result<Option<Message<'a>>> {
    match message {
        Message::AddAlias(name, cmd, profile) => {
            let config = model.profile_config_mut();
            let profile = match profile {
                AddAliasProfile::Profile(profile_name) => config
                    .get_profile_by_name_mut(&profile_name)
                    .ok_or_else(|| anyhow::anyhow!("Profile not found: {profile_name}"))?,
                AddAliasProfile::ActiveProfile => {
                    let active_profile = model.active_profile;
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
            let shell = crate::shell::Fish::default();
            for profile in model.profile_config().iter() {
                let Profile {
                    name,
                    inherits,
                    aliases,
                } = profile;
                if let Some(inherits) = inherits {
                    println!("# [profile({name}) extends {inherits}]");
                } else {
                    println!("# [profile({name})]");
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
        Message::LoadOrCreateProfile(name, inherits) => {
            match model.profile_config_mut().add_profile(name, inherits)? {
                profile::Response::ProfileAdded(i) => {
                    model.active_profile = i;
                    debug!("Profile added: {}", i);
                    // maybe there is a better way than doing this sort of upcall
                    Ok(Some(Message::SaveProfiles))
                }
                profile::Response::ProfileActivated(i) => {
                    model.active_profile = i;
                    debug!("Profile activated: {}", i);
                    Ok(None)
                }
            }
        }
        Message::SaveProfiles => {
            model.profile_config().save()?;
            Ok(None)
        }
        Message::ListAliasesForShell(shell) => {
            let active_profile = model.active_profile;
            let active_profile = model.profile_config().get_profile(active_profile).unwrap();

            match shell {
                Shells::Fish => {
                    setup_fish_config_file()?;
                    setup_fish_functions_for_profile(&active_profile)?;
                }
                _ => unimplemented!("ListAliasesForShell for shell: {shell}"),
            }

            Ok(None)
        }
        Message::InitShell(shell) => {
            let active_profile = model.active_profile;
            let active_profile = model.profile_config().get_profile(active_profile).unwrap();

            match shell {
                Shells::Fish => {
                    init_shell_code(&active_profile)?;
                }
                _ => unimplemented!("InitShell for shell: {shell}"),
            }

            Ok(None)
        }
    }
}
