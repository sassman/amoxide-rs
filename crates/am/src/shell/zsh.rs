use super::{NixShell, Shell};
use crate::alias::AliasEntry;

#[derive(Debug, Default)]
pub struct Zsh;

impl Shell for Zsh {
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

    fn echo(&self, message: &str) -> String {
        NixShell.echo(message)
    }

    fn subcommand_wrapper(
        &self,
        program: &str,
        base_cmd: &str,
        entries: &[crate::subcommand::SubcommandEntry],
    ) -> String {
        NixShell.subcommand_wrapper(program, base_cmd, entries)
    }
}
