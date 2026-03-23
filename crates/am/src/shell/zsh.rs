use super::{NixShell, Shell};

#[derive(Debug, Default)]
pub struct Zsh;

impl Shell for Zsh {
    fn unalias(&self, alias_name: &str) -> String {
        NixShell.unalias(alias_name)
    }

    fn alias(&self, alias_name: &str, command: &str) -> String {
        NixShell.alias(alias_name, command)
    }

    fn set_env(&self, var_name: &str, value: &str) -> String {
        NixShell.set_env(var_name, value)
    }

    fn unset_env(&self, var_name: &str) -> String {
        NixShell.unset_env(var_name)
    }
}
