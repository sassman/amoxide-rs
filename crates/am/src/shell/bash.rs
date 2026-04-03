use super::{NixShell, Shell};
use crate::alias::AliasEntry;

#[derive(Debug, Default)]
pub struct Bash;

impl Shell for Bash {
    fn unalias(&self, alias_name: &str) -> String {
        NixShell.unalias(alias_name)
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        NixShell.alias(entry)
    }

    fn set_env(&self, var_name: &str, value: &str) -> String {
        NixShell.set_env(var_name, value)
    }

    fn unset_env(&self, var_name: &str) -> String {
        NixShell.unset_env(var_name)
    }
}
