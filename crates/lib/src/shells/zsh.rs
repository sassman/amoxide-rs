use std::{fs::File, path::PathBuf};

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
        let home = std::env::var("HOME").context("Failed to get home directory")?;
        let history_path = PathBuf::from(home).join(".zsh_history");
        let history_file = File::open(history_path).context("Failed to open history file")?;
        let cmds = lines_from_file(&history_file, 1);

        if cmds.is_empty() {
            bail!("No commands found in history file");
        }

        let cmd = cmds[0].trim();
        if let Some((_, cmd)) = cmd.split_once(";") {
            Ok(cmd.to_string())
        } else {
            bail!("History file has no command");
        }
    }
}
