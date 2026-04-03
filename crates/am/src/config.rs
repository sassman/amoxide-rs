use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{AliasDetail, AliasName, AliasSet, TomlAlias};

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub active_profiles: Vec<String>,
    #[serde(default)]
    pub aliases: AliasSet,
}

impl Config {
    pub fn load_from(config_dir: &Path) -> crate::Result<Self> {
        let path = config_dir.join(CONFIG_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path)?;
        let config = toml::from_str(&data)?;
        Ok(config)
    }

    pub fn save_to(&self, config_dir: &Path) -> crate::Result<()> {
        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir)?;
        }
        let path = config_dir.join(CONFIG_FILE);
        let data = toml::to_string(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn load() -> crate::Result<Self> {
        Self::load_from(&crate::dirs::config_dir())
    }

    pub fn save(&self) -> crate::Result<()> {
        self.save_to(&crate::dirs::config_dir())
    }

    pub fn add_alias(&mut self, name: String, command: String, raw: bool) {
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
    }

    pub fn merge_aliases(&mut self, incoming: AliasSet) {
        for (name, alias) in incoming.iter() {
            self.aliases.insert(name.clone(), alias.clone());
        }
    }

    pub fn remove_alias(&mut self, name: &str) -> crate::Result<()> {
        let key: AliasName = name.into();
        self.aliases
            .remove(&key)
            .ok_or_else(|| anyhow::anyhow!("Global alias '{name}' not found"))?;
        Ok(())
    }

    pub fn toggle_profile(&mut self, name: String) {
        if let Some(pos) = self.active_profiles.iter().position(|p| p == &name) {
            self.active_profiles.remove(pos);
        } else {
            self.active_profiles.push(name);
        }
    }

    pub fn activation_order(&self, name: &str) -> Option<usize> {
        self.active_profiles
            .iter()
            .position(|p| p == name)
            .map(|i| i + 1)
    }

    pub fn is_active(&self, name: &str) -> bool {
        self.active_profiles.contains(&name.to_string())
    }

    pub fn use_profile_at(&mut self, name: String, priority: usize) {
        self.active_profiles.retain(|p| p != &name);
        let idx = (priority.saturating_sub(1)).min(self.active_profiles.len());
        self.active_profiles.insert(idx, name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load_from(dir.path()).unwrap();
        assert!(config.active_profiles.is_empty());
        assert!(config.aliases.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config {
            active_profiles: vec!["rust".to_string()],
            aliases: AliasSet::default(),
        };
        config.add_alias("ll".to_string(), "ls -lha".to_string(), false);
        config.save_to(dir.path()).unwrap();

        let loaded = Config::load_from(dir.path()).unwrap();
        assert_eq!(loaded.active_profiles, vec!["rust".to_string()]);
        assert_eq!(loaded.aliases.iter().count(), 1);
    }

    #[test]
    fn test_add_and_remove_global_alias() {
        let mut config = Config::default();
        config.add_alias("ll".to_string(), "ls -lha".to_string(), false);
        assert_eq!(config.aliases.iter().count(), 1);

        config.remove_alias("ll").unwrap();
        assert!(config.aliases.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_global_alias() {
        let mut config = Config::default();
        let err = config.remove_alias("nope").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_active_profiles_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            active_profiles: vec!["git".to_string(), "rust".to_string()],
            aliases: AliasSet::default(),
        };
        config.save_to(dir.path()).unwrap();

        let loaded = Config::load_from(dir.path()).unwrap();
        assert_eq!(
            loaded.active_profiles,
            vec!["git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn test_toggle_profile_appends() {
        let mut config = Config::default();
        config.toggle_profile("git".to_string());
        assert_eq!(config.active_profiles, vec!["git".to_string()]);
        config.toggle_profile("rust".to_string());
        assert_eq!(
            config.active_profiles,
            vec!["git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn test_toggle_profile_removes() {
        let mut config = Config {
            active_profiles: vec!["git".to_string(), "rust".to_string()],
            aliases: AliasSet::default(),
        };
        config.toggle_profile("git".to_string());
        assert_eq!(config.active_profiles, vec!["rust".to_string()]);
    }

    #[test]
    fn test_profile_activation_order() {
        let config = Config {
            active_profiles: vec!["git".to_string(), "rust".to_string(), "node".to_string()],
            aliases: AliasSet::default(),
        };
        assert_eq!(config.activation_order("git"), Some(1));
        assert_eq!(config.activation_order("rust"), Some(2));
        assert_eq!(config.activation_order("node"), Some(3));
        assert_eq!(config.activation_order("python"), None);
    }

    #[test]
    fn test_use_profile_at_inserts_at_position() {
        let mut config = Config {
            active_profiles: vec!["git".to_string(), "rust".to_string()],
            aliases: AliasSet::default(),
        };
        config.use_profile_at("node".to_string(), 1);
        assert_eq!(
            config.active_profiles,
            vec!["node".to_string(), "git".to_string(), "rust".to_string(),]
        );
    }

    #[test]
    fn test_use_profile_at_repositions_existing() {
        let mut config = Config {
            active_profiles: vec!["git".to_string(), "rust".to_string(), "node".to_string()],
            aliases: AliasSet::default(),
        };
        config.use_profile_at("node".to_string(), 1);
        assert_eq!(
            config.active_profiles,
            vec!["node".to_string(), "git".to_string(), "rust".to_string(),]
        );
    }

    #[test]
    fn test_merge_aliases_into_global() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let mut incoming = AliasSet::default();
        incoming.insert("gs".into(), TomlAlias::Command("git status".into()));
        incoming.insert("ll".into(), TomlAlias::Command("ls -la".into()));
        config.merge_aliases(incoming);
        assert_eq!(config.aliases.len(), 2);
        assert_eq!(
            config
                .aliases
                .get(&AliasName::from("ll"))
                .unwrap()
                .command(),
            "ls -la"
        );
    }

    #[test]
    fn test_use_profile_at_clamps_to_end() {
        let mut config = Config {
            active_profiles: vec!["git".to_string()],
            aliases: AliasSet::default(),
        };
        config.use_profile_at("rust".to_string(), 100);
        assert_eq!(
            config.active_profiles,
            vec!["git".to_string(), "rust".to_string()]
        );
    }
}
