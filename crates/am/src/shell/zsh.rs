use std::collections::HashSet;

use super::{has_template_args, quote_cmd, NixShell, ShellAdapter};
use crate::alias::AliasEntry;

#[derive(Debug, Default)]
pub struct Zsh {
    /// Functions already defined in the user's shell — used to conditionally emit
    /// `unset -f <name>` before a native alias that would otherwise be shadowed.
    external_functions: HashSet<String>,
    /// Aliases already defined in the user's shell — used to conditionally emit
    /// `unalias <name>` before a function that would otherwise be shadowed.
    external_aliases: HashSet<String>,
}

impl Zsh {
    pub fn new(external_functions: HashSet<String>, external_aliases: HashSet<String>) -> Self {
        Zsh {
            external_functions,
            external_aliases,
        }
    }
}

impl ShellAdapter for Zsh {
    fn unalias(&self, alias_name: &str) -> String {
        NixShell.unalias(alias_name)
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        if !entry.raw && has_template_args(entry.command) {
            // Parameterised → function; only clear a conflicting alias when we know it exists.
            let body = super::substitute_nix(entry.command);
            if self.external_aliases.contains(entry.name) {
                format!(
                    "unalias {} 2>/dev/null\n{}() {{ {}; }}",
                    entry.name, entry.name, body
                )
            } else {
                format!("{}() {{ {}; }}", entry.name, body)
            }
        } else {
            // Native alias: only clear a conflicting function when we know it exists.
            if self.external_functions.contains(entry.name) {
                format!(
                    "unset -f {} 2>/dev/null\nalias {}={}",
                    entry.name,
                    entry.name,
                    quote_cmd(entry.command)
                )
            } else {
                format!("alias {}={}", entry.name, quote_cmd(entry.command))
            }
        }
    }

    fn set_env(&self, var_name: &str, value: &str) -> String {
        NixShell.set_env(var_name, value)
    }

    fn unset_env(&self, var_name: &str) -> String {
        NixShell.unset_env(var_name)
    }

    fn echo(&self, message: &str) -> String {
        NixShell.echo(message)
    }

    fn subcommand_wrapper(
        &self,
        program: &str,
        base_cmd: &str,
        entries: &[crate::subcommand::SubcommandEntry],
    ) -> String {
        NixShell.subcommand_wrapper(program, base_cmd, entries)
    }
}

// ── Scanning helpers ──────────────────────────────────────────────────────────

/// Parse the stdout of `typeset +f` into a set of function names.
///
/// `typeset +f` outputs one bare function name per line, e.g.:
///   add-zsh-hook
///   compinit
///   am
pub fn parse_zsh_function_names(output: &str) -> HashSet<String> {
    output
        .lines()
        .filter_map(|line| {
            let name = line.trim();
            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        })
        .collect()
}

/// Parse the stdout of `alias` into a set of alias names.
///
/// zsh `alias` output lines follow the grammar:
///   name=value
///   name='quoted value'
///   name='it'\''s here'   (embedded single-quote)
///
/// Only the key (left of the first `=`) is extracted.
pub fn parse_zsh_alias_names(output: &str) -> HashSet<String> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            line.split('=').next().map(|k| k.to_string())
        })
        .collect()
}

/// Scan function names defined in the user's interactive zsh environment.
pub fn scan_external_functions() -> HashSet<String> {
    run_zsh_inspect("typeset +f", parse_zsh_function_names)
}

/// Scan alias names defined in the user's interactive zsh environment.
pub fn scan_external_aliases() -> HashSet<String> {
    run_zsh_inspect("alias", parse_zsh_alias_names)
}

fn run_zsh_inspect(cmd: &str, parse: impl Fn(&str) -> HashSet<String>) -> HashSet<String> {
    let output = std::process::Command::new("zsh")
        .args(["-i", "-c", cmd])
        .env(crate::env_vars::AM_DETECTING_ALIASES, "1")
        .output();

    match output {
        Ok(out) if out.status.success() || out.status.code() == Some(1) => {
            parse(&String::from_utf8_lossy(&out.stdout))
        }
        _ => HashSet::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::test_helpers::{raw, simple};

    // ── native alias (no template args) ──────────────────────────────────

    #[test]
    fn test_zsh_native_alias_no_cleanup_when_sets_empty() {
        let zsh = Zsh::default();
        assert_eq!(
            zsh.alias(&simple("gs", "git status")),
            "alias gs=\"git status\""
        );
    }

    #[test]
    fn test_zsh_native_alias_emits_unset_f_when_function_exists() {
        let fns = HashSet::from(["gs".to_string()]);
        let zsh = Zsh::new(fns, HashSet::new());

        assert_eq!(
            zsh.alias(&simple("gs", "git status")),
            "unset -f gs 2>/dev/null\nalias gs=\"git status\""
        );
        // ll not in set → no cleanup
        assert_eq!(zsh.alias(&simple("ll", "ls -lha")), "alias ll=\"ls -lha\"");
    }

    // ── parameterised alias (template args → function) ────────────────────

    #[test]
    fn test_zsh_function_alias_no_cleanup_when_sets_empty() {
        let zsh = Zsh::default();
        assert_eq!(
            zsh.alias(&simple("cmf", "cm feat: {{@}}")),
            "cmf() { cm feat: \"$@\"; }"
        );
    }

    #[test]
    fn test_zsh_function_alias_emits_unalias_when_alias_exists() {
        let aliases = HashSet::from(["cmf".to_string()]);
        let zsh = Zsh::new(HashSet::new(), aliases);

        assert_eq!(
            zsh.alias(&simple("cmf", "cm feat: {{@}}")),
            "unalias cmf 2>/dev/null\ncmf() { cm feat: \"$@\"; }"
        );
        // x not in aliases set → no unalias
        assert_eq!(
            zsh.alias(&simple("x", "echo {{1}}")),
            "x() { echo \"$1\"; }"
        );
    }

    #[test]
    fn test_zsh_raw_alias_never_emits_unalias() {
        // raw = true skips template detection → always native alias path
        let aliases = HashSet::from(["my-awk".to_string()]);
        let zsh = Zsh::new(HashSet::new(), aliases);
        assert_eq!(
            zsh.alias(&raw("my-awk", "awk '{print {{1}}}'")),
            "alias my-awk=\"awk '{print {{1}}}'\""
        );
    }

    // ── parsers ───────────────────────────────────────────────────────────

    #[test]
    fn test_parse_zsh_function_names_handles_typical_output() {
        let raw = "add-zsh-hook\ncompinit\nam\namoxide_hook\n";
        let result = parse_zsh_function_names(raw);
        assert_eq!(result.len(), 4);
        assert!(result.contains("add-zsh-hook"));
        assert!(result.contains("am"));
    }

    #[test]
    fn test_parse_zsh_function_names_ignores_blank_lines() {
        let result = parse_zsh_function_names("\nadd-zsh-hook\n\ncompinit\n\n");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_parse_zsh_alias_names_handles_various_formats() {
        let input = "gs='git status'\nll='ls -lh'\ncomplex='it'\\''s a value'\nsimple=value\n";
        let result = parse_zsh_alias_names(input);
        assert_eq!(result.len(), 4);
        assert!(result.contains("gs"));
        assert!(result.contains("ll"));
        assert!(result.contains("complex"));
        assert!(result.contains("simple"));
    }

    #[test]
    fn test_parse_zsh_alias_names_ignores_blank_lines() {
        let result = parse_zsh_alias_names("\ngs='git status'\n\nll='ls -lh'\n");
        assert_eq!(result.len(), 2);
    }
}
