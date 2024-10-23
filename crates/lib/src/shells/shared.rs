use std::{fmt::Debug, fs::File, io::BufRead};

use anyhow::bail;
use rev_buf_reader::RevBufReader;

use crate::alias::ShellAlias;

use super::{zsh::Zsh, NixShell};

pub trait Shell: Send + Sync + Debug {
    fn render_unalias(&self, unalias: &ShellAlias) -> String;
    fn render_alias(&self, alias: &ShellAlias) -> String;
    fn last_command_from_history(&self) -> anyhow::Result<String>;
}

pub struct ShellBuilder;

impl ShellBuilder {
    pub fn build_current(self) -> anyhow::Result<Box<dyn Shell>> {
        current_shell()
    }
}

fn current_shell() -> anyhow::Result<Box<dyn Shell>> {
    // try out if it's zsh, so we expect $SHELL to be set
    if let Ok(zsh) = std::env::var("SHELL") {
        match zsh.as_str() {
            "/bin/zsh" => return Ok(Box::new(Zsh)),
            "/bin/bash" => return Ok(Box::new(NixShell)),
            "/bin/sh" => return Ok(Box::new(NixShell)),
            sh => bail!("Unsupported shell: {sh}"),
        }
    }

    // try to check if we got a bash or sh shelll so we check $BASH
    if let Ok(bash) = std::env::var("BASH") {
        match bash.as_str() {
            "/bin/zsh" => return Ok(Box::new(Zsh)),
            "/bin/bash" => return Ok(Box::new(NixShell)),
            "/bin/sh" => return Ok(Box::new(NixShell)),
            sh => bail!("Unsupported shell: {sh}"),
        }
    }

    // not sure how to check if powershell is the current shell

    // "powershell" => Ok(Box::new(PowerShell)),
    bail!("Unsupported shell");
}

pub fn quote_cmd(cmd: &str) -> String {
    let (cmd, quotes) = if cmd.starts_with("'") && cmd.ends_with("'") {
        (&cmd[1..cmd.len() - 1], "'")
    } else {
        (cmd, "\"")
    };

    format!("{quotes}{cmd}{quotes}")
}

pub fn lines_from_file(file: &File, limit: usize) -> Vec<String> {
    let buf = RevBufReader::new(file);
    buf.lines()
        .take(limit)
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_shell() {
        let x = current_shell().unwrap();
        assert_eq!(format!("{:?}", x), format!("{:?}", NixShell));
    }

    #[test]
    fn test_shell_builder() {
        let x = ShellBuilder.build_current().unwrap();
        assert_eq!(format!("{:?}", x), format!("{:?}", NixShell));
    }

    #[test]
    fn test_quote_cmd() {
        let cmd = "echo hello";
        assert_eq!(quote_cmd(cmd), "\"echo hello\"");

        let cmd = "'echo hello'";
        assert_eq!(quote_cmd(cmd), "'echo hello'");
    }
}
