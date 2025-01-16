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

    fn env_var(&self, var_name: &str, value: &str) -> String {
        NixShell.env_var(var_name, value)
    }

    // fn last_command_from_history(&self) -> anyhow::Result<String> {
    //     let history_path = home()?.join(".zsh_history");
    //     let history_file = File::open(history_path).context("Failed to open history file")?;
    //     let cmds = lines_from_file(&history_file, 2);

    //     if cmds.is_empty() {
    //         bail!("No commands found in history file");
    //     }

    //     let cmd = cmds[1].trim();
    //     if let Some((_, cmd)) = cmd.split_once(";") {
    //         Ok(cmd.to_string())
    //     } else {
    //         bail!("History file has no commands");
    //     }
    // }

    // fn open_rc_file(&self) -> anyhow::Result<File> {
    //     let path = home()?.join(".zshrc");
    //     File::open(path).context("Failed to open ~/.zshrc file")
    // }
}
