use std::path::Path;

use serde::{Deserialize, Serialize};

const SESSION_FILE: &str = "session.toml";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Session {
    #[serde(default)]
    pub active_profiles: Vec<String>,
}

impl Session {
    pub fn load_from(config_dir: &Path) -> crate::Result<Self> {
        let path = config_dir.join(SESSION_FILE);
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = std::fs::read_to_string(path)?;
        let session = toml::from_str(&data)?;
        Ok(session)
    }

    pub fn save_to(&self, config_dir: &Path) -> crate::Result<()> {
        if !config_dir.exists() {
            std::fs::create_dir_all(config_dir)?;
        }
        let path = config_dir.join(SESSION_FILE);
        let data = toml::to_string(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn load() -> crate::Result<Self> {
        Self::load_from(&crate::dirs::config_dir())
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
    fn test_default_session_when_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let session = Session::load_from(dir.path()).unwrap();
        assert!(session.active_profiles.is_empty());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let session = Session {
            active_profiles: vec!["git".to_string(), "rust".to_string()],
        };
        session.save_to(dir.path()).unwrap();
        let loaded = Session::load_from(dir.path()).unwrap();
        assert_eq!(
            loaded.active_profiles,
            vec!["git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn test_toggle_profile_appends() {
        let mut session = Session::default();
        session.toggle_profile("git".to_string());
        assert_eq!(session.active_profiles, vec!["git".to_string()]);
        session.toggle_profile("rust".to_string());
        assert_eq!(
            session.active_profiles,
            vec!["git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn test_toggle_profile_removes() {
        let mut session = Session {
            active_profiles: vec!["git".to_string(), "rust".to_string()],
        };
        session.toggle_profile("git".to_string());
        assert_eq!(session.active_profiles, vec!["rust".to_string()]);
    }

    #[test]
    fn test_is_active() {
        let session = Session {
            active_profiles: vec!["git".to_string()],
        };
        assert!(session.is_active("git"));
        assert!(!session.is_active("rust"));
    }

    #[test]
    fn test_activation_order() {
        let session = Session {
            active_profiles: vec!["git".to_string(), "rust".to_string(), "node".to_string()],
        };
        assert_eq!(session.activation_order("git"), Some(1));
        assert_eq!(session.activation_order("rust"), Some(2));
        assert_eq!(session.activation_order("node"), Some(3));
        assert_eq!(session.activation_order("python"), None);
    }

    #[test]
    fn test_use_profile_at_inserts_at_position() {
        let mut session = Session {
            active_profiles: vec!["git".to_string(), "rust".to_string()],
        };
        session.use_profile_at("node".to_string(), 1);
        assert_eq!(
            session.active_profiles,
            vec!["node".to_string(), "git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn test_use_profile_at_repositions_existing() {
        let mut session = Session {
            active_profiles: vec!["git".to_string(), "rust".to_string(), "node".to_string()],
        };
        session.use_profile_at("node".to_string(), 1);
        assert_eq!(
            session.active_profiles,
            vec!["node".to_string(), "git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn test_use_profile_at_clamps_to_end() {
        let mut session = Session {
            active_profiles: vec!["git".to_string()],
        };
        session.use_profile_at("rust".to_string(), 100);
        assert_eq!(
            session.active_profiles,
            vec!["git".to_string(), "rust".to_string()]
        );
    }
}
