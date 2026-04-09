use std::fmt::{Debug, Display};
use std::sync::LazyLock;

use clap::ValueEnum;
use regex::Regex;

pub trait Shell: Send + Sync + Debug {
    fn unalias(&self, alias_name: &str) -> String;
    fn alias(&self, entry: &crate::alias::AliasEntry) -> String;
    fn set_env(&self, var_name: &str, value: &str) -> String;
    fn unset_env(&self, var_name: &str) -> String;
    fn echo(&self, message: &str) -> String;
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

impl Shells {
    pub fn as_shell(self) -> Box<dyn Shell> {
        self.into()
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

impl From<Shells> for Box<dyn Shell> {
    fn from(shell: Shells) -> Box<dyn Shell> {
        match shell {
            Shells::Zsh => Box::from(super::zsh::Zsh),
            Shells::Bash => Box::from(super::bash::Bash),
            Shells::Brush => Box::from(super::brush::Brush),
            Shells::Fish => Box::from(super::fish::Fish),
            Shells::Powershell => Box::from(super::powershell::PowerShell),
            // #[cfg(windows)]
            // Shells::Cmd => Box::from(super::windows_cmd::WindowsCmd),
        }
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

static TEMPLATE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{\{([1-9]|@)\}\}").unwrap());

/// Returns true if the command contains valid template args like `{{1}}`, `{{@}}`.
pub fn has_template_args(cmd: &str) -> bool {
    TEMPLATE_RE.is_match(cmd)
}

/// Substitute `{{N}}` → `$argv[N]` and `{{@}}` → `$argv` for fish shell.
pub fn substitute_fish(cmd: &str) -> String {
    TEMPLATE_RE
        .replace_all(cmd, |caps: &regex::Captures| match &caps[1] {
            "@" => "$argv".to_string(),
            n => format!("$argv[{n}]"),
        })
        .to_string()
}

/// Substitute `{{N}}` → `$($args[N-1])` and `{{@}}` → `$args` for PowerShell.
pub fn substitute_powershell(cmd: &str) -> String {
    TEMPLATE_RE
        .replace_all(cmd, |caps: &regex::Captures| match &caps[1] {
            "@" => "$args".to_string(),
            n => {
                let idx: usize = n.parse::<usize>().unwrap() - 1;
                format!("$($args[{idx}])")
            }
        })
        .to_string()
}

/// Substitute `{{N}}` → `"$N"` and `{{@}}` → `"$@"` for bash/zsh.
pub fn substitute_nix(cmd: &str) -> String {
    TEMPLATE_RE
        .replace_all(cmd, |caps: &regex::Captures| match &caps[1] {
            "@" => "\"$@\"".to_string(),
            n => format!("\"${n}\""),
        })
        .to_string()
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
    fn test_bash_shell_generates_nix_syntax() {
        let shell: Box<dyn Shell> = Shells::Bash.as_shell();
        let entry = simple("gs", "git status");
        assert_eq!(shell.alias(&entry), "gs() { git status \"$@\"; }");
        assert_eq!(shell.unalias("gs"), "unset -f gs");
        assert_eq!(shell.set_env("FOO", "bar"), "export FOO=\"bar\"");
        assert_eq!(shell.unset_env("FOO"), "unset FOO");
    }
}
