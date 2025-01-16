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

    fn env_var(&self, var_name: &str, value: &str) -> String {
        format!("export {var_name}={}", quote_cmd(value),)
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
    }
}
