use std::{fmt::Debug, fs::File, io::BufRead};

use anyhow::bail;
use log::info;
use rev_buf_reader::RevBufReader;

use crate::alias::ShellAlias;

use super::{zsh::Zsh, NixShell, PowerShell};

pub trait Shell: Send + Sync + Debug {
    fn render_unalias(&self, unalias: &ShellAlias) -> String;
    fn render_alias(&self, alias: &ShellAlias) -> String;
    fn last_command_from_history(&self) -> anyhow::Result<String>;
}

pub struct ShellBuilder {
    try_current: bool,
    name: Option<String>,
}

impl ShellBuilder {
    pub fn new() -> Self {
        Self {
            try_current: false,
            name: None,
        }
    }

    /// Configure the builder to use the current shell, this might fail
    pub fn guess(mut self) -> Self {
        self.try_current = true;
        self.name = None;

        self
    }

    /// Set the shell name to build
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_lowercase().to_string());
        self.try_current = false;
        self
    }

    /// Try to build the shell from the builders configuration
    pub fn build(self) -> anyhow::Result<Box<dyn Shell>> {
        if let Some(name) = &self.name {
            match name.as_str() {
                "zsh" => Ok(Box::new(Zsh)),
                "bash" | "sh" | "fish" => Ok(Box::new(NixShell)),
                "powershell" => Ok(Box::new(PowerShell)),
                _ => bail!("Unsupported shell"),
            }
        } else {
            current_shell()
        }
    }
}

fn current_shell() -> anyhow::Result<Box<dyn Shell>> {
    // try out if it's zsh, so we expect $SHELL to be set
    if let Ok(zsh_version) = std::env::var("ZSH_VERSION") {
        info!("Detected zsh version: {}", zsh_version);
        Ok(Box::new(Zsh))
    } else if let Ok(bash) = std::env::var("BASH") {
        // try to check if we got a bash or sh shell so we check $BASH
        match bash.as_str() {
            "/bin/zsh" => Ok(Box::new(Zsh)),
            "/bin/bash" => Ok(Box::new(NixShell)),
            "/bin/sh" => Ok(Box::new(NixShell)),
            sh => bail!("Unsupported shell: {sh}"),
        }
    } else {
        info!("Could not detect shell, trying to guess");
        // Ok(Box::new(NixShell))
        // todo: not sure how to check if powershell is the current shell
        // "powershell" => Ok(Box::new(PowerShell)),
        bail!("Unrecognized shell");
    }
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
    use crate::shells::zsh::Zsh;

    #[test]
    fn test_current_shell() {
        let x = current_shell().unwrap();
        assert_eq!(format!("{:?}", x), format!("{:?}", Zsh));
    }

    #[test]
    fn test_shell_builder() {
        let x = ShellBuilder::new().guess().build().unwrap();
        assert_eq!(format!("{:?}", x), format!("{:?}", Zsh));
    }

    #[test]
    fn test_shell_builder_by_name() {
        let x = ShellBuilder::new().with_name("zsh").build().unwrap();
        assert_eq!(format!("{:?}", x), format!("{:?}", Zsh));
    }

    #[test]
    fn test_quote_cmd() {
        let cmd = "echo hello";
        assert_eq!(quote_cmd(cmd), "\"echo hello\"");

        let cmd = "'echo hello'";
        assert_eq!(quote_cmd(cmd), "'echo hello'");
    }
}
