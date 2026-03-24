use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{AliasDetail, AliasName, AliasSet, TomlAlias};

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub active_profile: Option<String>,
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

    pub fn remove_alias(&mut self, name: &str) -> crate::Result<()> {
        let key: AliasName = name.into();
        self.aliases
            .remove(&key)
            .ok_or_else(|| anyhow::anyhow!("Global alias '{name}' not found"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load_from(dir.path()).unwrap();
        assert_eq!(config.active_profile, None);
        assert!(config.aliases.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config {
            active_profile: Some("rust".to_string()),
            aliases: AliasSet::default(),
        };
        config.add_alias("ll".to_string(), "ls -lha".to_string(), false);
        config.save_to(dir.path()).unwrap();

        let loaded = Config::load_from(dir.path()).unwrap();
        assert_eq!(loaded.active_profile, Some("rust".to_string()));
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
    fn test_backwards_compat_no_aliases_section() {
        let toml_str = r#"active_profile = "rust""#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.active_profile, Some("rust".to_string()));
        assert!(config.aliases.is_empty());
    }
}
