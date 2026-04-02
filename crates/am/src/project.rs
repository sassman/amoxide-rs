use log::warn;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::dirs::home_dir;
use crate::{AliasDetail, AliasName, AliasSet, TomlAlias};

pub const ALIASES_FILE: &str = ".aliases";

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProjectAliases {
    pub aliases: AliasSet,
}

impl ProjectAliases {
    /// Walk up from `start_dir` looking for a `.aliases` file.
    /// Stops before the user's home directory.
    pub fn find(start_dir: &Path) -> crate::Result<Option<Self>> {
        match Self::find_path(start_dir)? {
            Some(path) => {
                let data = std::fs::read_to_string(&path)?;
                match toml::from_str::<ProjectAliases>(&data) {
                    Ok(project) => Ok(Some(project)),
                    Err(e) => {
                        warn!("Skipping {}: {e}", path.display());
                        Ok(None)
                    }
                }
            }
            None => Ok(None),
        }
    }

    /// Walk up from `start_dir` looking for a `.aliases` file path.
    /// Returns the path without parsing.
    pub fn find_path(start_dir: &Path) -> crate::Result<Option<PathBuf>> {
        let home = home_dir();
        let mut dir = start_dir.to_path_buf();
        loop {
            if home.as_ref().is_some_and(|h| h == &dir) {
                return Ok(None);
            }
            let candidate = dir.join(ALIASES_FILE);
            if candidate.exists() {
                return Ok(Some(candidate));
            }
            if !dir.pop() {
                return Ok(None);
            }
        }
    }

    /// Load from a specific path.
    pub fn load(path: &Path) -> crate::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let project = toml::from_str(&data)?;
        Ok(project)
    }

    /// Save to a specific path.
    pub fn save(&self, path: &Path) -> crate::Result<()> {
        let data = toml::to_string(self)?;
        std::fs::write(path, data)?;
        Ok(())
    }

    pub fn remove_alias(&mut self, name: &str) -> crate::Result<()> {
        let key: AliasName = name.into();
        self.aliases
            .remove(&key)
            .ok_or_else(|| anyhow::anyhow!("Alias '{name}' not found in {ALIASES_FILE}"))?;
        Ok(())
    }

    /// Find the nearest `.aliases` file starting from the given directory.
    /// Checks `cwd` first, then walks up parent directories.
    pub fn find_local_path_in(cwd: &Path) -> Option<PathBuf> {
        let local = cwd.join(ALIASES_FILE);
        if local.exists() {
            return Some(local);
        }
        cwd.parent().and_then(|p| Self::find_path(p).ok().flatten())
    }

    /// Find the nearest `.aliases` file starting from the current directory.
    pub fn find_local_path() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        Self::find_local_path_in(&cwd)
    }

    /// Remove an alias from the nearest local `.aliases` file.
    /// Returns the path of the modified file.
    pub fn remove_from_local(name: &str) -> crate::Result<PathBuf> {
        let path =
            Self::find_local_path().ok_or_else(|| anyhow::anyhow!("No {ALIASES_FILE} found"))?;
        let mut project = Self::load(&path)?;
        project.remove_alias(name)?;
        project.save(&path)?;
        Ok(path)
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_load_aliases_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nb = \"./x.py build\"\nt = \"./x.py test\"\n",
        )
        .unwrap();

        let project = ProjectAliases::find(dir.path()).unwrap();
        assert!(project.is_some());
        let project = project.unwrap();
        assert_eq!(project.aliases.iter().count(), 2);
    }

    #[test]
    fn test_no_aliases_file() {
        let dir = tempfile::tempdir().unwrap();
        let project = ProjectAliases::find(dir.path()).unwrap();
        assert!(project.is_none());
    }

    #[test]
    fn test_invalid_toml_is_skipped() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".aliases"),
            "alias foo=\"bar\"\nthis is not toml",
        )
        .unwrap();

        let project = ProjectAliases::find(dir.path()).unwrap();
        assert!(project.is_none());
    }

    #[test]
    fn test_find_aliases_in_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("src").join("deep");
        fs::create_dir_all(&sub).unwrap();
        fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nb = \"make build\"\n",
        )
        .unwrap();

        let project = ProjectAliases::find(&sub).unwrap();
        assert!(project.is_some());
    }

    #[test]
    fn test_find_path_returns_location() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("src");
        fs::create_dir_all(&sub).unwrap();
        fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nb = \"make build\"\n",
        )
        .unwrap();

        let path = ProjectAliases::find_path(&sub).unwrap();
        assert_eq!(path.unwrap(), dir.path().join(".aliases"));
    }

    #[test]
    fn test_add_alias_and_save_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".aliases");

        let mut project = ProjectAliases::default();
        project.add_alias("t".to_string(), "cargo test".to_string(), false);
        project.save(&path).unwrap();

        let loaded = ProjectAliases::load(&path).unwrap();
        assert_eq!(loaded.aliases.iter().count(), 1);
    }
}
