use std::path::Path;

use serde::{Deserialize, Serialize};

const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub active_profile: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            active_profile: "default".to_string(),
        }
    }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::load_from(dir.path()).unwrap();
        assert_eq!(config.active_profile, "default");
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config {
            active_profile: "rust".to_string(),
        };
        config.save_to(dir.path()).unwrap();

        let loaded = Config::load_from(dir.path()).unwrap();
        assert_eq!(loaded.active_profile, "rust");
    }
}
