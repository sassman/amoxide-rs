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

#[derive(Debug, Default, Clone, PartialEq)]
pub struct PrecedenceDiff {
    pub added: Vec<EffectiveEntry>,
    pub changed: Vec<EffectiveEntry>,
    pub removed: Vec<String>,
    pub unchanged: Vec<EffectiveEntry>,
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
}
