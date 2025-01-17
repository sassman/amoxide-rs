use std::fmt::Display;
use std::usize;

use log::info;
use serde::{Deserialize, Serialize};

use crate::dirs::config_dir;
use crate::{AliasName, AliasSet, Result, TomlAlias};

const CONFIG_FILE: &str = "profiles.toml";

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ProfileConfig {
    profiles: Vec<Profile>,
}

impl ProfileConfig {
    /// Get the default profile
    pub fn get_default_profile(&self) -> Option<&Profile> {
        self.get_profile_by_name("default")
    }

    /// Get the default profile mutable
    pub fn get_default_profile_mut(&mut self) -> Option<&mut Profile> {
        self.get_profile_by_name_mut("default")
    }

    /// Get a profile by name
    pub fn get_profile_by_name(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|p| p.name == name)
    }

    /// Get a profile by name mutable
    pub fn get_profile_by_name_mut(&mut self, name: &str) -> Option<&mut Profile> {
        self.profiles.iter_mut().find(|p| p.name == name)
    }

    /// Get a profile by index
    pub fn get_profile(&self, index: usize) -> Option<&Profile> {
        self.profiles.get(index)
    }

    /// Get a profile by index mutable
    pub fn get_profile_mut(&mut self, index: usize) -> Option<&mut Profile> {
        self.profiles.get_mut(index)
    }

    /// Get the amount of profiles availbale
    pub fn len(&self) -> usize {
        self.profiles.len()
    }
}

pub enum Response {
    ProfileAdded(usize),
    ProfileActivated(usize),
}

impl ProfileConfig {
    /// Provide an iterator over the profiles
    pub fn iter(&self) -> impl Iterator<Item = &Profile> {
        self.profiles.iter()
    }

    /// Add the default profile if it doesn't exist
    pub fn add_default_profile(&mut self) -> Result<Response> {
        let p = Profile::default();
        self.add_profile(&p.name, &None)
    }

    pub fn add_profile(&mut self, name: &str, inherits: &Option<String>) -> Result<Response> {
        let name = name.to_string();
        let mut existing_profile = self.profiles.binary_search_by(|p1| p1.name.cmp(&name));
        if let Ok(i) = existing_profile {
            return Ok(Response::ProfileActivated(i));
        }

        let profile_name = name.clone();
        let profile = Profile::new(name, inherits.clone());
        self.profiles.push(profile.clone());
        self.profiles.sort();
        existing_profile = self
            .profiles
            .binary_search_by(|p1| p1.name.cmp(&profile_name));

        dbg!(&self.profiles);

        let i = existing_profile.unwrap();
        Ok(Response::ProfileAdded(i))
    }
}

impl ProfileConfig {
    pub fn load() -> Result<Self> {
        let profile_config_file = config_dir().join(CONFIG_FILE);
        if !profile_config_file.exists() {
            return Ok(Self::default());
        }

        let toml_str = std::fs::read_to_string(profile_config_file)?;
        let mut decoded: ProfileConfig = toml::from_str(&toml_str)?;

        if decoded.get_default_profile().is_none() {
            let default_profile = Profile::default();
            decoded.profiles.push(default_profile);
        }

        decoded.profiles.sort();

        Ok(decoded)
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = config_dir();
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }
        let profile_config_file = config_dir.join(CONFIG_FILE);
        let toml_str = toml::to_string(self)?;
        std::fs::write(&profile_config_file, toml_str)?;

        info!("saved file {}", profile_config_file.display());
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Profile {
    pub name: String,
    pub inherits: Option<String>,
    pub aliases: AliasSet,
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl PartialEq<Profile> for Profile {
    fn eq(&self, other: &Profile) -> bool {
        self.name == other.name
    }
}

impl PartialOrd<Profile> for Profile {
    fn partial_cmp(&self, other: &Profile) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Eq for Profile {}

impl Ord for Profile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            inherits: None,
            aliases: Default::default(),
        }
    }
}

impl Profile {
    pub fn new(name: String, inherits: Option<String>) -> Self {
        Self {
            name,
            inherits,
            aliases: Default::default(),
        }
    }

    pub fn add_alias(&mut self, name: String, command: String) -> Result<()> {
        let name: AliasName = name.into();
        let alias = TomlAlias::Command(command);

        self.aliases.insert(name, alias);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn test_deserialization_profiles() {
        let toml_str = indoc! {r#"
            [[profiles]]
            name = "default"

            [[profiles]]
            name = "work"
            inherits = "default"
        "#};

        let decoded: ProfileConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(decoded.profiles.len(), 2);
        assert_eq!(decoded.profiles[0].name, "default");
        assert_eq!(decoded.profiles[0].inherits, None);

        assert_eq!(decoded.profiles[1].name, "work");
        assert_eq!(decoded.profiles[1].inherits, Some("default".to_string()));
    }
}
