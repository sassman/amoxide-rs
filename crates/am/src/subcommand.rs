use std::collections::BTreeMap;

use anyhow::anyhow;
use log::warn;
use serde::{Deserialize, Serialize};

/// Validates that a name is a safe shell identifier.
///
/// Accepted characters: ASCII alphanumeric, `-`, `_`, `.`.
/// The name must also be non-empty. This set is intentionally conservative:
/// shell function names and `case` labels built from these characters will
/// never produce broken syntax regardless of the surrounding shell script.
pub struct ProgramValidator;

impl ProgramValidator {
    /// Return `Ok(())` when `name` is a valid shell-safe identifier, or an
    /// `Err` with a descriptive message otherwise.
    pub fn validate(name: &str, label: &str) -> anyhow::Result<()> {
        if name.is_empty() {
            return Err(anyhow!(
                "invalid {label} '': must be a non-empty shell identifier"
            ));
        }
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(anyhow!(
                "invalid {label} '{name}': must contain only alphanumeric characters, '-', '_', or '.'"
            ));
        }
        Ok(())
    }
}

/// A parsed subcommand alias.
///
/// Example: key `"jj:b:l"` with value `["branch", "list"]` gives:
///   program = "jj", short_subcommands = ["b", "l"], long_subcommands = ["branch", "list"]
#[derive(Debug, Clone, PartialEq)]
pub struct SubcommandEntry {
    pub program: String,
    pub short_subcommands: Vec<String>,
    pub long_subcommands: Vec<String>,
    pub description: Option<String>,
}

impl SubcommandEntry {
    /// Parse a colon-separated key and its expansion values into a SubcommandEntry.
    pub fn parse_key(
        key: &str,
        long_subcommands: Vec<String>,
        description: Option<String>,
    ) -> anyhow::Result<Self> {
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() < 2 {
            return Err(anyhow!("Subcommand key must contain ':' (got '{key}')"));
        }
        let program = parts[0].to_string();
        let short_subcommands: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

        ProgramValidator::validate(&program, "program name")?;

        if short_subcommands.iter().any(|s| s.is_empty()) {
            return Err(anyhow!("Subcommand segments cannot be empty in '{key}'"));
        }
        for token in &short_subcommands {
            ProgramValidator::validate(token, "short subcommand token")?;
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
            description,
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

/// The TOML wire form of one subcommand entry. Untagged so the existing
/// array form `"jj:ab" = ["abandon"]` continues to parse unchanged, while
/// the detailed inline-table form `{ expansions = [...], description = "..." }`
/// becomes available for entries that want to carry metadata.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(untagged)]
pub enum TomlSubcommand {
    Expansion(Vec<String>),
    Detailed(SubcommandDetail),
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SubcommandDetail {
    pub expansions: Vec<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_normalized_description"
    )]
    pub description: Option<String>,
}

fn deserialize_normalized_description<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: Option<String> = Option::deserialize(d)?;
    Ok(raw.and_then(|s| crate::normalize_description(&s)))
}

impl TomlSubcommand {
    /// View the long-form tokens regardless of which variant we hold.
    pub fn expansions(&self) -> &[String] {
        match self {
            TomlSubcommand::Expansion(v) => v,
            TomlSubcommand::Detailed(d) => &d.expansions,
        }
    }
}

impl crate::Described for TomlSubcommand {
    fn description(&self) -> Option<&str> {
        match self {
            TomlSubcommand::Expansion(_) => None,
            TomlSubcommand::Detailed(d) => d.description.as_deref(),
        }
    }
}

/// Storage type for subcommand aliases. Key is the full colon-joined string
/// (e.g., `"jj:b:l"`), value is `TomlSubcommand` carrying the expansion plus
/// optional description.
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct SubcommandSet(BTreeMap<String, TomlSubcommand>);

impl AsRef<BTreeMap<String, TomlSubcommand>> for SubcommandSet {
    fn as_ref(&self) -> &BTreeMap<String, TomlSubcommand> {
        &self.0
    }
}

impl AsMut<BTreeMap<String, TomlSubcommand>> for SubcommandSet {
    fn as_mut(&mut self) -> &mut BTreeMap<String, TomlSubcommand> {
        &mut self.0
    }
}

impl SubcommandSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// Kept as a method so `#[serde(skip_serializing_if = "SubcommandSet::is_empty")]`
    /// can reference it directly. All other access should go through `AsRef`/`AsMut`.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Group the entries by program name. Each entry whose key fails to parse
    /// (mismatched short/long count, empty segment, etc.) is skipped with a
    /// warning. Returns a `BTreeMap<program, Vec<SubcommandEntry>>` so callers
    /// can iterate per-program.
    pub fn group_by_program(&self) -> BTreeMap<String, Vec<SubcommandEntry>> {
        let mut groups: BTreeMap<String, Vec<SubcommandEntry>> = BTreeMap::new();
        for (key, entry) in self {
            let description = crate::Described::description(entry).map(str::to_string);
            match SubcommandEntry::parse_key(key, entry.expansions().to_vec(), description) {
                Ok(parsed) => {
                    groups.entry(parsed.program.clone()).or_default().push(parsed);
                }
                Err(e) => {
                    warn!("Skipping invalid subcommand alias '{key}': {e}");
                }
            }
        }
        groups
    }
}

impl<'a> IntoIterator for &'a SubcommandSet {
    type Item = (&'a String, &'a TomlSubcommand);
    type IntoIter = std::collections::btree_map::Iter<'a, String, TomlSubcommand>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for SubcommandSet {
    type Item = (String, TomlSubcommand);
    type IntoIter = std::collections::btree_map::IntoIter<String, TomlSubcommand>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(String, TomlSubcommand)> for SubcommandSet {
    fn from_iter<I: IntoIterator<Item = (String, TomlSubcommand)>>(iter: I) -> Self {
        Self(BTreeMap::from_iter(iter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- ProgramValidator ---

    #[test]
    fn validator_rejects_empty_program() {
        let err = ProgramValidator::validate("", "program name").unwrap_err();
        assert!(
            err.to_string().contains("non-empty"),
            "expected 'non-empty' in error, got: {err}"
        );
    }

    #[test]
    fn validator_rejects_glob_chars_in_program() {
        let err = ProgramValidator::validate("*jj", "program name").unwrap_err();
        assert!(
            err.to_string().contains("*jj"),
            "expected name in error, got: {err}"
        );
    }

    #[test]
    fn validator_accepts_hyphenated_program() {
        ProgramValidator::validate("jj-cli", "program name").unwrap();
    }

    #[test]
    fn validator_accepts_dotted_program() {
        ProgramValidator::validate("kubectl.exe", "program name").unwrap();
    }

    #[test]
    fn validator_rejects_question_mark_in_token() {
        let err = ProgramValidator::validate("ab?", "short subcommand token").unwrap_err();
        assert!(
            err.to_string().contains("ab?"),
            "expected name in error, got: {err}"
        );
    }

    #[test]
    fn validator_accepts_normal_short_token() {
        ProgramValidator::validate("ab", "short subcommand token").unwrap();
    }

    // --- SubcommandEntry::parse_key ---

    #[test]
    fn parse_rejects_empty_program_name() {
        let result = SubcommandEntry::parse_key(":ab", vec!["abandon".into()], None);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("non-empty"), "unexpected message: {msg}");
    }

    #[test]
    fn parse_rejects_invalid_program_name() {
        let result = SubcommandEntry::parse_key("*jj:ab", vec!["abandon".into()], None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_invalid_short_token() {
        let result = SubcommandEntry::parse_key("jj:ab?", vec!["abandon".into()], None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_single_level() {
        let entry =
            SubcommandEntry::parse_key("jj:ab", vec!["abandon".into()], None).unwrap();
        assert_eq!(entry.program, "jj");
        assert_eq!(entry.short_subcommands, vec!["ab"]);
        assert_eq!(entry.long_subcommands, vec!["abandon"]);
        assert_eq!(entry.description, None);
    }

    #[test]
    fn parse_multi_level() {
        let entry = SubcommandEntry::parse_key(
            "jj:b:l",
            vec!["branch".into(), "list".into()],
            None,
        )
        .unwrap();
        assert_eq!(entry.program, "jj");
        assert_eq!(entry.short_subcommands, vec!["b", "l"]);
        assert_eq!(entry.long_subcommands, vec!["branch", "list"]);
        assert_eq!(entry.description, None);
    }

    #[test]
    fn parse_rejects_mismatched_counts() {
        let result = SubcommandEntry::parse_key("jj:b:l", vec!["branch".into()], None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_no_colon() {
        let result = SubcommandEntry::parse_key("jj", vec!["abandon".into()], None);
        assert!(result.is_err());
    }

    #[test]
    fn parse_rejects_empty_segment() {
        let result = SubcommandEntry::parse_key("jj::ab", vec!["abandon".into()], None);
        assert!(result.is_err());
    }

    #[test]
    fn to_key_roundtrips() {
        let entry = SubcommandEntry::parse_key(
            "jj:b:l",
            vec!["branch".into(), "list".into()],
            None,
        )
        .unwrap();
        assert_eq!(entry.to_key(), "jj:b:l");
    }

    #[test]
    fn group_by_program_groups_correctly() {
        let mut set = SubcommandSet::new();
        set.as_mut()
            .insert("jj:ab".into(), TomlSubcommand::Expansion(vec!["abandon".into()]));
        set.as_mut().insert(
            "jj:b:l".into(),
            TomlSubcommand::Expansion(vec!["branch".into(), "list".into()]),
        );
        set.as_mut()
            .insert("git:co".into(), TomlSubcommand::Expansion(vec!["checkout".into()]));

        let groups = set.group_by_program();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups["jj"].len(), 2);
        assert_eq!(groups["git"].len(), 1);
    }

    #[test]
    fn group_by_program_empty() {
        let set = SubcommandSet::new();
        let groups = set.group_by_program();
        assert!(groups.is_empty());
    }

    #[test]
    fn group_by_program_skips_invalid_entries() {
        let mut set = SubcommandSet::new();
        set.as_mut()
            .insert("jj:ab".into(), TomlSubcommand::Expansion(vec!["abandon".into()]));
        // mismatched counts — invalid
        set.as_mut()
            .insert("jj:b:l".into(), TomlSubcommand::Expansion(vec!["branch".into()]));
        // no colon — invalid
        set.as_mut()
            .insert("bad".into(), TomlSubcommand::Expansion(vec!["whatever".into()]));

        let groups = set.group_by_program();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups["jj"].len(), 1);
        assert_eq!(groups["jj"][0].short_subcommands, vec!["ab"]);
    }

    // --- SubcommandSet newtype API ---

    #[test]
    fn subcommandset_basic_ops_via_as_ref_as_mut() {
        let mut set = SubcommandSet::new();
        assert!(set.is_empty());

        set.as_mut()
            .insert("jj:ab".into(), TomlSubcommand::Expansion(vec!["abandon".into()]));
        assert_eq!(set.as_ref().len(), 1);
        assert!(set.as_ref().contains_key("jj:ab"));
        assert_eq!(
            set.as_ref().get("jj:ab"),
            Some(&TomlSubcommand::Expansion(vec!["abandon".to_string()]))
        );

        let removed = set.as_mut().remove("jj:ab");
        assert_eq!(
            removed,
            Some(TomlSubcommand::Expansion(vec!["abandon".to_string()]))
        );
        assert!(set.is_empty());
    }

    #[test]
    fn subcommandset_iteration_via_into_iterator() {
        let set: SubcommandSet = [
            (
                "a:x".to_string(),
                TomlSubcommand::Expansion(vec!["one".to_string()]),
            ),
            (
                "b:y".to_string(),
                TomlSubcommand::Expansion(vec!["two".to_string()]),
            ),
        ]
        .into_iter()
        .collect();

        // IntoIterator for &SubcommandSet lets for-loops work directly.
        let keys: Vec<&str> = (&set).into_iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, vec!["a:x", "b:y"]);

        // Owning IntoIterator yields (String, TomlSubcommand).
        let owned: Vec<(String, TomlSubcommand)> = set.into_iter().collect();
        assert_eq!(owned.len(), 2);
    }

    #[test]
    fn subcommandset_serde_transparent() {
        let set: SubcommandSet = [
            (
                "jj:ab".to_string(),
                TomlSubcommand::Expansion(vec!["abandon".to_string()]),
            ),
            (
                "jj:b:l".to_string(),
                TomlSubcommand::Expansion(vec!["branch".to_string(), "list".to_string()]),
            ),
        ]
        .into_iter()
        .collect();

        // Serializes as a plain map (not as a tuple-struct wrapper).
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Wrapper {
            subcommands: SubcommandSet,
        }
        let toml_str = toml::to_string(&Wrapper { subcommands: set }).unwrap();
        assert!(toml_str.contains("[subcommands]"));
        assert!(toml_str.contains("\"jj:ab\" = [\"abandon\"]"));

        let parsed: Wrapper = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.subcommands.as_ref().len(), 2);
        assert_eq!(
            parsed.subcommands.as_ref()["jj:ab"],
            TomlSubcommand::Expansion(vec!["abandon".to_string()])
        );
    }

    #[test]
    fn subcommandset_serde_round_trip_detailed_form() {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Wrapper {
            subcommands: SubcommandSet,
        }
        let toml_str = r#"
[subcommands]
"jj:ab" = ["abandon"]
"jj:b:l" = { expansions = ["branch", "list"], description = "list branches" }
"#;
        let parsed: Wrapper = toml::from_str(toml_str).unwrap();
        let map = parsed.subcommands.as_ref();
        assert_eq!(map.len(), 2);
        match &map["jj:ab"] {
            TomlSubcommand::Expansion(v) => assert_eq!(v, &vec!["abandon".to_string()]),
            _ => panic!("expected Expansion"),
        }
        match &map["jj:b:l"] {
            TomlSubcommand::Detailed(d) => {
                assert_eq!(d.expansions, vec!["branch", "list"]);
                assert_eq!(d.description.as_deref(), Some("list branches"));
            }
            _ => panic!("expected Detailed"),
        }
    }

    #[test]
    fn subcommandset_serde_empty_description_field_normalises_to_none() {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Wrapper {
            subcommands: SubcommandSet,
        }
        let toml_str = r#"
[subcommands]
"jj:ab" = { expansions = ["abandon"], description = "" }
"#;
        let parsed: Wrapper = toml::from_str(toml_str).unwrap();
        match &parsed.subcommands.as_ref()["jj:ab"] {
            TomlSubcommand::Detailed(d) => assert_eq!(d.description, None),
            _ => panic!("expected Detailed"),
        }
    }

    #[test]
    fn subcommandset_serialises_simple_form_when_no_description() {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Wrapper {
            subcommands: SubcommandSet,
        }
        let mut set = SubcommandSet::new();
        set.as_mut()
            .insert("jj:ab".into(), TomlSubcommand::Expansion(vec!["abandon".into()]));
        let toml_str = toml::to_string(&Wrapper { subcommands: set }).unwrap();
        assert!(toml_str.contains(r#""jj:ab" = ["abandon"]"#));
        assert!(!toml_str.contains("description"));
    }
}
