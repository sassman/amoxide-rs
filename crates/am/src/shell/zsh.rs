use super::{NixShell, Shell};
use crate::alias::AliasEntry;

#[derive(Debug, Default)]
pub struct Zsh;

impl Shell for Zsh {
    fn unalias(&self, alias_name: &str) -> String {
        NixShell.unalias(alias_name)
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        NixShell.alias(entry)
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

/// Parses the raw stdout of `zsh -i -c 'alias'` into a set of alias names.
///
/// Each line follows the grammar: `name=value` or `name='quoted value'`.
/// Only the key (left of the first `=`) is extracted.
pub(crate) fn parse_zsh_alias_keys(output: &str) -> std::collections::HashSet<String> {
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

/// Spawns `zsh -i -c 'alias'` and returns the set of alias names defined in
/// the user's zsh startup files.
///
/// Returns an empty set on any error — scan failure is non-fatal.
/// Sets `AM_DETECTING_ALIASES=1` to prevent recursive `am` invocation if
/// `.zshrc` contains `eval "$(am init zsh)"`.
pub fn scan_external_aliases() -> std::collections::HashSet<String> {
    // Guard against recursive invocation: if we are already inside a
    // `zsh -i -c alias` subprocess triggered by an outer scan, return empty
    // immediately rather than spawning another child.
    if std::env::var("AM_DETECTING_ALIASES").is_ok() {
        return Default::default();
    }
    let output = std::process::Command::new("zsh")
        .args(["-i", "-c", "alias"])
        .env("AM_DETECTING_ALIASES", "1")
        .stderr(std::process::Stdio::null())
        .output();
    match output {
        Ok(out) => parse_zsh_alias_keys(&String::from_utf8_lossy(&out.stdout)),
        Err(_) => Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_alias_keys_handles_quoted_values() {
        let raw = "gs='git status'\nll='ls -lh'\n";
        let keys = parse_zsh_alias_keys(raw);
        assert!(keys.contains("gs"));
        assert!(keys.contains("ll"));
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn parse_alias_keys_handles_all_formats() {
        let raw = "gs='git status'\nll='ls -lh'\ncomplex='it'\\''s a value'\nsimple=value\n";
        let keys = parse_zsh_alias_keys(raw);
        assert!(keys.contains("gs"));
        assert!(keys.contains("ll"));
        assert!(keys.contains("complex"));
        assert!(keys.contains("simple"));
        assert_eq!(keys.len(), 4);
    }

    #[test]
    fn parse_alias_keys_empty_input_returns_empty_set() {
        assert!(parse_zsh_alias_keys("").is_empty());
    }

    #[test]
    fn parse_alias_keys_ignores_blank_lines() {
        let raw = "\ngs='git status'\n\nll='ls -lh'\n\n";
        let keys = parse_zsh_alias_keys(raw);
        assert_eq!(keys.len(), 2);
    }
}
