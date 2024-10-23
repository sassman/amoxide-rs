use std::fmt::Debug;

use crate::alias::ShellAlias;
use crate::shells::{quote_cmd, Shell};

#[derive(Debug, Default)]
pub struct NixShell;

impl Shell for NixShell {
    fn render_unalias(&self, unalias: &ShellAlias) -> String {
        format!("unalias {}", unalias.name)
    }

    fn render_alias(&self, alias: &ShellAlias) -> String {
        format!(
            "alias {name}={cmd}",
            cmd = quote_cmd(&alias.value),
            name = &alias.name
        )
    }

    fn last_command_from_history(&self) -> anyhow::Result<String> {
        todo!("Implement last_command_from_history for NixShell")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_to_string_zsh() {
        let alias = ShellAlias {
            name: "h".to_string(),
            value: "echo hello".to_string(),
        };

        assert_eq!(&NixShell.render_alias(&alias), "alias h=\"echo hello\"");
        assert_eq!(&NixShell.render_unalias(&alias), "unalias h");
    }

    #[test]
    fn test_alias_with_single_quotes() {
        let alias = ShellAlias {
            name: "h".to_string(),
            value: "'echo hello'".to_string(),
        };

        assert_eq!(&NixShell.render_alias(&alias), "alias h='echo hello'");
        assert_eq!(&NixShell.render_unalias(&alias), "unalias h");
    }
}
