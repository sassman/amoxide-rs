use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const SECURITY_FILE: &str = "security.toml";

#[derive(Debug, Clone, PartialEq)]
pub enum TrustStatus {
    Trusted,
    Untrusted,
    Tampered,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedEntry {
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamperedEntry {
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UntrustedEntry {
    pub path: PathBuf,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default)]
    pub trusted: Vec<TrustedEntry>,
    #[serde(default)]
    pub tampered: Vec<TamperedEntry>,
    #[serde(default)]
    pub untrusted: Vec<UntrustedEntry>,
}

impl SecurityConfig {
    pub fn load_from(config_dir: &Path) -> crate::Result<Self> {
        let path = config_dir.join(SECURITY_FILE);
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
        let path = config_dir.join(SECURITY_FILE);
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

    /// Check trust status for a given path and current file hash.
    /// If a trusted entry has a hash mismatch, transitions to tampered in-memory.
    pub fn check(&mut self, path: &Path, current_hash: &str) -> TrustStatus {
        // Check tampered first (already flagged)
        if self.tampered.iter().any(|e| e.path == path) {
            return TrustStatus::Tampered;
        }

        // Check untrusted
        if self.untrusted.iter().any(|e| e.path == path) {
            return TrustStatus::Untrusted;
        }

        // Check trusted — verify hash
        if let Some(pos) = self.trusted.iter().position(|e| e.path == path) {
            if self.trusted[pos].hash == current_hash {
                return TrustStatus::Trusted;
            }
            // Hash mismatch — move to tampered
            let entry = self.trusted.remove(pos);
            self.tampered.push(TamperedEntry {
                path: entry.path,
                hash: entry.hash,
            });
            return TrustStatus::Tampered;
        }

        TrustStatus::Unknown
    }

    /// Add or move an entry to the trusted list with the given hash.
    /// Removes the path from tampered/untrusted if present.
    pub fn trust(&mut self, path: &Path, hash: &str) {
        self.forget(path);
        self.trusted.push(TrustedEntry {
            path: path.to_path_buf(),
            hash: hash.to_string(),
        });
    }

    /// Move an entry to the untrusted list.
    /// Removes the path from trusted/tampered if present.
    pub fn untrust(&mut self, path: &Path) {
        self.forget(path);
        self.untrusted.push(UntrustedEntry {
            path: path.to_path_buf(),
        });
    }

    /// Remove an entry entirely from all lists.
    pub fn forget(&mut self, path: &Path) {
        self.trusted.retain(|e| e.path != path);
        self.tampered.retain(|e| e.path != path);
        self.untrusted.retain(|e| e.path != path);
    }

    /// Update the hash for an already-trusted path (after local alias mutation).
    /// No-op if the path is not in the trusted list.
    pub fn update_hash(&mut self, path: &Path, new_hash: &str) {
        if let Some(entry) = self.trusted.iter_mut().find(|e| e.path == path) {
            entry.hash = new_hash.to_string();
        }
    }

    /// Check if a path is tracked in any list.
    pub fn is_tracked(&self, path: &Path) -> bool {
        self.trusted.iter().any(|e| e.path == path)
            || self.tampered.iter().any(|e| e.path == path)
            || self.untrusted.iter().any(|e| e.path == path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_unknown_path_returns_unknown() {
        let mut config = SecurityConfig::default();
        let status = config.check(Path::new("/some/path/.aliases"), "abc123");
        assert_eq!(status, TrustStatus::Unknown);
    }

    #[test]
    fn check_trusted_matching_hash_returns_trusted() {
        let mut config = SecurityConfig {
            trusted: vec![TrustedEntry {
                path: PathBuf::from("/project/.aliases"),
                hash: "abc123".to_string(),
            }],
            ..Default::default()
        };
        let status = config.check(Path::new("/project/.aliases"), "abc123");
        assert_eq!(status, TrustStatus::Trusted);
    }

    #[test]
    fn check_trusted_mismatched_hash_returns_tampered_and_moves() {
        let mut config = SecurityConfig {
            trusted: vec![TrustedEntry {
                path: PathBuf::from("/project/.aliases"),
                hash: "old_hash".to_string(),
            }],
            ..Default::default()
        };
        let status = config.check(Path::new("/project/.aliases"), "new_hash");
        assert_eq!(status, TrustStatus::Tampered);
        assert!(config.trusted.is_empty());
        assert_eq!(config.tampered.len(), 1);
        assert_eq!(config.tampered[0].hash, "old_hash");
    }

    #[test]
    fn check_untrusted_returns_untrusted() {
        let mut config = SecurityConfig {
            untrusted: vec![UntrustedEntry {
                path: PathBuf::from("/project/.aliases"),
            }],
            ..Default::default()
        };
        let status = config.check(Path::new("/project/.aliases"), "abc123");
        assert_eq!(status, TrustStatus::Untrusted);
    }

    #[test]
    fn check_tampered_returns_tampered() {
        let mut config = SecurityConfig {
            tampered: vec![TamperedEntry {
                path: PathBuf::from("/project/.aliases"),
                hash: "old_hash".to_string(),
            }],
            ..Default::default()
        };
        let status = config.check(Path::new("/project/.aliases"), "whatever");
        assert_eq!(status, TrustStatus::Tampered);
    }

    #[test]
    fn trust_adds_to_trusted_list() {
        let mut config = SecurityConfig::default();
        config.trust(Path::new("/project/.aliases"), "abc123");
        assert_eq!(config.trusted.len(), 1);
        assert_eq!(config.trusted[0].hash, "abc123");
    }

    #[test]
    fn trust_moves_from_tampered_to_trusted() {
        let mut config = SecurityConfig {
            tampered: vec![TamperedEntry {
                path: PathBuf::from("/project/.aliases"),
                hash: "old_hash".to_string(),
            }],
            ..Default::default()
        };
        config.trust(Path::new("/project/.aliases"), "new_hash");
        assert!(config.tampered.is_empty());
        assert_eq!(config.trusted.len(), 1);
        assert_eq!(config.trusted[0].hash, "new_hash");
    }

    #[test]
    fn trust_moves_from_untrusted_to_trusted() {
        let mut config = SecurityConfig {
            untrusted: vec![UntrustedEntry {
                path: PathBuf::from("/project/.aliases"),
            }],
            ..Default::default()
        };
        config.trust(Path::new("/project/.aliases"), "abc123");
        assert!(config.untrusted.is_empty());
        assert_eq!(config.trusted.len(), 1);
    }

    #[test]
    fn untrust_moves_from_trusted_to_untrusted() {
        let mut config = SecurityConfig {
            trusted: vec![TrustedEntry {
                path: PathBuf::from("/project/.aliases"),
                hash: "abc123".to_string(),
            }],
            ..Default::default()
        };
        config.untrust(Path::new("/project/.aliases"));
        assert!(config.trusted.is_empty());
        assert_eq!(config.untrusted.len(), 1);
    }

    #[test]
    fn untrust_moves_from_tampered_to_untrusted() {
        let mut config = SecurityConfig {
            tampered: vec![TamperedEntry {
                path: PathBuf::from("/project/.aliases"),
                hash: "old_hash".to_string(),
            }],
            ..Default::default()
        };
        config.untrust(Path::new("/project/.aliases"));
        assert!(config.tampered.is_empty());
        assert_eq!(config.untrusted.len(), 1);
    }

    #[test]
    fn forget_removes_from_trusted() {
        let mut config = SecurityConfig {
            trusted: vec![TrustedEntry {
                path: PathBuf::from("/project/.aliases"),
                hash: "abc123".to_string(),
            }],
            ..Default::default()
        };
        config.forget(Path::new("/project/.aliases"));
        assert!(config.trusted.is_empty());
    }

    #[test]
    fn forget_removes_from_untrusted() {
        let mut config = SecurityConfig {
            untrusted: vec![UntrustedEntry {
                path: PathBuf::from("/project/.aliases"),
            }],
            ..Default::default()
        };
        config.forget(Path::new("/project/.aliases"));
        assert!(config.untrusted.is_empty());
    }

    #[test]
    fn forget_removes_from_tampered() {
        let mut config = SecurityConfig {
            tampered: vec![TamperedEntry {
                path: PathBuf::from("/project/.aliases"),
                hash: "old".to_string(),
            }],
            ..Default::default()
        };
        config.forget(Path::new("/project/.aliases"));
        assert!(config.tampered.is_empty());
    }

    #[test]
    fn update_hash_updates_trusted_entry() {
        let mut config = SecurityConfig {
            trusted: vec![TrustedEntry {
                path: PathBuf::from("/project/.aliases"),
                hash: "old_hash".to_string(),
            }],
            ..Default::default()
        };
        config.update_hash(Path::new("/project/.aliases"), "new_hash");
        assert_eq!(config.trusted[0].hash, "new_hash");
    }

    #[test]
    fn update_hash_noop_if_not_trusted() {
        let mut config = SecurityConfig {
            untrusted: vec![UntrustedEntry {
                path: PathBuf::from("/project/.aliases"),
            }],
            ..Default::default()
        };
        config.update_hash(Path::new("/project/.aliases"), "new_hash");
        assert!(config.trusted.is_empty());
    }

    #[test]
    fn is_tracked_returns_true_for_all_lists() {
        let mut config = SecurityConfig::default();
        config.trust(Path::new("/a/.aliases"), "hash");
        config.untrust(Path::new("/b/.aliases"));
        config.tampered.push(TamperedEntry {
            path: PathBuf::from("/c/.aliases"),
            hash: "old".to_string(),
        });
        assert!(config.is_tracked(Path::new("/a/.aliases")));
        assert!(config.is_tracked(Path::new("/b/.aliases")));
        assert!(config.is_tracked(Path::new("/c/.aliases")));
        assert!(!config.is_tracked(Path::new("/d/.aliases")));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = SecurityConfig::default();
        config.trust(Path::new("/a/.aliases"), "hash_a");
        config.untrust(Path::new("/b/.aliases"));
        config.tampered.push(TamperedEntry {
            path: PathBuf::from("/c/.aliases"),
            hash: "old_c".to_string(),
        });

        config.save_to(dir.path()).unwrap();
        let loaded = SecurityConfig::load_from(dir.path()).unwrap();

        assert_eq!(loaded.trusted.len(), 1);
        assert_eq!(loaded.trusted[0].path, Path::new("/a/.aliases"));
        assert_eq!(loaded.untrusted.len(), 1);
        assert_eq!(loaded.tampered.len(), 1);
    }

    #[test]
    fn load_missing_file_returns_default() {
        let dir = tempfile::tempdir().unwrap();
        let config = SecurityConfig::load_from(dir.path()).unwrap();
        assert!(config.trusted.is_empty());
        assert!(config.untrusted.is_empty());
        assert!(config.tampered.is_empty());
    }
}
