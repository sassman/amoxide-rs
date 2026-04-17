use std::collections::HashSet;

use super::{has_template_args, quote_cmd, substitute_nix, NixShell, Shell};
use crate::alias::AliasEntry;

#[derive(Debug, Default)]
pub struct Bash {
    /// Functions already defined in the user's shell — only emit `unset -f` for names here.
    external_functions: HashSet<String>,
    /// Aliases already defined in the user's shell — only emit `unalias` for names here.
    external_aliases: HashSet<String>,
}

impl Bash {
    pub fn new(external_functions: HashSet<String>, external_aliases: HashSet<String>) -> Self {
        Bash {
            external_functions,
            external_aliases,
        }
    }
}

impl Shell for Bash {
    fn unalias(&self, alias_name: &str) -> String {
        NixShell.unalias(alias_name)
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        if !entry.raw && has_template_args(entry.command) {
            let body = substitute_nix(entry.command);
            if self.external_aliases.contains(entry.name) {
                format!(
                    "unalias {} 2>/dev/null\n{}() {{ {}; }}",
                    entry.name, entry.name, body
                )
            } else {
                format!("{}() {{ {}; }}", entry.name, body)
            }
        } else {
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

/// Parse the stdout of `declare -F` into a set of function names.
///
/// `declare -F` outputs one entry per line:
///   declare -f am
///   declare -f __git_ps1
pub fn parse_bash_function_names(output: &str) -> HashSet<String> {
    output
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            // Must be exactly "declare -f <name>"
            if parts.next() == Some("declare") && parts.next() == Some("-f") {
                parts.next().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Parse the stdout of `alias` into a set of alias names.
///
/// bash `alias` output prefixes each line with `alias `:
///   alias gs='git status'
///   alias ll='ls -lha'
pub fn parse_bash_alias_names(output: &str) -> HashSet<String> {
    output
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("alias ")?;
            rest.split('=').next().map(|k| k.to_string())
        })
        .collect()
}

/// Scan function names defined in the user's interactive bash environment.
pub fn scan_external_functions() -> HashSet<String> {
    run_bash_inspect("declare -F", parse_bash_function_names)
}

/// Scan alias names defined in the user's interactive bash environment.
pub fn scan_external_aliases() -> HashSet<String> {
    run_bash_inspect("alias", parse_bash_alias_names)
}

fn run_bash_inspect(cmd: &str, parse: impl Fn(&str) -> HashSet<String>) -> HashSet<String> {
    let output = std::process::Command::new("bash")
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

    // ── native alias ──────────────────────────────────────────────────────

    #[test]
    fn test_bash_native_alias_no_cleanup_when_sets_empty() {
        let bash = Bash::default();
        assert_eq!(
            bash.alias(&simple("gs", "git status")),
            "alias gs=\"git status\""
        );
    }

    #[test]
    fn test_bash_native_alias_emits_unset_f_when_function_exists() {
        let bash = Bash::new(HashSet::from(["gs".to_string()]), HashSet::new());
        assert_eq!(
            bash.alias(&simple("gs", "git status")),
            "unset -f gs 2>/dev/null\nalias gs=\"git status\""
        );
        assert_eq!(bash.alias(&simple("ll", "ls -lha")), "alias ll=\"ls -lha\"");
    }

    // ── parameterised alias (function) ────────────────────────────────────

    #[test]
    fn test_bash_function_alias_no_cleanup_when_sets_empty() {
        let bash = Bash::default();
        assert_eq!(
            bash.alias(&simple("cmf", "cm feat: {{@}}")),
            "cmf() { cm feat: \"$@\"; }"
        );
    }

    #[test]
    fn test_bash_function_alias_emits_unalias_when_alias_exists() {
        let bash = Bash::new(HashSet::new(), HashSet::from(["cmf".to_string()]));
        assert_eq!(
            bash.alias(&simple("cmf", "cm feat: {{@}}")),
            "unalias cmf 2>/dev/null\ncmf() { cm feat: \"$@\"; }"
        );
        assert_eq!(
            bash.alias(&simple("x", "echo {{1}}")),
            "x() { echo \"$1\"; }"
        );
    }

    #[test]
    fn test_bash_raw_alias_skips_template_detection() {
        let bash = Bash::new(HashSet::new(), HashSet::from(["my-awk".to_string()]));
        assert_eq!(
            bash.alias(&raw("my-awk", "awk '{print {{1}}}'")),
            "alias my-awk=\"awk '{print {{1}}}'\""
        );
    }

    // ── parsers ───────────────────────────────────────────────────────────

    #[test]
    fn test_parse_bash_function_names_handles_declare_f_output() {
        let input = "declare -f am\ndeclare -f __git_ps1\ndeclare -f add-function\n";
        let result = parse_bash_function_names(input);
        assert_eq!(result.len(), 3);
        assert!(result.contains("am"));
        assert!(result.contains("__git_ps1"));
        assert!(result.contains("add-function"));
    }

    #[test]
    fn test_parse_bash_function_names_ignores_blank_and_unrelated_lines() {
        let input = "\ndeclare -f am\n\nsome other line\ndeclare -f foo\n";
        let result = parse_bash_function_names(input);
        assert_eq!(result.len(), 2);
        assert!(result.contains("am"));
        assert!(result.contains("foo"));
    }

    #[test]
    fn test_parse_bash_alias_names_handles_alias_prefix_format() {
        let input = "alias gs='git status'\nalias ll='ls -lha'\nalias simple=value\n";
        let result = parse_bash_alias_names(input);
        assert_eq!(result.len(), 3);
        assert!(result.contains("gs"));
        assert!(result.contains("ll"));
        assert!(result.contains("simple"));
    }

    #[test]
    fn test_parse_bash_alias_names_ignores_blank_lines() {
        let result = parse_bash_alias_names("\nalias gs='git status'\n\nalias ll='ls -lha'\n");
        assert_eq!(result.len(), 2);
    }
}
