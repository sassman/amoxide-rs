use crate::alias::TomlAlias;
use crate::env_vars;
use crate::shell::ShellAdapter;
use crate::subcommand::SubcommandEntry;

use super::env_state::{AliasWithHash, AliasWithHashList};

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

/// Where an alias was defined. Used by `InvalidEntry` to point users to the
/// source of a misconfiguration. Distinct from `AliasTarget` (which encodes
/// mutation intent including `ActiveProfile`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OriginScope {
    Global,
    Profile(String),
    Project,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidReason {
    MissingVars(Vec<crate::vars::VarName>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidEntry {
    pub name: String,
    pub scope: OriginScope,
    pub reason: InvalidReason,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub message: String,
}

/// Output of `Precedence::resolve()`. Carries both the sync actions (`diff`)
/// and human-facing warnings (`diagnostics`). Diagnostics are a sibling of
/// `diff.invalid` — they are the rendered, ready-to-print form.
#[derive(Debug, Default, Clone)]
pub struct ResolveOutcome {
    pub diff: PrecedenceDiff,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PrecedenceDiff {
    pub added: Vec<EffectiveEntry>,
    pub changed: Vec<EffectiveEntry>,
    pub removed: Vec<String>,
    pub unchanged: Vec<EffectiveEntry>,
    pub invalid: Vec<InvalidEntry>,
}

/// Build a change-summary line like
/// `"<head> — N verb1: a, b | M verb2: c"`.
///
/// Empty sections are skipped; returns `None` when every section is empty so
/// callers can stay silent. Shared between [`PrecedenceDiff::change_summary`]
/// (fixed head, shell-state diff verbs) and the profile-toggle message in
/// `update.rs` (dynamic head, shadow-aware verbs).
pub(crate) fn format_change_summary(head: &str, sections: &[(&str, &[&str])]) -> Option<String> {
    let parts: Vec<String> = sections
        .iter()
        .filter(|(_, names)| !names.is_empty())
        .map(|(verb, names)| format!("{} {verb}: {}", names.len(), names.join(", ")))
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(format!("{head} — {}", parts.join(" | ")))
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
        format_change_summary(
            "am: aliases changed",
            &[
                ("loaded", &added),
                ("updated", &changed),
                ("unloaded", &removed),
            ],
        )
    }

    /// Like [`Self::change_summary`] but with project-unload prefix and labels.
    ///
    /// Uses "added" for aliases that gained a new definition (profile/global
    /// taking over) and "unloaded" for aliases that are gone entirely.
    pub fn unload_summary(&self) -> Option<String> {
        let added: Vec<&str> = self
            .added
            .iter()
            .chain(self.changed.iter())
            .map(|e| e.name.as_str())
            .collect();
        let removed: Vec<&str> = self.removed.iter().map(|s| s.as_str()).collect();
        format_change_summary(
            "am: .aliases unloaded",
            &[("added", &added), ("unloaded", &removed)],
        )
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
    use super::super::engine::Precedence;
    use super::*;
    use crate::alias::{AliasName, AliasSet};
    use crate::config::ShellsTomlConfig;
    use crate::shell::Shell;
    use crate::subcommand::SubcommandSet;

    fn aset(pairs: &[(&str, &str)]) -> AliasSet {
        let mut s = AliasSet::default();
        for (n, c) in pairs {
            s.insert(AliasName::from(*n), TomlAlias::Command((*c).into()));
        }
        s
    }

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

    #[test]
    fn unload_summary_uses_project_prefix_and_labels() {
        let project = aset(&[("b", "make build"), ("t", "cargo test")]);
        let diff = Precedence::new()
            .with_project(&project, &SubcommandSet::new())
            .with_shell_state_from_env(Some("b|0000000,t|1111111"), None)
            .resolve();

        // All aliases removed (no global/profile to take over)
        let summary = diff.unload_summary();
        assert!(summary.is_some());
        let msg = summary.unwrap();
        assert!(msg.starts_with("am: .aliases unloaded"), "got: {msg}");
        assert!(msg.contains("unloaded"), "got: {msg}");
    }

    #[test]
    fn unload_summary_shows_added_for_takeover() {
        let global = aset(&[("b", "global build")]);
        let _project = aset(&[("b", "project build"), ("t", "cargo test")]);

        // Shell had both from project. Now project is gone, global takes over b.
        // Simulate: resolve with only global + old shell state that had both
        let diff_after = Precedence::new()
            .with_global(&global, &SubcommandSet::new())
            .with_shell_state_from_env(Some("b|0000000,t|1111111"), None)
            .resolve();

        let summary = diff_after.unload_summary();
        assert!(summary.is_some());
        let msg = summary.unwrap();
        assert!(msg.starts_with("am: .aliases unloaded"), "got: {msg}");
        // b is "added" (global takes over), t is "unloaded" (gone)
        assert!(msg.contains("added"), "expected 'added' in: {msg}");
        assert!(msg.contains("unloaded"), "expected 'unloaded' in: {msg}");
    }

    #[test]
    fn unload_summary_returns_none_when_nothing_changed() {
        let diff = PrecedenceDiff::default();
        assert!(diff.unload_summary().is_none());
    }

    #[test]
    fn precedence_diff_default_has_empty_invalid() {
        let d = PrecedenceDiff::default();
        assert!(d.invalid.is_empty());
    }

    #[test]
    fn invalid_entry_has_name_scope_and_reason() {
        use crate::vars::VarName;

        let e = InvalidEntry {
            name: "cc".into(),
            scope: OriginScope::Profile("compile_help".into()),
            reason: InvalidReason::MissingVars(vec![VarName::parse("opt-flags").unwrap()]),
        };
        assert_eq!(e.name, "cc");
        assert!(matches!(e.scope, OriginScope::Profile(_)));
        match e.reason {
            InvalidReason::MissingVars(v) => assert_eq!(v[0].as_str(), "opt-flags"),
        }
    }
}
