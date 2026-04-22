use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::alias::{AliasName, AliasSet, TomlAlias};
use crate::subcommand::{SubcommandEntry, SubcommandSet};

#[derive(Debug, Clone, PartialEq)]
pub enum EntryKind {
    Alias(TomlAlias),
    SubcommandWrapper {
        program: String,
        entries: Vec<SubcommandEntry>,
        base_cmd: Option<String>,
    },
    /// Per-key subcommand entry tracked in `_AM_SUBCOMMANDS` for fine-grained
    /// change detection. Never emitted as shell code — the program-level
    /// `SubcommandWrapper` is the shell-visible unit.
    SubcommandKey {
        longs: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveEntry {
    pub name: String,
    pub kind: EntryKind,
    pub hash: String,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PrecedenceDiff {
    pub added: Vec<EffectiveEntry>,
    pub changed: Vec<EffectiveEntry>,
    pub removed: Vec<String>,
    pub unchanged: Vec<EffectiveEntry>,
}

#[derive(Debug, Default)]
pub struct Precedence {
    global_aliases: AliasSet,
    global_subcommands: SubcommandSet,
    profile_aliases: AliasSet,
    profile_subcommands: SubcommandSet,
    project_aliases: AliasSet,
    project_subcommands: SubcommandSet,
    shell_alias_state: BTreeMap<String, Option<String>>,
    shell_subcmd_state: BTreeMap<String, Option<String>>,
    external_functions: HashSet<String>,
    external_aliases: HashSet<String>,
}

impl Precedence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_global(mut self, aliases: &AliasSet, subs: &SubcommandSet) -> Self {
        self.global_aliases = aliases.clone();
        self.global_subcommands = subs.clone();
        self
    }

    pub fn with_profiles(mut self, aliases: &AliasSet, subs: &SubcommandSet) -> Self {
        self.profile_aliases = aliases.clone();
        self.profile_subcommands = subs.clone();
        self
    }

    pub fn with_project(mut self, aliases: &AliasSet, subs: &SubcommandSet) -> Self {
        self.project_aliases = aliases.clone();
        self.project_subcommands = subs.clone();
        self
    }

    pub fn with_shell_state_from_env(
        mut self,
        aliases: Option<&str>,
        subcommands: Option<&str>,
    ) -> Self {
        self.shell_alias_state = Self::parse_state(aliases);
        self.shell_subcmd_state = Self::parse_state(subcommands);
        self
    }

    /// Internal: merged alias set keyed by shell-visible name,
    /// with project > profile > global precedence.
    fn merged_aliases(&self) -> BTreeMap<String, TomlAlias> {
        let mut out = BTreeMap::new();
        for layer in [&self.global_aliases, &self.profile_aliases, &self.project_aliases] {
            for (name, alias) in layer.iter() {
                out.insert(name.as_ref().to_string(), alias.clone());
            }
        }
        out
    }

    /// Internal: merged subcommand set keyed by full "program:seg:..." key,
    /// with project > profile > global precedence.
    fn merged_subcommands(&self) -> SubcommandSet {
        let mut out = SubcommandSet::new();
        for layer in [&self.global_subcommands, &self.profile_subcommands, &self.project_subcommands] {
            for (k, v) in layer {
                out.insert(k.clone(), v.clone());
            }
        }
        out
    }

    fn alias_hash(alias: &TomlAlias) -> String {
        crate::trust::compute_short_hash(alias.command().as_bytes())
    }

    fn subcmd_program_hash(program: &str, subs: &SubcommandSet) -> String {
        let entries_str: String = subs
            .iter()
            .filter(|(k, _)| k.starts_with(&format!("{program}:")))
            .map(|(k, v)| format!("{k}={}", v.join(",")))
            .collect::<Vec<_>>()
            .join(";");
        crate::trust::compute_short_hash(entries_str.as_bytes())
    }

    fn subcmd_key_hash(longs: &[String]) -> String {
        crate::trust::compute_short_hash(longs.join(",").as_bytes())
    }

    fn parse_state(raw: Option<&str>) -> BTreeMap<String, Option<String>> {
        let mut map = BTreeMap::new();
        let Some(s) = raw.filter(|s| !s.is_empty()) else {
            return map;
        };
        for entry in s.split(',') {
            if let Some((name, hash)) = entry.split_once('|') {
                map.insert(name.to_string(), Some(hash.to_string()));
            } else {
                map.insert(entry.to_string(), None);
            }
        }
        map
    }

    #[cfg(test)]
    fn merged_aliases_for_test(&self) -> BTreeMap<String, TomlAlias> {
        self.merged_aliases()
    }

    #[cfg(test)]
    pub(crate) fn alias_hash_for_test(alias: &TomlAlias) -> String {
        Self::alias_hash(alias)
    }

    #[cfg(test)]
    pub(crate) fn subcmd_program_hash_for_test(program: &str, subs: &SubcommandSet) -> String {
        Self::subcmd_program_hash(program, subs)
    }

    #[cfg(test)]
    pub(crate) fn subcmd_key_hash_for_test(longs: &[String]) -> String {
        Self::subcmd_key_hash(longs)
    }

    #[cfg(test)]
    pub(crate) fn shell_alias_state_for_test(&self) -> &BTreeMap<String, Option<String>> {
        &self.shell_alias_state
    }

    #[cfg(test)]
    pub(crate) fn shell_subcmd_state_for_test(&self) -> &BTreeMap<String, Option<String>> {
        &self.shell_subcmd_state
    }

    pub fn resolve(self) -> PrecedenceDiff {
        PrecedenceDiff::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_inputs_produce_empty_diff() {
        let diff = Precedence::new().resolve();
        assert_eq!(diff, PrecedenceDiff::default());
    }

    fn aset(pairs: &[(&str, &str)]) -> AliasSet {
        let mut s = AliasSet::default();
        for (n, c) in pairs {
            s.insert(AliasName::from(*n), TomlAlias::Command((*c).into()));
        }
        s
    }

    #[test]
    fn merge_project_overrides_profile_overrides_global() {
        let global = aset(&[("ll", "ls -lha"), ("t", "global-t")]);
        let profile = aset(&[("gs", "git status"), ("t", "profile-t")]);
        let project = aset(&[("b", "make build"), ("t", "project-t")]);

        let p = Precedence::new()
            .with_global(&global, &SubcommandSet::new())
            .with_profiles(&profile, &SubcommandSet::new())
            .with_project(&project, &SubcommandSet::new());

        let merged = p.merged_aliases_for_test();
        assert_eq!(merged.get("ll").unwrap().command(), "ls -lha");
        assert_eq!(merged.get("gs").unwrap().command(), "git status");
        assert_eq!(merged.get("b").unwrap().command(), "make build");
        assert_eq!(merged.get("t").unwrap().command(), "project-t");
    }

    #[test]
    fn merge_without_project_falls_back_to_profile() {
        let global = aset(&[("t", "global-t")]);
        let profile = aset(&[("t", "profile-t")]);
        let p = Precedence::new()
            .with_global(&global, &SubcommandSet::new())
            .with_profiles(&profile, &SubcommandSet::new());
        let merged = p.merged_aliases_for_test();
        assert_eq!(merged.get("t").unwrap().command(), "profile-t");
    }

    #[test]
    fn hash_alias_stable_and_differs_by_command() {
        let a = TomlAlias::Command("make build".into());
        let b = TomlAlias::Command("cargo build".into());
        let h_a = Precedence::alias_hash_for_test(&a);
        let h_b = Precedence::alias_hash_for_test(&b);
        assert_eq!(h_a.len(), 7);
        assert_ne!(h_a, h_b);
        assert_eq!(h_a, Precedence::alias_hash_for_test(&a));
    }

    #[test]
    fn hash_subcmd_program_includes_all_entries_under_it() {
        let mut a = SubcommandSet::new();
        a.insert("jj:ab".into(), vec!["abandon".into()]);
        let mut b = a.clone();
        b.insert("jj:bl".into(), vec!["branch".into(), "list".into()]);

        let h_a = Precedence::subcmd_program_hash_for_test("jj", &a);
        let h_b = Precedence::subcmd_program_hash_for_test("jj", &b);
        assert_eq!(h_a.len(), 7);
        assert_ne!(h_a, h_b, "adding jj:bl must change jj program hash");
    }

    #[test]
    fn hash_subcmd_key_hashes_long_subcommands() {
        let key_hash = Precedence::subcmd_key_hash_for_test(&["branch".into(), "list".into()]);
        assert_eq!(key_hash.len(), 7);
        assert_eq!(
            key_hash,
            Precedence::subcmd_key_hash_for_test(&["branch".into(), "list".into()])
        );
    }

    #[test]
    fn parse_shell_state_new_format() {
        let p = Precedence::new()
            .with_shell_state_from_env(Some("b|abc1234,t|def5678"), None);
        let aliases = p.shell_alias_state_for_test();
        assert_eq!(aliases.get("b"), Some(&Some("abc1234".into())));
        assert_eq!(aliases.get("t"), Some(&Some("def5678".into())));
    }

    #[test]
    fn parse_shell_state_old_name_only_format_treated_as_unknown() {
        let p = Precedence::new().with_shell_state_from_env(Some("b,t"), None);
        let aliases = p.shell_alias_state_for_test();
        assert_eq!(aliases.get("b"), Some(&None));
        assert_eq!(aliases.get("t"), Some(&None));
    }

    #[test]
    fn parse_shell_state_empty_and_none() {
        let p1 = Precedence::new().with_shell_state_from_env(None, None);
        assert!(p1.shell_alias_state_for_test().is_empty());
        let p2 = Precedence::new().with_shell_state_from_env(Some(""), None);
        assert!(p2.shell_alias_state_for_test().is_empty());
    }

    #[test]
    fn parse_shell_state_mixed_format() {
        let p = Precedence::new()
            .with_shell_state_from_env(Some("b|abc1234,t,gs|fed9876"), None);
        let aliases = p.shell_alias_state_for_test();
        assert_eq!(aliases.get("b"), Some(&Some("abc1234".into())));
        assert_eq!(aliases.get("t"), Some(&None));
        assert_eq!(aliases.get("gs"), Some(&Some("fed9876".into())));
    }

    #[test]
    fn parse_shell_state_subcommands_stored_separately() {
        let p = Precedence::new()
            .with_shell_state_from_env(Some("b|aaa0000"), Some("jj|bbb1111,jj:ab|ccc2222"));
        assert!(p.shell_alias_state_for_test().contains_key("b"));
        let subs = p.shell_subcmd_state_for_test();
        assert_eq!(subs.get("jj"), Some(&Some("bbb1111".into())));
        assert_eq!(subs.get("jj:ab"), Some(&Some("ccc2222".into())));
    }
}
