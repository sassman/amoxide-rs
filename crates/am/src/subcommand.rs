use std::collections::BTreeMap;

use anyhow::anyhow;
use log::warn;

/// A parsed subcommand alias.
///
/// Example: key `"jj:b:l"` with value `["branch", "list"]` gives:
///   program = "jj", short_subcommands = ["b", "l"], long_subcommands = ["branch", "list"]
#[derive(Debug, Clone, PartialEq)]
pub struct SubcommandEntry {
    pub program: String,
    pub short_subcommands: Vec<String>,
    pub long_subcommands: Vec<String>,
}

impl SubcommandEntry {
    /// Parse a colon-separated key and its expansion values into a SubcommandEntry.
    pub fn parse_key(key: &str, long_subcommands: Vec<String>) -> anyhow::Result<Self> {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() < 2 {
            return Err(anyhow!("Subcommand key must contain ':' (got '{key}')"));
        }
        let program = parts[0].to_string();
        let short_subcommands: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        if short_subcommands.iter().any(|s| s.is_empty()) {
            return Err(anyhow!("Subcommand segments cannot be empty in '{key}'"));
        }
        if short_subcommands.len() != long_subcommands.len() {
            return Err(anyhow!(
                "Subcommand count mismatch: {} short segments but {} expansions",
                short_subcommands.len(),
                long_subcommands.len()
            ));
        }

        Ok(Self {
            program,
            short_subcommands,
            long_subcommands,
        })
    }

    /// Reconstruct the TOML key from the entry.
    pub fn to_key(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.short_subcommands.iter().map(|s| s.as_str()))
            .collect::<Vec<_>>()
            .join(":")
    }
}

/// Storage type for subcommand aliases. Key is the full colon-joined string
/// (e.g., "jj:b:l"), value is the Vec of long subcommands (e.g., ["branch", "list"]).
pub type SubcommandSet = BTreeMap<String, Vec<String>>;

/// Group subcommand entries by program name.
pub fn group_by_program(set: &SubcommandSet) -> BTreeMap<String, Vec<SubcommandEntry>> {
    let mut groups: BTreeMap<String, Vec<SubcommandEntry>> = BTreeMap::new();
    for (key, values) in set {
        match SubcommandEntry::parse_key(key, values.clone()) {
            Ok(entry) => {
                groups.entry(entry.program.clone()).or_default().push(entry);
            }
            Err(e) => {
                warn!("Skipping invalid subcommand alias '{key}': {e}");
            }
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_level() {
        let entry = SubcommandEntry::parse_key("jj:ab", vec!["abandon".into()]).unwrap();
        assert_eq!(entry.program, "jj");
        assert_eq!(entry.short_subcommands, vec!["ab"]);
        assert_eq!(entry.long_subcommands, vec!["abandon"]);
    }

    #[test]
    fn parse_multi_level() {
        let entry =
            SubcommandEntry::parse_key("jj:b:l", vec!["branch".into(), "list".into()]).unwrap();
        assert_eq!(entry.program, "jj");
        assert_eq!(entry.short_subcommands, vec!["b", "l"]);
        assert_eq!(entry.long_subcommands, vec!["branch", "list"]);
    }

    #[test]
    fn parse_rejects_mismatched_counts() {
        let result = SubcommandEntry::parse_key("jj:b:l", vec!["branch".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_no_colon() {
        let result = SubcommandEntry::parse_key("jj", vec!["abandon".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_empty_segment() {
        let result = SubcommandEntry::parse_key("jj::ab", vec!["abandon".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn to_key_roundtrips() {
        let entry =
            SubcommandEntry::parse_key("jj:b:l", vec!["branch".into(), "list".into()]).unwrap();
        assert_eq!(entry.to_key(), "jj:b:l");
    }

    #[test]
    fn group_by_program_groups_correctly() {
        let mut set = SubcommandSet::new();
        set.insert("jj:ab".into(), vec!["abandon".into()]);
        set.insert("jj:b:l".into(), vec!["branch".into(), "list".into()]);
        set.insert("git:co".into(), vec!["checkout".into()]);

        let groups = group_by_program(&set);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups["jj"].len(), 2);
        assert_eq!(groups["git"].len(), 1);
    }

    #[test]
    fn group_by_program_empty() {
        let set = SubcommandSet::new();
        let groups = group_by_program(&set);
        assert!(groups.is_empty());
    }

    #[test]
    fn group_by_program_skips_invalid_entries() {
        let mut set = SubcommandSet::new();
        set.insert("jj:ab".into(), vec!["abandon".into()]);
        // mismatched counts — invalid
        set.insert("jj:b:l".into(), vec!["branch".into()]);
        // no colon — invalid
        set.insert("bad".into(), vec!["whatever".into()]);

        let groups = group_by_program(&set);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups["jj"].len(), 1);
        assert_eq!(groups["jj"][0].short_subcommands, vec!["ab"]);
    }
}
