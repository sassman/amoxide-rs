use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::alias::{AliasSet, TomlAlias};
use crate::subcommand::SubcommandSet;
use crate::vars::{substitute_vars, VarSet};

use super::diff::{
    Diagnostic, EffectiveEntry, EntryKind, InvalidEntry, InvalidReason, OriginScope,
    PrecedenceDiff, ResolveOutcome,
};
use super::env_state::AliasWithHashList;

/// One profile's contribution to the precedence merge. Each layer carries its
/// own var set so substitution can be scope-local.
#[derive(Debug, Clone, Default)]
pub struct ProfileLayer {
    pub name: String,
    pub aliases: AliasSet,
    pub subcommands: SubcommandSet,
    pub vars: VarSet,
}

#[derive(Debug, Default)]
pub struct Precedence {
    global_aliases: AliasSet,
    global_subcommands: SubcommandSet,
    global_vars: VarSet,
    profile_layers: Vec<ProfileLayer>,
    project_aliases: AliasSet,
    project_subcommands: SubcommandSet,
    project_vars: VarSet,
    shell_alias_state: BTreeMap<String, Option<String>>,
    shell_subcmd_state: BTreeMap<String, Option<String>>,
    external_functions: HashSet<String>,
    external_aliases: HashSet<String>,
}

impl Precedence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_global(mut self, aliases: &AliasSet, subs: &SubcommandSet, vars: &VarSet) -> Self {
        self.global_aliases = aliases.clone();
        self.global_subcommands = subs.clone();
        self.global_vars = vars.clone();
        self
    }

    pub fn with_profiles(mut self, layers: &[ProfileLayer]) -> Self {
        self.profile_layers = layers.to_vec();
        self
    }

    pub fn with_project(mut self, aliases: &AliasSet, subs: &SubcommandSet, vars: &VarSet) -> Self {
        self.project_aliases = aliases.clone();
        self.project_subcommands = subs.clone();
        self.project_vars = vars.clone();
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

    /// Internal: merged subcommand set keyed by full "program:seg:..." key,
    /// with project > profile > global precedence.
    fn merged_subcommands(&self) -> SubcommandSet {
        let mut out = SubcommandSet::new();
        for layer in std::iter::once(&self.global_subcommands)
            .chain(self.profile_layers.iter().map(|l| &l.subcommands))
            .chain(std::iter::once(&self.project_subcommands))
        {
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
        // Test helper: pre-substitute and merge with the same precedence the
        // real `resolve()` uses. Skip invalid entries.
        let (global_resolved, _, _) =
            resolve_layer_aliases(&self.global_aliases, &self.global_vars, OriginScope::Global);
        let mut profile_resolved = AliasSet::default();
        for layer in &self.profile_layers {
            let (aliases, _, _) = resolve_layer_aliases(
                &layer.aliases,
                &layer.vars,
                OriginScope::Profile(layer.name.clone()),
            );
            for (n, a) in aliases.iter() {
                profile_resolved.insert(n.clone(), a.clone());
            }
        }
        let (project_resolved, _, _) = resolve_layer_aliases(
            &self.project_aliases,
            &self.project_vars,
            OriginScope::Project,
        );

        let mut out = BTreeMap::new();
        for layer in [&global_resolved, &profile_resolved, &project_resolved] {
            for (name, alias) in layer.iter() {
                out.insert(name.as_ref().to_string(), alias.clone());
            }
        }
        out
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

    pub fn resolve(self) -> ResolveOutcome {
        // 1. Per-layer var substitution. Invalid aliases are excluded from the
        //    resolved sets and recorded as `InvalidEntry` plus a `Diagnostic`.
        let mut invalid: Vec<InvalidEntry> = Vec::new();
        let mut diagnostics: Vec<Diagnostic> = Vec::new();

        let (global_resolved, mut global_invs, mut global_diags) =
            resolve_layer_aliases(&self.global_aliases, &self.global_vars, OriginScope::Global);
        invalid.append(&mut global_invs);
        diagnostics.append(&mut global_diags);

        let mut profile_resolved = AliasSet::default();
        for layer in &self.profile_layers {
            let (resolved, mut invs, mut diags) = resolve_layer_aliases(
                &layer.aliases,
                &layer.vars,
                OriginScope::Profile(layer.name.clone()),
            );
            invalid.append(&mut invs);
            diagnostics.append(&mut diags);
            for (name, alias) in resolved.iter() {
                profile_resolved.insert(name.clone(), alias.clone());
            }
        }

        let (project_resolved, mut project_invs, mut project_diags) = resolve_layer_aliases(
            &self.project_aliases,
            &self.project_vars,
            OriginScope::Project,
        );
        invalid.append(&mut project_invs);
        diagnostics.append(&mut project_diags);

        // 2. Merge with project > profile > global precedence on the *resolved* sets.
        let merged_aliases: BTreeMap<String, TomlAlias> = {
            let mut out = BTreeMap::new();
            for layer in [&global_resolved, &profile_resolved, &project_resolved] {
                for (name, alias) in layer.iter() {
                    out.insert(name.as_ref().to_string(), alias.clone());
                }
            }
            out
        };
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

        // Names dropped from the effective set due to var-resolution failure
        // also need to land in `removed` if they were previously loaded — so the
        // shell unloads the stale value. The diff loop above already does this
        // since invalid aliases are absent from the `effective` map.
        diff.invalid = invalid;
        ResolveOutcome { diff, diagnostics }
    }
}

/// Apply var substitution to every alias in `aliases` using `vars`. Aliases
/// with missing vars are excluded from the returned `AliasSet` and reported
/// via `Vec<InvalidEntry>` and `Vec<Diagnostic>`.
fn resolve_layer_aliases(
    aliases: &AliasSet,
    vars: &VarSet,
    scope: OriginScope,
) -> (AliasSet, Vec<InvalidEntry>, Vec<Diagnostic>) {
    let mut out = AliasSet::default();
    let mut invs = Vec::new();
    let mut diags = Vec::new();
    for (name, alias) in aliases.iter() {
        let result = substitute_vars(alias.command(), vars);
        if result.missing.is_empty() {
            let resolved = match alias {
                TomlAlias::Command(_) => TomlAlias::Command(result.output),
                TomlAlias::Detailed(d) => TomlAlias::Detailed(crate::AliasDetail {
                    command: result.output,
                    description: d.description.clone(),
                    raw: d.raw,
                }),
            };
            out.insert(name.clone(), resolved);
        } else {
            let scope_label = match &scope {
                OriginScope::Global => "global".to_string(),
                OriginScope::Profile(p) => format!("profile '{p}'"),
                OriginScope::Project => "project".to_string(),
            };
            let names: Vec<String> = result
                .missing
                .iter()
                .map(|v| v.as_str().to_string())
                .collect();
            diags.push(Diagnostic {
                message: format!(
                    "warning: alias '{}' in {} references undefined vars: {}",
                    name.as_ref(),
                    scope_label,
                    names.join(", "),
                ),
            });
            invs.push(InvalidEntry {
                name: name.as_ref().to_string(),
                scope: scope.clone(),
                reason: InvalidReason::MissingVars(result.missing),
            });
        }
    }
    (out, invs, diags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::AliasName;

    fn profile_layer(name: &str, aliases: &AliasSet, subs: &SubcommandSet) -> ProfileLayer {
        ProfileLayer {
            name: name.into(),
            aliases: aliases.clone(),
            subcommands: subs.clone(),
            vars: VarSet::default(),
        }
    }

    #[test]
    fn empty_inputs_produce_empty_diff() {
        let outcome = Precedence::new().resolve();
        assert_eq!(outcome.diff, PrecedenceDiff::default());
        assert!(outcome.diagnostics.is_empty());
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
            .with_global(&global, &SubcommandSet::new(), &VarSet::default())
            .with_profiles(&[profile_layer("p", &profile, &SubcommandSet::new())])
            .with_project(&project, &SubcommandSet::new(), &VarSet::default());

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
            .with_global(&global, &SubcommandSet::new(), &VarSet::default())
            .with_profiles(&[profile_layer("p", &profile, &SubcommandSet::new())]);
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
            .with_global(&global, &SubcommandSet::new(), &VarSet::default())
            .with_profiles(&[profile_layer("p", &profile, &SubcommandSet::new())])
            .with_project(&project, &SubcommandSet::new(), &VarSet::default())
            .resolve()
            .diff;
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
            .with_project(&project, &SubcommandSet::new(), &VarSet::default())
            .with_shell_state_from_env(Some(&prev), None)
            .resolve()
            .diff;
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
            .with_project(&project, &SubcommandSet::new(), &VarSet::default())
            .with_shell_state_from_env(Some(prev), None)
            .resolve()
            .diff;
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(cmd_of(&diff.changed[0]), "cargo build");
        assert!(diff.added.is_empty());
        assert!(diff.removed.is_empty());
    }

    #[test]
    fn resolve_backward_compat_bare_name_triggers_reload() {
        let project = aset(&[("b", "make build")]);
        let diff = Precedence::new()
            .with_project(&project, &SubcommandSet::new(), &VarSet::default())
            .with_shell_state_from_env(Some("b"), None) // old format
            .resolve()
            .diff;
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].name, "b");
    }

    #[test]
    fn resolve_removed_when_no_layer_contains_name() {
        let diff = Precedence::new()
            .with_shell_state_from_env(Some("gone|abc1234"), None)
            .resolve()
            .diff;
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
            .with_profiles(&[profile_layer("p", &profile, &SubcommandSet::new())])
            .with_shell_state_from_env(Some(&prev), None)
            .resolve()
            .diff;
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
            .with_project(&AliasSet::default(), &project_subs, &VarSet::default())
            .resolve()
            .diff;
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
        let diff = Precedence::new()
            .with_project(&aliases, &subs, &VarSet::default())
            .resolve()
            .diff;
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
            .with_profiles(&[profile_layer("p", &AliasSet::default(), &profile_subs)])
            .with_project(&AliasSet::default(), &project_subs, &VarSet::default())
            .resolve()
            .diff;
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
            .with_profiles(&[profile_layer("p", &AliasSet::default(), &profile_subs)])
            .with_project(&AliasSet::default(), &project_subs, &VarSet::default())
            .resolve()
            .diff;
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
            .with_project(&AliasSet::default(), &subs, &VarSet::default())
            .with_shell_state_from_env(Some(&prev_aliases), Some(&prev_subs))
            .resolve()
            .diff;
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
            .with_project(&AliasSet::default(), &subs_after, &VarSet::default())
            .with_shell_state_from_env(Some(&prev_aliases), Some(&prev_subs))
            .resolve()
            .diff;
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

    fn vset(pairs: &[(&str, &str)]) -> VarSet {
        let mut v = VarSet::default();
        for (k, val) in pairs {
            v.insert(crate::vars::VarName::parse(k).unwrap(), (*val).to_string());
        }
        v
    }

    #[test]
    fn resolve_substitutes_vars_for_global_alias() {
        let aliases = aset(&[("hello", "echo {{who}}")]);
        let vars = vset(&[("who", "world")]);
        let outcome = Precedence::new()
            .with_global(&aliases, &SubcommandSet::new(), &vars)
            .resolve();
        let added = outcome.diff.added;
        assert_eq!(added.len(), 1);
        assert_eq!(cmd_of(&added[0]), "echo world");
        assert!(outcome.diff.invalid.is_empty());
    }

    #[test]
    fn resolve_global_alias_using_profile_var_is_invalid() {
        let global_aliases = aset(&[("hello", "echo {{who}}")]);
        let profile_aliases = AliasSet::default();
        let global_vars = VarSet::default();
        let profile_vars = vset(&[("who", "world")]);

        let outcome = Precedence::new()
            .with_global(&global_aliases, &SubcommandSet::new(), &global_vars)
            .with_profiles(&[ProfileLayer {
                aliases: profile_aliases.clone(),
                subcommands: SubcommandSet::new(),
                vars: profile_vars,
                name: "p1".into(),
            }])
            .resolve();

        assert_eq!(outcome.diff.invalid.len(), 1);
        let inv = &outcome.diff.invalid[0];
        assert_eq!(inv.name, "hello");
        assert!(matches!(inv.scope, OriginScope::Global));
        match &inv.reason {
            InvalidReason::MissingVars(v) => {
                assert_eq!(v.len(), 1);
                assert_eq!(v[0].as_str(), "who");
            }
        }
        assert!(outcome.diff.added.iter().all(|e| e.name != "hello"));
        assert!(outcome.diff.changed.iter().all(|e| e.name != "hello"));
    }

    #[test]
    fn resolve_two_profiles_use_their_own_vars() {
        let p1_alias = aset(&[("run", "exec {{path}}/run.sh")]);
        let p1_vars = vset(&[("path", "/v1")]);
        let p2_alias = aset(&[("test", "exec {{path}}/test.sh")]);
        let p2_vars = vset(&[("path", "/v2")]);

        let outcome = Precedence::new()
            .with_profiles(&[
                ProfileLayer {
                    aliases: p1_alias,
                    subcommands: SubcommandSet::new(),
                    vars: p1_vars,
                    name: "p1".into(),
                },
                ProfileLayer {
                    aliases: p2_alias,
                    subcommands: SubcommandSet::new(),
                    vars: p2_vars,
                    name: "p2".into(),
                },
            ])
            .resolve();

        let run = find(&outcome.diff.added, "run").unwrap();
        assert_eq!(cmd_of(run), "exec /v1/run.sh");
        let test_entry = find(&outcome.diff.added, "test").unwrap();
        assert_eq!(cmd_of(test_entry), "exec /v2/test.sh");
        assert!(outcome.diff.invalid.is_empty());
    }

    #[test]
    fn resolve_invalid_alias_previously_loaded_appears_in_both_invalid_and_removed() {
        let project_aliases = aset(&[("cc", "compile {{flags}}")]);
        let project_vars = VarSet::default();
        let prev_hash = Precedence::alias_hash_for_test(&TomlAlias::Command("compile X".into()));
        let prev = format!("cc|{prev_hash}");

        let outcome = Precedence::new()
            .with_project(&project_aliases, &SubcommandSet::new(), &project_vars)
            .with_shell_state_from_env(Some(&prev), None)
            .resolve();

        assert_eq!(
            outcome.diff.removed,
            vec!["cc".to_string()],
            "must unload from shell"
        );
        assert_eq!(
            outcome.diff.invalid.len(),
            1,
            "must be diagnosed as invalid"
        );
        assert_eq!(outcome.diff.invalid[0].name, "cc");
    }

    #[test]
    fn resolve_invalid_alias_never_loaded_only_in_invalid() {
        let global_aliases = aset(&[("cc", "compile {{flags}}")]);
        let outcome = Precedence::new()
            .with_global(&global_aliases, &SubcommandSet::new(), &VarSet::default())
            .resolve();

        assert!(outcome.diff.removed.is_empty());
        assert_eq!(outcome.diff.invalid.len(), 1);
        assert!(outcome.diff.added.iter().all(|e| e.name != "cc"));
    }

    /// Cross-scope combinations: any alias that references a var not in its
    /// own scope is invalid, regardless of direction.
    #[test]
    fn resolve_cross_scope_var_lookup_always_invalid() {
        let cases = [
            ("global", "profile"),
            ("global", "project"),
            ("profile", "global"),
            ("profile", "project"),
            ("project", "global"),
            ("project", "profile"),
        ];
        for (alias_scope, var_scope) in cases {
            let alias = aset(&[("a", "x {{v}}")]);
            let vars = vset(&[("v", "VALUE")]);

            let mut p = Precedence::new();
            match alias_scope {
                "global" => {
                    p = p.with_global(&alias, &SubcommandSet::new(), &VarSet::default());
                }
                "profile" => {
                    p = p.with_profiles(&[ProfileLayer {
                        aliases: alias.clone(),
                        subcommands: SubcommandSet::new(),
                        vars: VarSet::default(),
                        name: "p".into(),
                    }]);
                }
                "project" => {
                    p = p.with_project(&alias, &SubcommandSet::new(), &VarSet::default());
                }
                _ => unreachable!(),
            }
            match var_scope {
                "global" => {
                    p = p.with_global(&AliasSet::default(), &SubcommandSet::new(), &vars);
                }
                "profile" => {
                    p = p.with_profiles(&[ProfileLayer {
                        aliases: AliasSet::default(),
                        subcommands: SubcommandSet::new(),
                        vars: vars.clone(),
                        name: "px".into(),
                    }]);
                }
                "project" => {
                    p = p.with_project(&AliasSet::default(), &SubcommandSet::new(), &vars);
                }
                _ => unreachable!(),
            }

            let outcome = p.resolve();
            assert_eq!(
                outcome.diff.invalid.len(),
                1,
                "alias_scope={alias_scope} var_scope={var_scope}: expected 1 invalid"
            );
            assert!(
                outcome.diff.added.iter().all(|e| e.name != "a"),
                "alias_scope={alias_scope} var_scope={var_scope}: alias must not be added"
            );
        }
    }
}
