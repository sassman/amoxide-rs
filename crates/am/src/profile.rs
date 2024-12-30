use dirs::config_dir;
use serde::Deserialize;

use crate::Result;

#[derive(Debug, Deserialize, Default)]
pub struct ProfileConfig {
    profiles: Vec<Profile>,
}

impl ProfileConfig {
    pub fn iter(&self) -> impl Iterator<Item = &Profile> {
        self.profiles.iter()
    }

    pub(crate) fn add_profile_with_inheritance(
        &self,
        name: &str,
        inherits: Option<&String>,
    ) -> Result<&Profile> {
        let name = name.to_string();
        let existing_profile = self.profiles.binary_search_by(|p1| p1.name.cmp(&name));
        if let Ok(i) = existing_profile {
            return Ok(self.profiles.get(i).unwrap());
        }

        todo!("search a profile and or add is WIP")
    }
}

impl ProfileConfig {
    pub fn load() -> Result<Self> {
        let profile_config_file = config_dir().unwrap().join("alias-manager/profiles.toml");
        if !profile_config_file.exists() {
            return Ok(Self::default());
        }

        let toml_str = std::fs::read_to_string(profile_config_file)?;
        let mut decoded: ProfileConfig = toml::from_str(&toml_str)?;
        decoded.profiles.sort();

        Ok(decoded)
    }
}

#[derive(Debug, Deserialize)]
pub struct Profile {
    pub name: String,
    pub inherits: Option<String>,
}

impl PartialEq<Profile> for Profile {
    fn eq(&self, other: &Profile) -> bool {
        self.name == other.name
    }
}

impl PartialOrd<Profile> for Profile {
    fn partial_cmp(&self, other: &Profile) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
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
        }
    }
}

impl Profile {
    pub fn new(name: String, inherits: Option<String>) -> Self {
        Self { name, inherits }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialization_profiles() {
        let toml_str = r#"
            [[profiles]]
            name = "default"

            [[profiles]]
            name = "work"
            inherits = "default"
        "#;

        let decoded: ProfileConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(decoded.profiles.len(), 2);
        assert_eq!(decoded.profiles[0].name, "default");
        assert_eq!(decoded.profiles[0].inherits, None);

        assert_eq!(decoded.profiles[1].name, "work");
        assert_eq!(decoded.profiles[1].inherits, Some("default".to_string()));
    }
}
