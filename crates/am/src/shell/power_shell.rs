use std::fmt::Debug;

use super::shell::{quote_cmd, Shell};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PowerShell;

impl Shell for PowerShell {
    fn unalias(&self, alias_name: &str) -> String {
        format!("Remove-Alias {alias_name}")
    }

    fn alias(&self, alias_name: &str, command: &str) -> String {
        format!(
            "New-Alias -Name {alias_name} -Value {cmd}",
            cmd = quote_cmd(command),
        )
    }

    fn env_var(&self, var_name: &str, value: &str) -> String {
        format!("Set-Variable -Name {var_name} -Value {}", quote_cmd(value),)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_to_string() {
        assert_eq!(
            &PowerShell.alias("h", "'echo hello'"),
            "New-Alias -Name h -Value 'echo hello'",
            "single quotes messed up"
        );

        assert_eq!(
            &PowerShell.alias("h", "echo hello"),
            "New-Alias -Name h -Value \"echo hello\""
        );
        assert_eq!(&PowerShell.unalias("h"), "Remove-Alias h");
    }
}
