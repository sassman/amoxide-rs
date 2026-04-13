use std::fmt::Display;

use log::info;
use serde::{Deserialize, Serialize};

use crate::dirs::config_dir;
use crate::subcommand::{group_by_program, SubcommandSet};
use crate::{AliasDetail, AliasName, AliasSet, Result, TomlAlias};

/// A collection of aliases (regular and/or subcommand) that can report its
/// size and produce a compact human-readable summary.
pub trait AliasCollection {
    /// Returns `true` when both regular aliases and subcommand entries are empty.
    fn is_empty(&self) -> bool;

    /// Total count: regular aliases + subcommand entries.
    fn len(&self) -> usize;

    /// Compact summary for use in activation messages, e.g.:
    /// - `"gs, ct"` – only regular aliases
    /// - `"◆ jj (ab, b l)"` – only subcommand groups
    /// - `"gs, ◆ jj (ab)"` – mixed
    fn short_list(&self) -> String;
}

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
    pub fn resolve_active_aliases(&self, profile_names: &[impl AsRef<str>]) -> AliasSet {
        let mut resolved = AliasSet::default();
        for name in profile_names {
            if let Some(profile) = self.get_profile_by_name(name.as_ref()) {
                for (alias_name, alias) in profile.aliases.iter() {
                    resolved.insert(alias_name.clone(), alias.clone());
                }
            }
        }
        resolved
    }

    /// Resolve the merged subcommand set for multiple active profiles.
    pub fn resolve_active_subcommands(&self, profile_names: &[impl AsRef<str>]) -> SubcommandSet {
        let mut resolved = SubcommandSet::new();
        for name in profile_names {
            if let Some(profile) = self.get_profile_by_name(name.as_ref()) {
                for (key, values) in &profile.subcommands {
                    resolved.insert(key.clone(), values.clone());
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

    pub fn merge_profile(&mut self, profile: Profile) {
        if let Some(existing) = self.get_profile_by_name_mut(&profile.name) {
            for (name, alias) in profile.aliases.iter() {
                existing.aliases.insert(name.clone(), alias.clone());
            }
        } else {
            self.profiles.push(profile);
            self.profiles.sort();
        }
    }

    pub fn to_vec(&self) -> Vec<Profile> {
        self.profiles.clone()
    }

    pub fn from_profiles(profiles: Vec<Profile>) -> Self {
        Self { profiles }
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

    pub fn load_from(config_dir: &std::path::Path) -> Result<Self> {
        let profile_config_file = config_dir.join(CONFIG_FILE);
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
    #[serde(default, skip_serializing_if = "SubcommandSet::is_empty")]
    pub subcommands: SubcommandSet,
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
            subcommands: Default::default(),
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

    pub fn add_subcommand(&mut self, key: String, long_subcommands: Vec<String>) {
        self.subcommands.insert(key, long_subcommands);
    }

    pub fn remove_subcommand(&mut self, key: &str) -> Result<()> {
        self.subcommands
            .remove(key)
            .ok_or_else(|| anyhow::anyhow!("Subcommand alias '{key}' not found"))?;
        Ok(())
    }
}

impl AliasCollection for Profile {
    fn is_empty(&self) -> bool {
        self.aliases.is_empty() && self.subcommands.is_empty()
    }

    fn len(&self) -> usize {
        self.aliases.len() + self.subcommands.len()
    }

    fn short_list(&self) -> String {
        let mut parts: Vec<String> = self
            .aliases
            .iter()
            .map(|(k, _)| k.as_ref().to_string())
            .collect();

        let groups = group_by_program(&self.subcommands);
        for (program, entries) in &groups {
            // Each entry's short_subcommands are space-joined; entries within
            // the same program are comma-separated.
            let subcommand_tokens: Vec<String> = entries
                .iter()
                .map(|e| e.short_subcommands.join(" "))
                .collect();
            parts.push(format!("◆ {} ({})", program, subcommand_tokens.join(", ")));
        }

        parts.join(", ")
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

        let resolved = config.resolve_active_aliases(&["git"]);
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

        let resolved = config.resolve_active_aliases(&["git", "rust"]);
        assert_eq!(resolved.iter().count(), 3); // gs, ct, t (rust overrides git)
        assert!(resolved.iter().any(|(n, _)| n.as_ref() == "gs"));
        assert!(resolved.iter().any(|(n, _)| n.as_ref() == "ct"));
        // "t" should be "cargo test" (from rust, which comes later)
        let t_alias = resolved.iter().find(|(n, _)| n.as_ref() == "t").unwrap();
        assert_eq!(t_alias.1.command(), "cargo test");
    }

    #[test]
    fn test_merge_profile_new() {
        let mut config = ProfileConfig::default();
        let mut profile = Profile::new("git".into());
        profile
            .add_alias("gs".into(), "git status".into(), false)
            .unwrap();
        config.merge_profile(profile);
        assert_eq!(config.len(), 1);
        assert!(config.get_profile_by_name("git").is_some());
    }

    #[test]
    fn test_merge_profile_existing() {
        let mut config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#})
        .unwrap();
        let mut incoming = Profile::new("git".into());
        incoming
            .add_alias("gp".into(), "git push".into(), false)
            .unwrap();
        incoming
            .add_alias("gs".into(), "git status -s".into(), false)
            .unwrap();
        config.merge_profile(incoming);
        assert_eq!(config.len(), 1);
        let profile = config.get_profile_by_name("git").unwrap();
        assert_eq!(profile.aliases.len(), 2);
        assert_eq!(
            profile
                .aliases
                .get(&AliasName::from("gs"))
                .unwrap()
                .command(),
            "git status -s"
        );
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

        let resolved = config.resolve_active_aliases(&[] as &[&str]);
        assert!(resolved.is_empty());
    }

    #[test]
    fn profile_config_load_from_reads_from_given_dir() {
        let dir = tempfile::tempdir().unwrap();
        let toml = "[[profiles]]\nname = \"git\"\n";
        std::fs::write(dir.path().join("profiles.toml"), toml).unwrap();

        let config = ProfileConfig::load_from(dir.path()).unwrap();
        assert_eq!(config.len(), 1);
        assert!(config.get_profile_by_name("git").is_some());
    }

    #[test]
    fn profile_config_load_from_returns_default_when_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        let config = ProfileConfig::load_from(dir.path()).unwrap();
        assert!(config.is_empty());
    }

    // --- AliasCollection trait ---

    #[test]
    fn alias_collection_is_empty_for_empty_profile() {
        let profile = Profile::new("empty".into());
        assert!(profile.is_empty());
    }

    #[test]
    fn alias_collection_is_not_empty_with_regular_alias() {
        let mut profile = Profile::new("git".into());
        profile
            .add_alias("gs".into(), "git status".into(), false)
            .unwrap();
        assert!(!profile.is_empty());
    }

    #[test]
    fn alias_collection_is_not_empty_with_subcommand() {
        let mut profile = Profile::new("jj".into());
        profile.add_subcommand("jj:ab".into(), vec!["abandon".into()]);
        assert!(!profile.is_empty());
    }

    #[test]
    fn alias_collection_len_counts_both() {
        let mut profile = Profile::new("mixed".into());
        profile
            .add_alias("gs".into(), "git status".into(), false)
            .unwrap();
        profile.add_subcommand("jj:ab".into(), vec!["abandon".into()]);
        profile.add_subcommand("jj:b:l".into(), vec!["branch".into(), "list".into()]);
        assert_eq!(profile.len(), 3); // 1 alias + 2 subcommand entries
    }

    #[test]
    fn alias_collection_short_list_aliases_only() {
        let config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
            ct = "cargo test"
        "#})
        .unwrap();
        let profile = config.get_profile_by_name("git").unwrap();
        let list = profile.short_list();
        // Both names should appear; order is BTreeMap (alphabetical)
        assert!(list.contains("gs"), "expected 'gs' in '{list}'");
        assert!(list.contains("ct"), "expected 'ct' in '{list}'");
    }

    #[test]
    fn alias_collection_short_list_subcommands_only() {
        let mut profile = Profile::new("jj".into());
        profile.add_subcommand("jj:ab".into(), vec!["abandon".into()]);
        profile.add_subcommand("jj:b:l".into(), vec!["branch".into(), "list".into()]);
        let list = profile.short_list();
        assert!(list.contains("◆ jj"), "expected '◆ jj' in '{list}'");
        assert!(list.contains("ab"), "expected 'ab' in '{list}'");
        assert!(list.contains("b l"), "expected 'b l' in '{list}'");
    }

    #[test]
    fn alias_collection_short_list_mixed() {
        let config: ProfileConfig = toml::from_str(indoc! {r#"
            [[profiles]]
            name = "mixed"
            [profiles.aliases]
            gs = "git status"
            [profiles.subcommands]
            "jj:ab" = ["abandon"]
        "#})
        .unwrap();
        let profile = config.get_profile_by_name("mixed").unwrap();
        let list = profile.short_list();
        assert!(list.contains("gs"), "expected 'gs' in '{list}'");
        assert!(list.contains("◆ jj"), "expected '◆ jj' in '{list}'");
    }

    #[test]
    fn alias_collection_short_list_empty_profile() {
        let profile = Profile::new("empty".into());
        assert_eq!(profile.short_list(), "");
    }
}
