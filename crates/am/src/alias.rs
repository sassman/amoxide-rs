use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AliasName<T: AsRef<str> = String>(T);

impl From<&str> for AliasName {
    fn from(name: &str) -> Self {
        Self(name.into())
    }
}

impl From<String> for AliasName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl AsRef<str> for AliasName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for AliasName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
pub struct AliasSet(BTreeMap<AliasName, TomlAlias>);

impl AsRef<BTreeMap<AliasName, TomlAlias>> for AliasSet {
    fn as_ref(&self) -> &BTreeMap<AliasName, TomlAlias> {
        &self.0
    }
}

impl AsMut<BTreeMap<AliasName, TomlAlias>> for AliasSet {
    fn as_mut(&mut self) -> &mut BTreeMap<AliasName, TomlAlias> {
        &mut self.0
    }
}

impl AliasSet {
    pub fn iter(&self) -> impl Iterator<Item = (&AliasName, &TomlAlias)> {
        self.as_ref().iter()
    }

    pub fn insert(&mut self, name: AliasName, alias: TomlAlias) -> Option<TomlAlias> {
        self.as_mut().insert(name, alias)
    }

    pub fn remove(&mut self, name: &AliasName) -> Option<TomlAlias> {
        self.as_mut().remove(name)
    }

    pub fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }

    pub fn contains_key(&self, name: &AliasName) -> bool {
        self.as_ref().contains_key(name)
    }

    pub fn get(&self, name: &AliasName) -> Option<&TomlAlias> {
        self.as_ref().get(name)
    }

    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    pub fn merge_check(&self, incoming: &AliasSet) -> MergeResult {
        let mut new_aliases = AliasSet::default();
        let mut conflicts = Vec::new();

        for (name, incoming_alias) in incoming.iter() {
            match self.get(name) {
                None => {
                    new_aliases.insert(name.clone(), incoming_alias.clone());
                }
                Some(existing_alias) => {
                    if existing_alias.command() != incoming_alias.command() {
                        conflicts.push(AliasConflict {
                            name: name.clone(),
                            current: existing_alias.clone(),
                            incoming: incoming_alias.clone(),
                        });
                    }
                }
            }
        }

        MergeResult {
            new_aliases,
            conflicts,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum TomlAlias {
    Command(String),
    Detailed(AliasDetail),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct AliasDetail {
    pub command: String,
    pub description: Option<String>,
    #[serde(default)]
    pub raw: bool,
}

#[derive(Debug)]
pub struct AliasEntry<'a> {
    pub name: &'a str,
    pub command: &'a str,
    pub raw: bool,
}

impl TomlAlias {
    pub fn as_entry<'a>(&'a self, name: &'a str) -> AliasEntry<'a> {
        AliasEntry {
            name,
            command: self.command(),
            raw: matches!(self, TomlAlias::Detailed(d) if d.raw),
        }
    }

    pub fn command(&self) -> &str {
        match self {
            TomlAlias::Command(cmd) => cmd,
            TomlAlias::Detailed(d) => &d.command,
        }
    }
}

#[derive(Debug)]
pub struct MergeResult {
    pub new_aliases: AliasSet,
    pub conflicts: Vec<AliasConflict>,
}

#[derive(Debug)]
pub struct AliasConflict {
    pub name: AliasName,
    pub current: TomlAlias,
    pub incoming: TomlAlias,
}

#[derive(Debug)]
pub enum AliasDisplayFilter {
    Used,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_entry_simple() {
        let alias = TomlAlias::Command("git status".to_string());
        let entry = alias.as_entry("gs");
        assert_eq!(entry.name, "gs");
        assert_eq!(entry.command, "git status");
        assert!(!entry.raw);
    }

    #[test]
    fn test_as_entry_detailed_raw() {
        let alias = TomlAlias::Detailed(AliasDetail {
            command: "awk '{print {{1}}}'".to_string(),
            description: None,
            raw: true,
        });
        let entry = alias.as_entry("my-awk");
        assert_eq!(entry.command, "awk '{print {{1}}}'");
        assert!(entry.raw);
    }

    #[test]
    fn test_as_entry_detailed_not_raw() {
        let alias = TomlAlias::Detailed(AliasDetail {
            command: "echo hi".to_string(),
            description: Some("greeting".to_string()),
            raw: false,
        });
        let entry = alias.as_entry("hi");
        assert!(!entry.raw);
    }

    #[test]
    fn test_command_accessor() {
        assert_eq!(TomlAlias::Command("ls".to_string()).command(), "ls");
        assert_eq!(
            TomlAlias::Detailed(AliasDetail {
                command: "echo hi".to_string(),
                description: None,
                raw: false,
            })
            .command(),
            "echo hi"
        );
    }

    #[test]
    fn test_toml_roundtrip_detailed_with_raw() {
        #[derive(Debug, serde::Deserialize)]
        struct Wrapper {
            aliases: std::collections::BTreeMap<AliasName, TomlAlias>,
        }

        let toml_str = r#"
[aliases]
gs = "git status"
my-awk = { command = "awk '{print {{1}}}'", raw = true }
fancy = { command = "echo hi", description = "A fancy alias" }
"#;
        let parsed: Wrapper = toml::from_str(toml_str).unwrap();
        assert_eq!(parsed.aliases.len(), 3);

        match &parsed.aliases[&AliasName::from("gs")] {
            TomlAlias::Command(cmd) => assert_eq!(cmd, "git status"),
            _ => panic!("expected Command"),
        }

        match &parsed.aliases[&AliasName::from("my-awk")] {
            TomlAlias::Detailed(d) => {
                assert!(d.raw);
                assert_eq!(d.command, "awk '{print {{1}}}'");
            }
            _ => panic!("expected Detailed"),
        }

        match &parsed.aliases[&AliasName::from("fancy")] {
            TomlAlias::Detailed(d) => {
                assert!(!d.raw);
                assert_eq!(d.description.as_deref(), Some("A fancy alias"));
            }
            _ => panic!("expected Detailed"),
        }
    }

    #[test]
    fn test_merge_check_all_new() {
        let existing = AliasSet::default();
        let mut incoming = AliasSet::default();
        incoming.insert("gs".into(), TomlAlias::Command("git status".into()));
        incoming.insert("gp".into(), TomlAlias::Command("git push".into()));
        let result = existing.merge_check(&incoming);
        assert_eq!(result.new_aliases.len(), 2);
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_merge_check_no_op_same_command() {
        let mut existing = AliasSet::default();
        existing.insert("gs".into(), TomlAlias::Command("git status".into()));
        let mut incoming = AliasSet::default();
        incoming.insert("gs".into(), TomlAlias::Command("git status".into()));
        let result = existing.merge_check(&incoming);
        assert!(result.new_aliases.is_empty());
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_merge_check_conflict() {
        let mut existing = AliasSet::default();
        existing.insert("gs".into(), TomlAlias::Command("git status --short".into()));
        let mut incoming = AliasSet::default();
        incoming.insert("gs".into(), TomlAlias::Command("git status".into()));
        let result = existing.merge_check(&incoming);
        assert!(result.new_aliases.is_empty());
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].name.as_ref(), "gs");
        assert_eq!(result.conflicts[0].current.command(), "git status --short");
        assert_eq!(result.conflicts[0].incoming.command(), "git status");
    }

    #[test]
    fn test_merge_check_mixed() {
        let mut existing = AliasSet::default();
        existing.insert("gs".into(), TomlAlias::Command("git status --short".into()));
        let mut incoming = AliasSet::default();
        incoming.insert("gs".into(), TomlAlias::Command("git status".into()));
        incoming.insert("gp".into(), TomlAlias::Command("git push".into()));
        let result = existing.merge_check(&incoming);
        assert_eq!(result.new_aliases.len(), 1);
        assert_eq!(result.conflicts.len(), 1);
    }

    #[test]
    fn test_merge_check_empty_incoming() {
        let mut existing = AliasSet::default();
        existing.insert("gs".into(), TomlAlias::Command("git status".into()));
        let result = existing.merge_check(&AliasSet::default());
        assert!(result.new_aliases.is_empty());
        assert!(result.conflicts.is_empty());
    }
}
