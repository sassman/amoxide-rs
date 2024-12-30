use log::info;

use crate::{Message, Profile, ProfileConfig};

#[derive(Default)]
pub struct AppModel {
    active_profile: Profile,
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

pub fn update(model: &mut AppModel, message: Message) {
    match message {
        Message::AddAlias(name, cmd) => {
            println!("alias {name}='{cmd}'");
            todo!()
        }
        Message::AddProfile(_, _) => todo!(),
        Message::SetEnv(_) => todo!(),
        Message::ListProfiles => {
            for profile in model.profile_config().iter() {
                let Profile { name, inherits } = profile;
                if let Some(inherits) = inherits {
                    println!("{name} -> {inherits}");
                } else {
                    println!("{name}");
                }
            }
        }
        Message::LoadOrCreateProfile(name, inherits) => {
            if let Some(inherits) = inherits {
                info!("Adding profile {} with inheritance from {}", name, inherits);
                model
                    .profile_config_mut()
                    .add_profile_with_inheritance(name, Some(inherits));
            } else {
                info!("Adding profile {}", name);
            }
            todo!()
        }
    }
}
