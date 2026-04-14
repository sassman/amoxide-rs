use std::collections::BTreeMap;
use std::fmt::{Debug, Display};
use std::sync::LazyLock;

use clap::ValueEnum;
use regex::Regex;

use crate::config::ShellsTomlConfig;

pub trait Shell: Send + Sync + Debug {
    fn unalias(&self, alias_name: &str) -> String;
    fn alias(&self, entry: &crate::alias::AliasEntry) -> String;
    fn set_env(&self, var_name: &str, value: &str) -> String;
    fn unset_env(&self, var_name: &str) -> String;
    fn echo(&self, message: &str) -> String;

    /// Generate a wrapper function for a program with subcommand aliases.
    /// `base_cmd` is either `"command <program>"` (no regular alias) or the alias expansion.
    fn subcommand_wrapper(
        &self,
        program: &str,
        base_cmd: &str,
        entries: &[crate::subcommand::SubcommandEntry],
    ) -> String;
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum Shells {
    Bash,
    Brush,
    // Elvish,
    Fish,
    // Ksh,
    // Nushell,
    // Posix,
    Powershell,
    // Xonsh,
    Zsh,
    // #[cfg(windows)]
    // Cmd,
}

pub struct ShellContext<'a> {
    pub shell: &'a Shells,
    pub cfg: &'a ShellsTomlConfig,
    pub cwd: &'a std::path::Path,
}

impl Shells {
    pub fn as_shell(self, shell_cfg: &ShellsTomlConfig) -> Box<dyn Shell> {
        match self {
            Shells::Fish => Box::new(super::fish::Fish::from_config(shell_cfg.fish.as_ref())),
            Shells::Zsh => Box::from(super::zsh::Zsh),
            Shells::Bash => Box::from(super::bash::Bash),
            Shells::Brush => Box::from(super::brush::Brush),
            Shells::Powershell => Box::from(super::powershell::PowerShell),
        }
    }
}

impl Display for Shells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shells::Bash => write!(f, "bash"),
            Shells::Brush => write!(f, "brush"),
            // Shells::Elvish => write!(f, "elvish"),
            Shells::Fish => write!(f, "fish"),
            // Shells::Ksh => write!(f, "ksh"),
            // Shells::Nushell => write!(f, "nushell"),
            // Shells::Posix => write!(f, "posix"),
            Shells::Powershell => write!(f, "powershell"),
            // Shells::Xonsh => write!(f, "xonsh"),
            Shells::Zsh => write!(f, "zsh"),
        }
    }
}

impl From<Shells> for String {
    fn from(val: Shells) -> Self {
        format!("{}", val)
    }
}


/// A trie node used to build recursive subcommand wrapper functions.
/// Each node may represent an intermediate group or a leaf alias (or both).
#[derive(Default)]
pub(super) struct WrapperNode {
    /// Full long subcommands for a complete entry at this depth (if one exists).
    pub leaf_longs: Option<Vec<String>>,
    /// Children keyed by the next short subcommand token.
    pub children: BTreeMap<String, WrapperNode>,
}

/// Build a trie from a flat list of subcommand entries grouped under one program.
/// Returns a map from first-level short token → node.
pub(super) fn build_wrapper_trie(
    entries: &[crate::subcommand::SubcommandEntry],
) -> BTreeMap<String, WrapperNode> {
    let mut roots: BTreeMap<String, WrapperNode> = BTreeMap::new();
    for entry in entries {
        if entry.short_subcommands.is_empty() {
            continue;
        }
        let node = roots.entry(entry.short_subcommands[0].clone()).or_default();
        trie_insert(node, &entry.short_subcommands[1..], &entry.long_subcommands);
    }
    roots
}

fn trie_insert(node: &mut WrapperNode, remaining: &[String], full_longs: &[String]) {
    if remaining.is_empty() {
        node.leaf_longs = Some(full_longs.to_vec());
    } else {
        let child = node.children.entry(remaining[0].clone()).or_default();
        trie_insert(child, &remaining[1..], full_longs);
    }
}

pub fn quote_cmd(cmd: &str) -> String {
    let (cmd, quotes) = if cmd.starts_with("'") && cmd.ends_with("'") {
        (&cmd[1..cmd.len() - 1], "'")
    } else {
        (cmd, "\"")
    };

    format!("{quotes}{cmd}{quotes}")
}

pub(super) static TEMPLATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{([1-9]|@)\}\}").unwrap());

/// Returns true if the command contains valid template args like `{{1}}`, `{{@}}`.
pub fn has_template_args(cmd: &str) -> bool {
    TEMPLATE_RE.is_match(cmd)
}

/// Template substitution that is aware of single-quote context.
///
/// In POSIX shells and fish, variables inside `'...'` are not expanded. When a
/// `{{N}}` template appears inside a single-quoted region the quoting is broken
/// so the variable can expand:
///
///   `'before{{1}}after'`  →  `'before'<sub>'after'`
///
/// `make_sub` receives the capture group (e.g. `"1"`, `"@"`) and returns the
/// shell-specific variable string.
pub(super) fn substitute_quote_aware(cmd: &str, make_sub: impl Fn(&str) -> String) -> String {
    let mut result = String::new();
    let mut rest = cmd;
    let mut in_single_quote = false;

    while !rest.is_empty() {
        let next_sq = rest.find('\'');
        let next_tmpl = TEMPLATE_RE.find(rest).map(|m| m.start());

        match (next_sq, next_tmpl) {
            (Some(sq), Some(tmpl)) if sq < tmpl => {
                // Single quote comes before the next template: consume it and flip state.
                result.push_str(&rest[..=sq]);
                in_single_quote = !in_single_quote;
                rest = &rest[sq + 1..];
            }
            (_, Some(tmpl)) => {
                // Template is next (possibly inside single quotes).
                result.push_str(&rest[..tmpl]);
                let m = TEMPLATE_RE.find(rest).unwrap();
                let cap = TEMPLATE_RE.captures(rest).unwrap();
                let sub = make_sub(&cap[1]);
                if in_single_quote {
                    // Break out of single quotes for the substitution, then reopen.
                    result.push('\'');
                    result.push_str(&sub);
                    result.push('\'');
                } else {
                    result.push_str(&sub);
                }
                rest = &rest[m.end()..];
            }
            _ => {
                // No more templates; emit the rest unchanged.
                result.push_str(rest);
                break;
            }
        }
    }
    result
}

/// Substitute `{{N}}` → `$argv[N]` and `{{@}}` → `$argv` for fish shell.
pub fn substitute_fish(cmd: &str) -> String {
    substitute_quote_aware(cmd, |n| match n {
        "@" => "$argv".to_string(),
        n => format!("$argv[{n}]"),
    })
}

/// Substitute `{{N}}` → `$($args[N-1])` and `{{@}}` → `$args` for PowerShell.
pub fn substitute_powershell(cmd: &str) -> String {
    substitute_quote_aware(cmd, |n| match n {
        "@" => "$args".to_string(),
        n => {
            let idx: usize = n.parse::<usize>().unwrap() - 1;
            format!("$($args[{idx}])")
        }
    })
}

/// Substitute `{{N}}` → `"$N"` and `{{@}}` → `"$@"` for bash/zsh.
pub fn substitute_nix(cmd: &str) -> String {
    substitute_quote_aware(cmd, |n| match n {
        "@" => "\"$@\"".to_string(),
        n => format!("\"${n}\""),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::test_helpers::simple;

    #[test]
    fn test_has_template_args_positive() {
        assert!(has_template_args("cm feat: {{@}}"));
        assert!(has_template_args("echo {{1}} and {{2}}"));
        assert!(has_template_args("deploy {{1}}"));
        assert!(has_template_args("mixed {{1}} and {{@}}"));
    }

    #[test]
    fn test_has_template_args_negative() {
        assert!(!has_template_args("git status"));
        assert!(!has_template_args("echo {{abc}}"));
        assert!(!has_template_args("echo {{ 1 }}"));
        assert!(!has_template_args("echo {{0}}"));
        assert!(!has_template_args("echo {{}}"));
        assert!(!has_template_args("echo {{10}}"));
    }

    #[test]
    fn test_substitute_fish() {
        assert_eq!(substitute_fish("cm feat: {{@}}"), "cm feat: $argv");
        assert_eq!(
            substitute_fish("echo {{1}} and {{2}}"),
            "echo $argv[1] and $argv[2]"
        );
    }

    #[test]
    fn test_substitute_nix() {
        assert_eq!(substitute_nix("cm feat: {{@}}"), "cm feat: \"$@\"");
        assert_eq!(
            substitute_nix("echo {{1}} and {{2}}"),
            "echo \"$1\" and \"$2\""
        );
    }

    #[test]
    fn test_substitute_powershell() {
        assert_eq!(substitute_powershell("cm feat: {{@}}"), "cm feat: $args");
        assert_eq!(
            substitute_powershell("echo {{1}} and {{2}}"),
            "echo $($args[0]) and $($args[1])"
        );
    }

    #[test]
    fn test_substitute_leaves_invalid_templates() {
        assert_eq!(substitute_fish("echo {{abc}}"), "echo {{abc}}");
        assert_eq!(substitute_nix("echo {{abc}}"), "echo {{abc}}");
        assert_eq!(substitute_powershell("echo {{abc}}"), "echo {{abc}}");
        assert_eq!(substitute_fish("echo {{0}}"), "echo {{0}}");
    }

    #[test]
    fn test_substitute_breaks_single_quotes_nix() {
        // Template inside a single-quoted string must break quoting so the variable expands.
        assert_eq!(
            substitute_nix("rebase -s 'mega()' -d 'toggle({{1}})'"),
            "rebase -s 'mega()' -d 'toggle('\"$1\"')'"
        );
        assert_eq!(substitute_nix("'{{@}}'"), "''\"$@\"''");
    }

    #[test]
    fn test_substitute_breaks_single_quotes_fish() {
        assert_eq!(
            substitute_fish("rebase -s 'mega()' -d 'toggle({{1}})'"),
            "rebase -s 'mega()' -d 'toggle('$argv[1]')'"
        );
    }

    #[test]
    fn test_substitute_unquoted_template_unchanged_behavior() {
        // Templates outside single quotes work as before.
        assert_eq!(
            substitute_nix("abandon --rev {{1}}"),
            "abandon --rev \"$1\""
        );
        assert_eq!(substitute_fish("log --limit {{@}}"), "log --limit $argv");
    }

    #[test]
    fn test_bash_shell_generates_nix_syntax() {
        let shell: Box<dyn Shell> = Shells::Bash.as_shell(&ShellsTomlConfig::default());
        let entry = simple("gs", "git status");
        assert_eq!(shell.alias(&entry), "gs() { git status \"$@\"; }");
        assert_eq!(shell.unalias("gs"), "unset -f gs");
        assert_eq!(shell.set_env("FOO", "bar"), "export FOO=\"bar\"");
        assert_eq!(shell.unset_env("FOO"), "unset FOO");
    }
}
