use std::fmt::Debug;

use crate::shells::{quote_cmd, Shell};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PowerShell;

impl Shell for PowerShell {
    fn render_unalias(&self, unalias: &crate::alias::ShellAlias) -> String {
        format!("Remove-Alias {}", unalias.name)
    }

    fn render_alias(&self, alias: &crate::alias::ShellAlias) -> String {
        format!(
            "New-Alias -Name {name} -Value {cmd}",
            cmd = quote_cmd(&alias.value),
            name = alias.name,
        )
    }

    fn last_command_from_history(&self) -> anyhow::Result<String> {
        todo!("Implement last_command_from_history for PowerShell")
    }
}

#[cfg(test)]
mod tests {
    use crate::alias::ShellAlias;

    use super::*;

    #[test]
    fn test_alias_to_string_zsh() {
        let alias = ShellAlias {
            name: "h".to_string(),
            value: "echo hello".to_string(),
        };

        assert_eq!(
            &PowerShell.render_alias(&alias),
            "New-Alias -Name h -Value \"echo hello\""
        );
        assert_eq!(&PowerShell.render_unalias(&alias), "Remove-Alias h");
    }

    #[test]
    fn test_alias_with_single_quotes() {
        let alias = ShellAlias {
            name: "h".to_string(),
            value: "'echo hello'".to_string(),
        };

        assert_eq!(
            &PowerShell.render_alias(&alias),
            "New-Alias -Name h -Value 'echo hello'"
        );
        assert_eq!(&PowerShell.render_unalias(&alias), "Remove-Alias h");
    }
}
