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

    /// Resolve the merged alias set for multiple active profiles.
    /// Profiles are merged in order: later profiles override earlier ones.
    pub fn resolve_active_aliases(&self, profile_names: &[String]) -> AliasSet {
        let mut resolved = AliasSet::default();
        for name in profile_names {
            if let Some(profile) = self.get_profile_by_name(name) {
                for (alias_name, alias) in profile.aliases.iter() {
                    resolved.insert(alias_name.clone(), alias.clone());
                }
            }
        }
        resolved
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

    pub fn add_profile(&mut self, name: &str) -> Result<Response> {
        let name = name.to_string();
        let existing_profile = self.profiles.binary_search_by(|p1| p1.name.cmp(&name));
        if let Ok(i) = existing_profile {
            return Ok(Response::ProfileActivated(i));
        }

        let profile_name = name.clone();
        let profile = Profile::new(name);
        self.profiles.push(profile.clone());
        self.profiles.sort();
        let i = self
            .profiles
            .binary_search_by(|p1| p1.name.cmp(&profile_name))
            .unwrap();
        Ok(Response::ProfileAdded(i))
    }

    pub fn remove_profile(&mut self, name: &str) -> Result<()> {
        let idx = self
            .profiles
            .iter()
            .position(|p| p.name == name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{name}' not found"))?;

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
        decoded.profiles.sort();
        Ok(decoded)
    }

    pub fn save(&self) -> Result<()> {
        self.save_to(&config_dir())
    }

    pub fn save_to(&self, config_dir: &std::path::Path) -> Result<()> {
        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir)?;
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

impl Profile {
    pub fn new(name: String) -> Self {
        Self {
            name,
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
        "#};

        let decoded: ProfileConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(decoded.profiles.len(), 2);
        assert_eq!(decoded.profiles[0].name, "default");
        assert_eq!(decoded.profiles[1].name, "work");
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
    fn test_remove_nonexistent_profile() {
        let mut config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "default"
        "#})
        .unwrap();

        let err = config.remove_profile("nope").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_resolve_active_aliases_single_profile() {
        let config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
            gp = "git push"
        "#})
        .unwrap();

        let resolved = config.resolve_active_aliases(&["git".to_string()]);
        assert_eq!(resolved.iter().count(), 2);
    }

    #[test]
    fn test_resolve_active_aliases_merges_in_order() {
        let config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
            t = "git test"

            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
            t = "cargo test"
        "#})
        .unwrap();

        let resolved =
            config.resolve_active_aliases(&["git".to_string(), "rust".to_string()]);
        assert_eq!(resolved.iter().count(), 3); // gs, ct, t (rust overrides git)
        assert!(resolved.iter().any(|(n, _)| n.as_ref() == "gs"));
        assert!(resolved.iter().any(|(n, _)| n.as_ref() == "ct"));
        // "t" should be "cargo test" (from rust, which comes later)
        let t_alias = resolved.iter().find(|(n, _)| n.as_ref() == "t").unwrap();
        assert_eq!(t_alias.1.command(), "cargo test");
    }

    #[test]
    fn test_resolve_active_aliases_empty() {
        let config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#})
        .unwrap();

        let resolved = config.resolve_active_aliases(&[]);
        assert!(resolved.is_empty());
    }
}
