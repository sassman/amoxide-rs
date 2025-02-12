use log::debug;
use serde::{Deserialize, Serialize};

use crate::{dirs::config_dir, Profile, ProfileConfig};

/// This is the marker trait for the state of the application
pub trait ReduxState: std::fmt::Debug {}

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct PeristedState {
    pub active_profile: usize,
}

impl PeristedState {
    pub fn save(&self, session_key: &str) -> crate::Result<()> {
        let dir = config_dir();
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
        }
        let path = dir.join(format!("{session_key}.toml"));
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

#[derive(Debug, Clone)]
pub struct AppState {
    pub state: PeristedState,
    pub profile_config: ProfileConfig,
    pub shell: crate::shell::Shells,
}

impl ReduxState for AppState {}

impl Default for AppState {
    fn default() -> Self {
        let mut profile_config = ProfileConfig::load().unwrap();
        // ensure there is a default profile
        if profile_config.get_default_profile().is_none() {
            profile_config.add_default_profile().unwrap();
        }

        Self {
            state: PeristedState::default(),
            profile_config,
            shell: crate::shell::Shells::Fish,
        }
    }
}

impl AppState {
    pub fn save(&self, session_key: &str) -> crate::Result<()> {
        self.state.save(session_key)
    }

    pub fn load(session_key: &str) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let state = PeristedState::load(session_key)?;
        let profile_config = ProfileConfig::load()?;
        Ok(Self {
            state,
            profile_config,
            shell: crate::shell::Shells::Fish,
        })
    }

    pub fn get_active_profile(&self) -> &Profile {
        let active = self.state.active_profile;
        self.profile_config().get_profile(active).unwrap()
    }

    pub fn profile_config_mut(&mut self) -> &mut ProfileConfig {
        &mut self.profile_config
    }

    pub fn profile_config(&self) -> &ProfileConfig {
        &self.profile_config
    }
}
