use std::fmt::Debug;

use super::{quote_cmd, Shell};

#[derive(Debug, Default)]
pub struct Fish;

impl Shell for Fish {
    fn unalias(&self, alias_name: &str) -> String {
        format!("functions -e {alias_name}")
    }

    fn alias(&self, alias_name: &str, command: &str) -> String {
        format!("alias {alias_name} {}", quote_cmd(command),)
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

    #[test]
    fn test_alias_to_string_zsh() {
        assert_eq!(
            &Fish.alias("h", "'echo hello'"),
            "alias h 'echo hello'",
            "single quotes messed up"
        );

        assert_eq!(&Fish.alias("h", "echo hello"), "alias h \"echo hello\"");
        assert_eq!(&Fish.unalias("h"), "functions -e h");
        assert_eq!(&Fish.set_env("FOO", "bar"), "set -gx FOO \"bar\"");
        assert_eq!(&Fish.unset_env("FOO"), "set -e FOO");
    }
}
