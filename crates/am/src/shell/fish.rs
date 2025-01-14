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
    }
}
