use std::fs::File;

use anyhow::{bail, Context};

use super::{lines_from_file, NixShell, Shell};

#[derive(Debug, Default)]
pub struct Zsh;

impl Shell for Zsh {
    fn render_unalias(&self, unalias: &crate::alias::ShellAlias) -> String {
        NixShell.render_unalias(unalias)
    }

    fn render_alias(&self, alias: &crate::alias::ShellAlias) -> String {
        NixShell.render_alias(alias)
    }

    fn last_command_from_history(&self) -> anyhow::Result<String> {
        let history_file = File::open("~/.zsh_history").context("Failed to open history file")?;
        let cmds = lines_from_file(&history_file, 1);

        if cmds.is_empty() {
            bail!("No commands found in history file");
        }

        Ok(cmds[0].clone())
    }
}
