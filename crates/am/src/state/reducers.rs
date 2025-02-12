use anyhow::bail;
use log::debug;

use crate::{profile, state::actions::Action, AddAliasProfile};

use super::state::AppState;

pub type Reducer<State> = fn(&State, &Action) -> crate::Result<State>;

pub fn app_reducer(state: &AppState, action: &Action) -> crate::Result<AppState> {
    let mut new_state = state.clone();

    match action {
        Action::ActivateProfile(_) => todo!(),
        Action::AddProfile(_, _) => todo!(),
        Action::AddAlias(name, cmd, profile) => {
            let config = new_state.profile_config_mut();
            let profile = match profile {
                AddAliasProfile::Profile(profile_name) => config
                    .get_profile_by_name_mut(&profile_name)
                    .ok_or_else(|| anyhow::anyhow!("Profile not found: {profile_name}"))?,
                AddAliasProfile::ActiveProfile => {
                    let active_profile = new_state.state.active_profile;
                    let config = new_state.profile_config_mut();

                    let profile = match config.get_profile_mut(active_profile) {
                        Some(profile) => profile,
                        None => bail!("Active profile not found, please check your config."),
                    };

                    profile
                }
            };

            profile.add_alias(name, cmd)?;
            // well, this is a bit of a hack, but it works for now
            return app_reducer(&new_state, &Action::SaveProfiles);
        }
        Action::CreateOrUpdateProfile(name, inherits) => {
            match new_state
                .profile_config_mut()
                .add_profile(&name, &inherits)?
            {
                profile::Response::ProfileAdded(i) => {
                    new_state.state.active_profile = i;
                    debug!("Profile added: {}", i);
                    // maybe there is a better way than doing this sort of upcall
                    return app_reducer(&new_state, &Action::SaveProfiles);
                }
                profile::Response::ProfileActivated(i) => {
                    new_state.state.active_profile = i;
                    debug!("Profile activated: {}", i);
                }
            }
        }
        Action::DoNothing => (),
        Action::InitShell(shells) => todo!(),
        Action::SetEnv(env) => {
            println!("SetEnv with {env}");
        }
        Action::ListProfiles => todo!(),
        Action::ListActiveAliases => todo!(),
        Action::RestoreState(_) => todo!(),
        Action::SaveState(_) => todo!(),
        Action::SaveProfiles => todo!(),
        Action::SetShell(shells) => todo!(),
    }

    Ok(new_state)
}
