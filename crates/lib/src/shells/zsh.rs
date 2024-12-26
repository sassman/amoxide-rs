use anyhow::{bail, Context};
use std::fs::File;

use super::{home, lines_from_file, NixShell, Shell};

#[derive(Debug, Default)]
pub struct Zsh;

impl Zsh {}

impl Shell for Zsh {
    fn render_unalias(&self, unalias: &crate::alias::ShellAlias) -> String {
        NixShell.render_unalias(unalias)
    }

    fn render_alias(&self, alias: &crate::alias::ShellAlias) -> String {
        NixShell.render_alias(alias)
    }

    fn last_command_from_history(&self) -> anyhow::Result<String> {
        let history_path = home()?.join(".zsh_history");
        let history_file = File::open(history_path).context("Failed to open history file")?;
        let cmds = lines_from_file(&history_file, 2);

        if cmds.is_empty() {
            bail!("No commands found in history file");
        }

        let cmd = cmds[1].trim();
        if let Some((_, cmd)) = cmd.split_once(";") {
            Ok(cmd.to_string())
        } else {
            bail!("History file has no commands");
        }
    }

    fn open_rc_file(&self) -> anyhow::Result<File> {
        let path = home()?.join(".zshrc");
        File::open(path).context("Failed to open ~/.zshrc file")
    }
}
