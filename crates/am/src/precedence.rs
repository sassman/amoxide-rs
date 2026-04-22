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
        let merged_aliases = self.merged_aliases();
        let merged_subcommands = self.merged_subcommands();
        let subcmd_groups = crate::subcommand::group_by_program(&merged_subcommands);
        let program_names: BTreeSet<String> = subcmd_groups.keys().cloned().collect();

        let mut effective: BTreeMap<String, EffectiveEntry> = BTreeMap::new();

        // Regular aliases — skip names absorbed by a subcommand wrapper.
        for (name, alias) in merged_aliases.iter() {
            if program_names.contains(name) {
                continue;
            }
            let hash = Self::alias_hash(alias);
            effective.insert(
                name.clone(),
                EffectiveEntry {
                    name: name.clone(),
                    kind: EntryKind::Alias(alias.clone()),
                    hash,
                },
            );
        }

        // Subcommand wrappers (one entry per program).
        for (program, entries) in &subcmd_groups {
            let base_cmd = merged_aliases
                .get(program)
                .map(|a| a.command().to_string());
            let hash = Self::subcmd_program_hash(program, &merged_subcommands);
            effective.insert(
                program.clone(),
                EffectiveEntry {
                    name: program.clone(),
                    kind: EntryKind::SubcommandWrapper {
                        program: program.clone(),
                        entries: entries.clone(),
                        base_cmd,
                    },
                    hash,
                },
            );
        }

        // Per-key subcommand tracking for `_AM_SUBCOMMANDS`.
        let mut effective_subkeys: BTreeMap<String, EffectiveEntry> = BTreeMap::new();
        for (key, longs) in merged_subcommands.iter() {
            let hash = Self::subcmd_key_hash(longs);
            effective_subkeys.insert(
                key.clone(),
                EffectiveEntry {
                    name: key.clone(),
                    kind: EntryKind::SubcommandKey {
                        longs: longs.clone(),
                    },
                    hash,
                },
            );
        }

        let mut diff = PrecedenceDiff::default();

        // --- Regular + wrapper diff against shell_alias_state ---
        for (name, _) in &self.shell_alias_state {
            if !effective.contains_key(name) {
                diff.removed.push(name.clone());
            }
        }
        for (name, entry) in effective {
            match self.shell_alias_state.get(&name) {
                None => diff.added.push(entry),
                Some(prev) if prev.as_deref() == Some(entry.hash.as_str()) => {
                    diff.unchanged.push(entry)
                }
                Some(_) => diff.changed.push(entry),
            }
        }

        // --- Per-key subcommand diff against shell_subcmd_state ---
        //
        // The program-level wrapper already lives in `effective`/`diff` above.
        // Here we additionally track individual keys so they appear in
        // `_AM_SUBCOMMANDS` with fine-grained hashes.
        for (name, _) in &self.shell_subcmd_state {
            // A program-level entry (no ':') is tracked in shell_alias_state, not here.
            if !name.contains(':') {
                continue;
            }
            if !effective_subkeys.contains_key(name) {
                diff.removed.push(name.clone());
            }
        }
        for (name, entry) in effective_subkeys {
            match self.shell_subcmd_state.get(&name) {
                None => diff.added.push(entry),
                Some(prev) if prev.as_deref() == Some(entry.hash.as_str()) => {
                    diff.unchanged.push(entry)
                }
                Some(_) => diff.changed.push(entry),
            }
        }

        diff
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

    fn find<'a>(v: &'a [EffectiveEntry], name: &str) -> Option<&'a EffectiveEntry> {
        v.iter().find(|e| e.name == name)
    }

    fn cmd_of(entry: &EffectiveEntry) -> &str {
        match &entry.kind {
            EntryKind::Alias(a) => a.command(),
            _ => panic!("expected Alias, got {:?}", entry.kind),
        }
    }

    #[test]
    fn resolve_fresh_load_everything_added() {
        let global = aset(&[("ll", "ls -lha")]);
        let profile = aset(&[("gs", "git status")]);
        let project = aset(&[("b", "make build")]);
        let diff = Precedence::new()
            .with_global(&global, &SubcommandSet::new())
            .with_profiles(&profile, &SubcommandSet::new())
            .with_project(&project, &SubcommandSet::new())
            .resolve();
        let added_names: BTreeSet<_> = diff.added.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(
            added_names,
            BTreeSet::from(["ll", "gs", "b"]),
        );
        assert!(diff.changed.is_empty());
        assert!(diff.removed.is_empty());
        assert!(diff.unchanged.is_empty());
    }

    #[test]
    fn resolve_unchanged_when_hashes_match() {
        let project = aset(&[("b", "make build")]);
        let hash = Precedence::alias_hash_for_test(&TomlAlias::Command("make build".into()));
        let prev = format!("b|{hash}");
        let diff = Precedence::new()
            .with_project(&project, &SubcommandSet::new())
            .with_shell_state_from_env(Some(&prev), None)
            .resolve();
        assert!(diff.added.is_empty());
        assert!(diff.changed.is_empty());
        assert!(diff.removed.is_empty());
        assert_eq!(diff.unchanged.len(), 1);
        assert_eq!(diff.unchanged[0].name, "b");
    }

    #[test]
    fn resolve_changed_when_hash_differs() {
        let project = aset(&[("b", "cargo build")]);
        let prev = "b|0000000"; // obviously not the real hash
        let diff = Precedence::new()
            .with_project(&project, &SubcommandSet::new())
            .with_shell_state_from_env(Some(prev), None)
            .resolve();
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(cmd_of(&diff.changed[0]), "cargo build");
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
    }

    #[test]
    fn resolve_backward_compat_bare_name_triggers_reload() {
        let project = aset(&[("b", "make build")]);
        let diff = Precedence::new()
            .with_project(&project, &SubcommandSet::new())
            .with_shell_state_from_env(Some("b"), None) // old format
            .resolve();
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].name, "b");
    }

    #[test]
    fn resolve_removed_when_no_layer_contains_name() {
        let diff = Precedence::new()
            .with_shell_state_from_env(Some("gone|abc1234"), None)
            .resolve();
        assert_eq!(diff.removed, vec!["gone".to_string()]);
    }

    #[test]
    fn resolve_shadow_restoration_via_changed_entry() {
        // Previous session: project 't' shadowed profile 't'. Now project layer is
        // gone (we left the project directory). Effective 't' reverts to profile.
        // The stored hash was the project's; the new effective hash is the profile's.
        // This must be detected as Changed -> the shell reloads with the profile value.
        let profile = aset(&[("t", "profile-t")]);
        let project_hash = Precedence::alias_hash_for_test(&TomlAlias::Command("project-t".into()));
        let prev = format!("t|{project_hash}");
        let diff = Precedence::new()
            .with_profiles(&profile, &SubcommandSet::new())
            .with_shell_state_from_env(Some(&prev), None)
            .resolve();
        assert_eq!(diff.changed.len(), 1, "shadow restoration must emit a reload");
        assert_eq!(cmd_of(&diff.changed[0]), "profile-t");
        assert!(diff.removed.is_empty());
    }

    fn subset(pairs: &[(&str, &[&str])]) -> SubcommandSet {
        let mut s = SubcommandSet::new();
        for (k, longs) in pairs {
            s.insert((*k).into(), longs.iter().map(|x| (*x).into()).collect());
        }
        s
    }

    #[test]
    fn resolve_subcommand_fresh_load_emits_wrapper() {
        let project_subs = subset(&[("jj:ab", &["abandon"])]);
        let diff = Precedence::new()
            .with_project(&AliasSet::default(), &project_subs)
            .resolve();
        let wrapper = find(&diff.added, "jj").expect("expected jj wrapper in added");
        match &wrapper.kind {
            EntryKind::SubcommandWrapper { program, entries, base_cmd } => {
                assert_eq!(program, "jj");
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].short_subcommands, vec!["ab"]);
                assert_eq!(entries[0].long_subcommands, vec!["abandon"]);
                assert!(base_cmd.is_none());
            }
            other => panic!("expected SubcommandWrapper, got {other:?}"),
        }
        // per-key entry also added (for env-var tracking)
        let key = find(&diff.added, "jj:ab").expect("expected per-key entry");
        assert!(matches!(key.kind, EntryKind::SubcommandKey { .. }));
    }

    #[test]
    fn resolve_subcommand_base_cmd_from_regular_alias_same_name() {
        let aliases = aset(&[("jj", "just-a-joke")]);
        let subs = subset(&[("jj:ab", &["abandon"])]);
        let diff = Precedence::new()
            .with_project(&aliases, &subs)
            .resolve();
        let wrapper = find(&diff.added, "jj").unwrap();
        match &wrapper.kind {
            EntryKind::SubcommandWrapper { base_cmd, .. } => {
                assert_eq!(base_cmd.as_deref(), Some("just-a-joke"));
            }
            _ => panic!(),
        }
        // Only one entry named "jj" — the wrapper, which absorbs the alias.
        let jj_hits = diff.added.iter().filter(|e| e.name == "jj").count();
        assert_eq!(jj_hits, 1, "only the wrapper entry should represent 'jj'");
    }

    #[test]
    fn resolve_subcommand_different_keys_coexist_across_layers() {
        let profile_subs = subset(&[("jj:ab", &["abandon"])]);
        let project_subs = subset(&[("jj:b:l", &["branch", "list"])]);
        let diff = Precedence::new()
            .with_profiles(&AliasSet::default(), &profile_subs)
            .with_project(&AliasSet::default(), &project_subs)
            .resolve();
        let wrapper = find(&diff.added, "jj").unwrap();
        match &wrapper.kind {
            EntryKind::SubcommandWrapper { entries, .. } => {
                let keys: BTreeSet<_> = entries.iter().map(|e| e.to_key()).collect();
                assert_eq!(keys, BTreeSet::from(["jj:ab".into(), "jj:b:l".into()]));
            }
            _ => panic!(),
        }
    }

    #[test]
    fn resolve_subcommand_project_key_overrides_profile_same_key() {
        let profile_subs = subset(&[("jj:ab", &["abandon"])]);
        let project_subs = subset(&[("jj:ab", &["abandon-force"])]);
        let diff = Precedence::new()
            .with_profiles(&AliasSet::default(), &profile_subs)
            .with_project(&AliasSet::default(), &project_subs)
            .resolve();
        let wrapper = find(&diff.added, "jj").unwrap();
        match &wrapper.kind {
            EntryKind::SubcommandWrapper { entries, .. } => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].long_subcommands, vec!["abandon-force"]);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn resolve_subcommand_unchanged_when_program_hash_matches() {
        let subs = subset(&[("jj:ab", &["abandon"])]);
        let merged = subs.clone();
        let program_hash = Precedence::subcmd_program_hash_for_test("jj", &merged);
        let key_hash = Precedence::subcmd_key_hash_for_test(&["abandon".into()]);
        let prev_aliases = format!("jj|{program_hash}");
        let prev_subs = format!("jj:ab|{key_hash}");
        let diff = Precedence::new()
            .with_project(&AliasSet::default(), &subs)
            .with_shell_state_from_env(Some(&prev_aliases), Some(&prev_subs))
            .resolve();
        assert!(diff.added.is_empty(), "got added: {:?}", diff.added);
        assert!(diff.changed.is_empty(), "got changed: {:?}", diff.changed);
        assert!(diff.removed.is_empty(), "got removed: {:?}", diff.removed);
        assert_eq!(diff.unchanged.len(), 2, "jj wrapper + jj:ab key both unchanged");
    }

    #[test]
    fn resolve_subcommand_regenerates_wrapper_when_entry_added() {
        // Previous: only jj:ab was tracked. Now jj:bl is added too.
        // The program hash changes -> wrapper must be in `changed`.
        let subs_before = subset(&[("jj:ab", &["abandon"])]);
        let program_hash_before = Precedence::subcmd_program_hash_for_test("jj", &subs_before);
        let key_hash_ab = Precedence::subcmd_key_hash_for_test(&["abandon".into()]);
        let prev_aliases = format!("jj|{program_hash_before}");
        let prev_subs = format!("jj:ab|{key_hash_ab}");

        let subs_after = subset(&[("jj:ab", &["abandon"]), ("jj:bl", &["branch", "list"])]);
        let diff = Precedence::new()
            .with_project(&AliasSet::default(), &subs_after)
            .with_shell_state_from_env(Some(&prev_aliases), Some(&prev_subs))
            .resolve();
        assert!(find(&diff.changed, "jj").is_some(), "wrapper must be regenerated");
        assert!(find(&diff.added, "jj:bl").is_some(), "new key must be added");
        // jj:ab itself unchanged
        assert!(find(&diff.unchanged, "jj:ab").is_some(), "jj:ab entry itself is unchanged");
    }
}
