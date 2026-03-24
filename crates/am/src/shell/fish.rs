use std::fmt::Debug;

use super::{has_template_args, quote_cmd, substitute_fish, Shell};
use crate::alias::AliasEntry;

#[derive(Debug, Default)]
pub struct Fish;

impl Shell for Fish {
    fn unalias(&self, alias_name: &str) -> String {
        format!("functions -e {alias_name}")
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        if !entry.raw && has_template_args(entry.command) {
            let body = substitute_fish(entry.command);
            format!("function {}\n    {}\nend", entry.name, body)
        } else {
            format!("alias {} {}", entry.name, quote_cmd(entry.command))
        }
    }

    fn set_env(&self, var_name: &str, value: &str) -> String {
        format!("set -gx {var_name} {}", quote_cmd(value))
    }

    fn unset_env(&self, var_name: &str) -> String {
        format!("set -e {var_name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::AliasEntry;

    fn simple<'a>(name: &'a str, cmd: &'a str) -> AliasEntry<'a> {
        AliasEntry {
            name,
            command: cmd,
            raw: false,
        }
    }

    fn raw<'a>(name: &'a str, cmd: &'a str) -> AliasEntry<'a> {
        AliasEntry {
            name,
            command: cmd,
            raw: true,
        }
    }

    #[test]
    fn test_fish_simple_alias() {
        assert_eq!(
            Fish.alias(&simple("h", "'echo hello'")),
            "alias h 'echo hello'"
        );
        assert_eq!(
            Fish.alias(&simple("h", "echo hello")),
            "alias h \"echo hello\""
        );
    }

    #[test]
    fn test_fish_parameterized() {
        assert_eq!(
            Fish.alias(&simple("cmf", "cm feat: {{@}}")),
            "function cmf\n    cm feat: $argv\nend"
        );
        assert_eq!(
            Fish.alias(&simple("x", "echo {{1}} and {{2}}")),
            "function x\n    echo $argv[1] and $argv[2]\nend"
        );
    }

    #[test]
    fn test_fish_raw_skips_templates() {
        assert_eq!(
            Fish.alias(&raw("my-awk", "awk '{print {{1}}}'")),
            "alias my-awk \"awk '{print {{1}}}'\""
        );
    }

    #[test]
    fn test_fish_unalias() {
        assert_eq!(Fish.unalias("h"), "functions -e h");
    }

    #[test]
    fn test_fish_env() {
        assert_eq!(Fish.set_env("FOO", "bar"), "set -gx FOO \"bar\"");
        assert_eq!(Fish.unset_env("FOO"), "set -e FOO");
    }
}
