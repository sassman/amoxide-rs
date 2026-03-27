use std::fmt::Debug;

use super::{has_template_args, substitute_powershell, Shell};
use crate::alias::AliasEntry;

#[derive(Debug, Default)]
pub struct PowerShell;

impl Shell for PowerShell {
    fn unalias(&self, alias_name: &str) -> String {
        format!("Remove-Item -ErrorAction SilentlyContinue Function:{alias_name}")
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        if !entry.raw && has_template_args(entry.command) {
            let body = substitute_powershell(entry.command);
            format!("function {} {{ {} }}", entry.name, body)
        } else {
            format!("function {} {{ {} @args }}", entry.name, entry.command)
        }
    }

    fn set_env(&self, var_name: &str, value: &str) -> String {
        format!("$env:{var_name} = \"{value}\"")
    }

    fn unset_env(&self, var_name: &str) -> String {
        format!("Remove-Item -ErrorAction SilentlyContinue Env:{var_name}")
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
    fn test_simple_alias() {
        assert_eq!(
            PowerShell.alias(&simple("gs", "git status")),
            "function gs { git status @args }"
        );
    }

    #[test]
    fn test_parameterized_alias() {
        assert_eq!(
            PowerShell.alias(&simple("cmf", "cm feat: {{@}}")),
            "function cmf { cm feat: $args }"
        );
        assert_eq!(
            PowerShell.alias(&simple("x", "echo {{1}} and {{2}}")),
            "function x { echo $($args[0]) and $($args[1]) }"
        );
    }

    #[test]
    fn test_raw_alias() {
        assert_eq!(
            PowerShell.alias(&raw("my-awk", "awk '{print {{1}}}'")),
            "function my-awk { awk '{print {{1}}}' @args }"
        );
    }

    #[test]
    fn test_unalias() {
        assert_eq!(
            PowerShell.unalias("gs"),
            "Remove-Item -ErrorAction SilentlyContinue Function:gs"
        );
    }

    #[test]
    fn test_set_env() {
        assert_eq!(PowerShell.set_env("FOO", "bar"), "$env:FOO = \"bar\"");
    }

    #[test]
    fn test_unset_env() {
        assert_eq!(
            PowerShell.unset_env("FOO"),
            "Remove-Item -ErrorAction SilentlyContinue Env:FOO"
        );
    }
}
