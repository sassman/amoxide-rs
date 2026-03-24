use std::fmt::Debug;

use super::{has_template_args, substitute_nix, Shell};
use crate::alias::AliasEntry;

#[derive(Debug, Default)]
pub struct NixShell;

impl Shell for NixShell {
    fn unalias(&self, alias_name: &str) -> String {
        format!("unset -f {alias_name}")
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        if !entry.raw && has_template_args(entry.command) {
            let body = substitute_nix(entry.command);
            format!("{}() {{ {}; }}", entry.name, body)
        } else {
            format!("{}() {{ {} \"$@\"; }}", entry.name, entry.command)
        }
    }

    fn set_env(&self, var_name: &str, value: &str) -> String {
        format!("export {var_name}=\"{value}\"")
    }

    fn unset_env(&self, var_name: &str) -> String {
        format!("unset {var_name}")
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
    fn test_nix_simple() {
        assert_eq!(
            NixShell.alias(&simple("h", "echo hello")),
            "h() { echo hello \"$@\"; }"
        );
    }

    #[test]
    fn test_nix_parameterized() {
        assert_eq!(
            NixShell.alias(&simple("cmf", "cm feat: {{@}}")),
            "cmf() { cm feat: \"$@\"; }"
        );
        assert_eq!(
            NixShell.alias(&simple("x", "echo {{1}} and {{2}}")),
            "x() { echo \"$1\" and \"$2\"; }"
        );
    }

    #[test]
    fn test_nix_raw_skips_templates() {
        assert_eq!(
            NixShell.alias(&raw("my-awk", "awk '{print {{1}}}'")),
            "my-awk() { awk '{print {{1}}}' \"$@\"; }"
        );
    }

    #[test]
    fn test_nix_unalias() {
        assert_eq!(NixShell.unalias("h"), "unset -f h");
    }

    #[test]
    fn test_nix_env() {
        assert_eq!(NixShell.set_env("FOO", "bar"), "export FOO=\"bar\"");
        assert_eq!(NixShell.unset_env("FOO"), "unset FOO");
    }
}
