use std::fmt::Debug;

use super::{quote_cmd, Shell};

#[derive(Debug, Default)]
pub struct NixShell;

impl Shell for NixShell {
    fn unalias(&self, alias_name: &str) -> String {
        format!("unalias {alias_name}")
    }

    fn alias(&self, alias_name: &str, command: &str) -> String {
        format!("alias {alias_name}={cmd}", cmd = quote_cmd(command),)
    }

    fn set_env(&self, var_name: &str, value: &str) -> String {
        format!("export {var_name}={}", quote_cmd(value))
    }

    fn unset_env(&self, var_name: &str) -> String {
        format!("unset {var_name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_to_string() {
        assert_eq!(
            &NixShell.alias("h", "'echo hello'"),
            "alias h='echo hello'",
            "single quotes messed up"
        );

        assert_eq!(&NixShell.alias("h", "echo hello"), "alias h=\"echo hello\"");
        assert_eq!(&NixShell.unalias("h"), "unalias h");
        assert_eq!(&NixShell.set_env("FOO", "bar"), "export FOO=\"bar\"");
        assert_eq!(&NixShell.unset_env("FOO"), "unset FOO");
    }
}
