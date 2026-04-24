use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::subcommand::SubcommandSet;
use crate::{AliasDetail, AliasName, AliasSet, TomlAlias};

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct ShellsTomlConfig {
    pub fish: Option<FishConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FishConfig {
    #[serde(default)]
    pub use_abbr: bool,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub aliases: AliasSet,
    #[serde(default, skip_serializing_if = "SubcommandSet::is_empty")]
    pub subcommands: SubcommandSet,
    #[serde(default)]
    pub shell: ShellsTomlConfig,
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

    pub fn add_subcommand(&mut self, key: String, long_subcommands: Vec<String>) {
        self.subcommands.as_mut().insert(key, long_subcommands);
    }

    pub fn remove_subcommand(&mut self, key: &str) -> crate::Result<()> {
        self.subcommands
            .as_mut()
            .remove(key)
            .ok_or_else(|| anyhow::anyhow!("Subcommand alias '{key}' not found"))?;
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
        assert!(config.aliases.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config {
            aliases: AliasSet::default(),
            ..Default::default()
        };
        config.add_alias("ll".to_string(), "ls -lha".to_string(), false);
        config.save_to(dir.path()).unwrap();

        let loaded = Config::load_from(dir.path()).unwrap();
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
    fn test_config_with_subcommands_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config
            .subcommands
            .as_mut()
            .insert("jj:ab".into(), vec!["abandon".into()]);
        config.save_to(dir.path()).unwrap();

        let loaded = Config::load_from(dir.path()).unwrap();
        assert_eq!(loaded.subcommands.as_ref().len(), 1);
        assert_eq!(loaded.subcommands.as_ref()["jj:ab"], vec!["abandon"]);
    }

    #[test]
    fn test_add_and_remove_subcommand() {
        let mut config = Config::default();
        config.add_subcommand("jj:ab".into(), vec!["abandon".into()]);
        assert_eq!(config.subcommands.as_ref().len(), 1);

        config.remove_subcommand("jj:ab").unwrap();
        assert!(config.subcommands.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_subcommand() {
        let mut config = Config::default();
        let err = config.remove_subcommand("jj:nope").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_fish_use_abbr_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            shell: ShellsTomlConfig {
                fish: Some(FishConfig { use_abbr: true }),
            },
            ..Default::default()
        };
        config.save_to(dir.path()).unwrap();
        let loaded = Config::load_from(dir.path()).unwrap();
        assert!(loaded.shell.fish.unwrap().use_abbr);
    }

    #[test]
    fn test_default_config_has_no_shell_config() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load_from(dir.path()).unwrap();
        assert!(config.shell.fish.is_none());
    }
}
