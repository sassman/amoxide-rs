use std::fmt::Display;

use log::info;
use serde::{Deserialize, Serialize};

use crate::dirs::config_dir;
use crate::{AliasDetail, AliasName, AliasSet, Result, TomlAlias};

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

    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
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

        let i = existing_profile.unwrap();
        Ok(Response::ProfileAdded(i))
    }

    pub fn remove_profile(&mut self, name: &str) -> Result<()> {
        if name == "default" {
            anyhow::bail!("Cannot remove the default profile");
        }
        let idx = self
            .profiles
            .iter()
            .position(|p| p.name == name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{name}' not found"))?;

        // Resolve inheritance: re-parent dependents to the removed profile's parent
        let new_parent = self.profiles[idx].inherits.clone();
        for profile in &mut self.profiles {
            if profile.inherits.as_deref() == Some(name) {
                profile.inherits = new_parent.clone();
            }
        }

        self.profiles.remove(idx);
        Ok(())
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
    #[serde(default)]
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
        Some(self.cmp(other))
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

    pub fn add_alias(&mut self, name: String, command: String, raw: bool) -> Result<()> {
        let key: AliasName = name.into();
        let alias = if raw {
            TomlAlias::Detailed(AliasDetail {
                command,
                description: None,
                raw: true,
            })
        } else {
            TomlAlias::Command(command)
        };
        self.aliases.insert(key, alias);
        Ok(())
    }

    pub fn remove_alias(&mut self, name: &str) -> Result<()> {
        let key: AliasName = name.into();
        self.aliases
            .remove(&key)
            .ok_or_else(|| anyhow::anyhow!("Alias '{name}' not found"))?;
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

    #[test]
    fn test_remove_profile() {
        let mut config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "default"

            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#})
        .unwrap();

        config.remove_profile("git").unwrap();
        assert_eq!(config.len(), 1);
        assert!(config.get_profile_by_name("git").is_none());
    }

    #[test]
    fn test_remove_profile_reparents_dependents() {
        let mut config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "base"

            [[profiles]]
            name = "git"
            inherits = "base"

            [[profiles]]
            name = "rust"
            inherits = "git"
        "#})
        .unwrap();

        config.remove_profile("git").unwrap();
        assert_eq!(config.len(), 2);
        // rust should now inherit from base
        let rust = config.get_profile_by_name("rust").unwrap();
        assert_eq!(rust.inherits.as_deref(), Some("base"));
    }

    #[test]
    fn test_remove_root_profile_clears_inherits() {
        let mut config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "base"

            [[profiles]]
            name = "git"
            inherits = "base"
        "#})
        .unwrap();

        config.remove_profile("base").unwrap();
        let git = config.get_profile_by_name("git").unwrap();
        assert_eq!(git.inherits, None);
    }

    #[test]
    fn test_cannot_remove_default_profile() {
        let mut config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "default"
        "#})
        .unwrap();

        let err = config.remove_profile("default").unwrap_err();
        assert!(err
            .to_string()
            .contains("Cannot remove the default profile"));
    }

    #[test]
    fn test_remove_nonexistent_profile() {
        let mut config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "default"
        "#})
        .unwrap();

        let err = config.remove_profile("nope").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }
}
