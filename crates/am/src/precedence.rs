use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;

use crate::alias::{AliasSet, TomlAlias};
use crate::env_vars;
use crate::shell::ShellAdapter;
use crate::subcommand::{SubcommandEntry, SubcommandSet};

/// One entry in the `_AM_ALIASES` / `_AM_SUBCOMMANDS` env var, in the
/// `"name|hash"` format (or a legacy bare `"name"` with no hash).
///
/// `hash = None` means the shell reloaded from an older amoxide that only
/// tracked names; the diff treats such entries as "always differs" so they
/// get reloaded on the next sync.
#[derive(Debug, Clone, PartialEq)]
pub struct AliasWithHash {
    name: String,
    hash: Option<String>,
}

impl AliasWithHash {
    pub fn new(name: impl Into<String>, hash: Option<String>) -> Self {
        Self {
            name: name.into(),
            hash,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn hash(&self) -> Option<&str> {
        self.hash.as_deref()
    }

    /// Parse one `"name|hash"` (or bare `"name"`) token. Returns `None` when
    /// the name segment is empty — callers skip such entries silently.
    pub fn parse(token: &str) -> Option<Self> {
        match token.split_once('|') {
            Some((name, hash)) if !name.is_empty() => Some(Self {
                name: name.to_string(),
                hash: Some(hash.to_string()),
            }),
            Some(_) => None, // empty name before '|'
            None if token.is_empty() => None,
            None => Some(Self {
                name: token.to_string(),
                hash: None,
            }),
        }
    }
}

impl fmt::Display for AliasWithHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.hash {
            Some(h) => write!(f, "{}|{}", self.name, h),
            None => write!(f, "{}", self.name),
        }
    }
}

/// A comma-separated list of [`AliasWithHash`] entries — the on-the-wire
/// format of `_AM_ALIASES` and `_AM_SUBCOMMANDS`.
///
/// Owns round-trip parsing and rendering so no other module has to know
/// about the `"name|hash,name|hash,..."` layout.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AliasWithHashList(Vec<AliasWithHash>);

impl AliasWithHashList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, entry: AliasWithHash) {
        self.0.push(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, AliasWithHash> {
        self.0.iter()
    }

    /// Parse an `_AM_ALIASES` / `_AM_SUBCOMMANDS` value. `None` or empty
    /// string yields an empty list; malformed tokens are skipped.
    pub fn parse(raw: Option<&str>) -> Self {
        let Some(s) = raw.filter(|s| !s.is_empty()) else {
            return Self::new();
        };
        Self(s.split(',').filter_map(AliasWithHash::parse).collect())
    }
}

impl fmt::Display for AliasWithHashList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, entry) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{entry}")?;
        }
        Ok(())
    }
}

impl FromIterator<AliasWithHash> for AliasWithHashList {
    fn from_iter<I: IntoIterator<Item = AliasWithHash>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a> IntoIterator for &'a AliasWithHashList {
    type Item = &'a AliasWithHash;
    type IntoIter = std::slice::Iter<'a, AliasWithHash>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for AliasWithHashList {
    type Item = AliasWithHash;
    type IntoIter = std::vec::IntoIter<AliasWithHash>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

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

    pub fn with_shell_state_from_introspection(
        mut self,
        functions: &HashSet<String>,
        aliases: &HashSet<String>,
    ) -> Self {
        for name in functions.iter().chain(aliases.iter()) {
            self.shell_alias_state.entry(name.clone()).or_insert(None);
        }
        self.external_functions = functions.clone();
        self.external_aliases = aliases.clone();
        self
    }

    /// Internal: merged alias set keyed by shell-visible name,
    /// with project > profile > global precedence.
    fn merged_aliases(&self) -> BTreeMap<String, TomlAlias> {
        let mut out = BTreeMap::new();
        for layer in [
            &self.global_aliases,
            &self.profile_aliases,
            &self.project_aliases,
        ] {
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
        for layer in [
            &self.global_subcommands,
            &self.profile_subcommands,
            &self.project_subcommands,
        ] {
            for (k, v) in layer {
                out.as_mut().insert(k.clone(), v.clone());
            }
        }
        out
    }

    fn alias_hash(alias: &TomlAlias) -> String {
        crate::trust::compute_short_hash(alias.command().as_bytes())
    }

    fn subcmd_program_hash(program: &str, subs: &SubcommandSet) -> String {
        let entries_str: String = subs
            .as_ref()
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
        AliasWithHashList::parse(raw)
            .into_iter()
            .map(|e| (e.name().to_string(), e.hash().map(String::from)))
            .collect()
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
        let subcmd_groups = merged_subcommands.group_by_program();
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
            let base_cmd = merged_aliases.get(program).map(|a| a.command().to_string());
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
        for (key, longs) in &merged_subcommands {
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
        for name in self.shell_alias_state.keys() {
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
        for name in self.shell_subcmd_state.keys() {
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

impl PrecedenceDiff {
    /// Human-readable summary of what changed, suitable for echoing to the
    /// user (e.g. `"am: aliases changed — 1 loaded: b | 1 unloaded: t"`).
    ///
    /// Returns `None` when nothing changed so callers can stay silent.
    pub fn change_summary(&self) -> Option<String> {
        let added: Vec<&str> = self.added.iter().map(|e| e.name.as_str()).collect();
        let changed: Vec<&str> = self.changed.iter().map(|e| e.name.as_str()).collect();
        let removed: Vec<&str> = self.removed.iter().map(|s| s.as_str()).collect();
        let parts: Vec<String> = [
            ("loaded", &added[..]),
            ("updated", &changed[..]),
            ("unloaded", &removed[..]),
        ]
        .iter()
        .filter(|(_, names)| !names.is_empty())
        .map(|(verb, names)| format!("{} {verb}: {}", names.len(), names.join(", ")))
        .collect();
        if parts.is_empty() {
            None
        } else {
            Some(format!("am: aliases changed — {}", parts.join(" | ")))
        }
    }

    /// Render this diff into shell code using the given adapter.
    ///
    /// Emission order:
    ///   1. unload (removed + changed) — skipping subcommand-key names
    ///      (they're tracking-only, not shell functions)
    ///   2. load (added + changed)
    ///   3. set `_AM_ALIASES` / `_AM_SUBCOMMANDS` to the union of added +
    ///      changed + unchanged
    pub fn render(&self, shell: &dyn ShellAdapter) -> String {
        let mut lines: Vec<String> = Vec::new();

        // 1. Unload
        for name in &self.removed {
            if name.contains(':') {
                continue;
            }
            lines.push(shell.unalias(name));
        }
        for entry in &self.changed {
            if matches!(entry.kind, EntryKind::SubcommandKey { .. }) {
                continue;
            }
            if entry.name.contains(':') {
                continue;
            }
            lines.push(shell.unalias(&entry.name));
        }

        // 2. Load (added + changed)
        for entry in self.added.iter().chain(self.changed.iter()) {
            match &entry.kind {
                EntryKind::Alias(alias) => {
                    lines.push(shell.alias(&alias.as_entry(&entry.name)));
                }
                EntryKind::SubcommandWrapper {
                    program,
                    entries,
                    base_cmd,
                } => {
                    let cmd = base_cmd
                        .clone()
                        .unwrap_or_else(|| format!("command {program}"));
                    lines.push(shell.subcommand_wrapper(program, &cmd, entries));
                }
                EntryKind::SubcommandKey { .. } => {}
            }
        }

        // 3. Update tracking env vars
        let mut alias_list = AliasWithHashList::new();
        let mut sub_list = AliasWithHashList::new();
        for e in self
            .added
            .iter()
            .chain(self.changed.iter())
            .chain(self.unchanged.iter())
        {
            let entry = AliasWithHash::new(&e.name, Some(e.hash.clone()));
            match &e.kind {
                EntryKind::SubcommandKey { .. } => sub_list.push(entry),
                _ => alias_list.push(entry),
            }
        }

        if !alias_list.is_empty() {
            lines.push(shell.set_env(env_vars::AM_ALIASES, &alias_list.to_string()));
        }
        if !sub_list.is_empty() {
            lines.push(shell.set_env(env_vars::AM_SUBCOMMANDS, &sub_list.to_string()));
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::AliasName;

    // ─── AliasWithHash / AliasWithHashList ──────────────────────────────

    #[test]
    fn alias_with_hash_parse_new_format() {
        let e = AliasWithHash::parse("b|abc1234").unwrap();
        assert_eq!(e.name(), "b");
        assert_eq!(e.hash(), Some("abc1234"));
    }

    #[test]
    fn alias_with_hash_parse_bare_name() {
        let e = AliasWithHash::parse("t").unwrap();
        assert_eq!(e.name(), "t");
        assert_eq!(e.hash(), None);
    }

    #[test]
    fn alias_with_hash_parse_empty_returns_none() {
        assert!(AliasWithHash::parse("").is_none());
        assert!(AliasWithHash::parse("|abc").is_none());
    }

    #[test]
    fn alias_with_hash_display_roundtrip() {
        assert_eq!(
            AliasWithHash::new("b", Some("abc1234".into())).to_string(),
            "b|abc1234"
        );
        assert_eq!(AliasWithHash::new("t", None).to_string(), "t");
    }

    #[test]
    fn alias_with_hash_list_parse_and_render() {
        let list = AliasWithHashList::parse(Some("b|abc1234,t|def5678"));
        assert_eq!(list.iter().count(), 2);
        assert_eq!(list.to_string(), "b|abc1234,t|def5678");
    }

    #[test]
    fn alias_with_hash_list_parse_mixed_format() {
        let list = AliasWithHashList::parse(Some("b|abc1234,t,gs|fed9876"));
        let names: Vec<&str> = list.iter().map(|e| e.name()).collect();
        assert_eq!(names, vec!["b", "t", "gs"]);
        assert_eq!(list.iter().nth(1).unwrap().hash(), None);
    }

    #[test]
    fn alias_with_hash_list_parse_empty_and_none() {
        assert!(AliasWithHashList::parse(None).is_empty());
        assert!(AliasWithHashList::parse(Some("")).is_empty());
    }

    #[test]
    fn alias_with_hash_list_parse_skips_malformed_tokens() {
        // Leading empty token and "|xxx" token get dropped silently.
        let list = AliasWithHashList::parse(Some(",|xxx,b|abc1234"));
        let names: Vec<&str> = list.iter().map(|e| e.name()).collect();
        assert_eq!(names, vec!["b"]);
    }

    #[test]
    fn alias_with_hash_list_display_empty() {
        assert_eq!(AliasWithHashList::new().to_string(), "");
    }

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
        a.as_mut().insert("jj:ab".into(), vec!["abandon".into()]);
        let mut b = a.clone();
        b.as_mut()
            .insert("jj:bl".into(), vec!["branch".into(), "list".into()]);

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
        let p = Precedence::new().with_shell_state_from_env(Some("b|abc1234,t|def5678"), None);
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
        let p = Precedence::new().with_shell_state_from_env(Some("b|abc1234,t,gs|fed9876"), None);
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
        assert_eq!(added_names, BTreeSet::from(["ll", "gs", "b"]),);
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
        assert_eq!(
            diff.changed.len(),
            1,
            "shadow restoration must emit a reload"
        );
        assert_eq!(cmd_of(&diff.changed[0]), "profile-t");
        assert!(diff.removed.is_empty());
    }

    fn subset(pairs: &[(&str, &[&str])]) -> SubcommandSet {
        let mut s = SubcommandSet::new();
        for (k, longs) in pairs {
            s.as_mut()
                .insert((*k).into(), longs.iter().map(|x| (*x).into()).collect());
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
            EntryKind::SubcommandWrapper {
                program,
                entries,
                base_cmd,
            } => {
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
        let diff = Precedence::new().with_project(&aliases, &subs).resolve();
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
        assert_eq!(
            diff.unchanged.len(),
            2,
            "jj wrapper + jj:ab key both unchanged"
        );
    }

    #[test]
    fn introspection_adds_names_with_unknown_hash() {
        let mut fns = HashSet::new();
        fns.insert("gs".to_string());
        let mut aliases = HashSet::new();
        aliases.insert("ll".to_string());
        let p = Precedence::new()
            .with_shell_state_from_env(Some("b|abc1234"), None)
            .with_shell_state_from_introspection(&fns, &aliases);
        let state = p.shell_alias_state_for_test();
        assert_eq!(state.get("b"), Some(&Some("abc1234".into())));
        assert_eq!(state.get("gs"), Some(&None));
        assert_eq!(state.get("ll"), Some(&None));
    }

    #[test]
    fn introspection_does_not_overwrite_known_hashes() {
        let mut fns = HashSet::new();
        fns.insert("b".to_string());
        let p = Precedence::new()
            .with_shell_state_from_env(Some("b|abc1234"), None)
            .with_shell_state_from_introspection(&fns, &HashSet::new());
        assert_eq!(
            p.shell_alias_state_for_test().get("b"),
            Some(&Some("abc1234".into()))
        );
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
        assert!(
            find(&diff.changed, "jj").is_some(),
            "wrapper must be regenerated"
        );
        assert!(
            find(&diff.added, "jj:bl").is_some(),
            "new key must be added"
        );
        // jj:ab itself unchanged
        assert!(
            find(&diff.unchanged, "jj:ab").is_some(),
            "jj:ab entry itself is unchanged"
        );
    }

    use crate::config::ShellsTomlConfig;
    use crate::shell::Shell;

    #[test]
    fn render_emits_unloads_then_loads_then_env() {
        let cfg = ShellsTomlConfig::default();
        let shell = Shell::Fish.as_shell(&cfg, Default::default(), Default::default());

        // Previous shell state: `b|0000000,gone|aaa` ; new effective: `b|make build`.
        let project = aset(&[("b", "make build")]);
        let diff = Precedence::new()
            .with_project(&project, &SubcommandSet::new())
            .with_shell_state_from_env(Some("b|0000000,gone|aaa"), None)
            .resolve();

        let out = diff.render(shell.as_ref());
        assert!(
            out.contains("functions -e gone"),
            "gone must be unloaded: {out}"
        );
        assert!(
            out.contains("functions -e b"),
            "changed b must be unloaded: {out}"
        );
        assert!(
            out.contains("function b\n    make build $argv\nend"),
            "b must be reloaded: {out}"
        );
        // env-var update must be the last section
        let env_pos = out.find("_AM_ALIASES").expect("env update missing");
        let fn_pos = out.find("function b").unwrap();
        assert!(env_pos > fn_pos, "env update must come after loads");
    }

    #[test]
    fn render_empty_diff_produces_empty_string() {
        let cfg = ShellsTomlConfig::default();
        let shell = Shell::Fish.as_shell(&cfg, Default::default(), Default::default());
        let out = PrecedenceDiff::default().render(shell.as_ref());
        assert!(out.is_empty());
    }
}
