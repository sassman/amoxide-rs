use std::path::Path;

use serde::Deserialize;

use crate::AliasSet;

const ALIASES_FILE: &str = ".aliases";

#[derive(Debug, Deserialize)]
pub struct ProjectAliases {
    pub aliases: AliasSet,
}

impl ProjectAliases {
    /// Walk up from `start_dir` looking for a `.aliases` file.
    /// Stops at the user's home directory to avoid loading stray files from `/` or `/home`.
    pub fn find(start_dir: &Path) -> crate::Result<Option<Self>> {
        let home = dirs::home_dir();
        let mut dir = start_dir.to_path_buf();
        loop {
            let candidate = dir.join(ALIASES_FILE);
            if candidate.exists() {
                let data = std::fs::read_to_string(candidate)?;
                let project: ProjectAliases = toml::from_str(&data)?;
                return Ok(Some(project));
            }
            // Stop at home directory boundary
            if home.as_ref().is_some_and(|h| h == &dir) {
                return Ok(None);
            }
            if !dir.pop() {
                return Ok(None);
            }
        }
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
}
