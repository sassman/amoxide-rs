use std::fmt::{Debug, Display};

use clap::ValueEnum;

pub trait Shell: Send + Sync + Debug {
    fn unalias(&self, alias_name: &str) -> String;
    fn alias(&self, alias_name: &str, command: &str) -> String;
    fn set_env(&self, var_name: &str, value: &str) -> String;
    fn unset_env(&self, var_name: &str) -> String;
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum Shells {
    // Bash,
    // Elvish,
    Fish,
    // Ksh,
    // Nushell,
    // Posix,
    // Powershell,
    // Xonsh,
    Zsh,
    // #[cfg(windows)]
    // Cmd,
}

impl Shells {
    pub fn as_shell(self) -> Box<dyn Shell> {
        self.into()
    }
}

impl Display for Shells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Shells::Bash => write!(f, "bash"),
            // Shells::Elvish => write!(f, "elvish"),
            Shells::Fish => write!(f, "fish"),
            // Shells::Ksh => write!(f, "ksh"),
            // Shells::Nushell => write!(f, "nushell"),
            // Shells::Posix => write!(f, "posix"),
            // Shells::Powershell => write!(f, "powershell"),
            // Shells::Xonsh => write!(f, "xonsh"),
            Shells::Zsh => write!(f, "zsh"),
        }
    }
}

impl From<Shells> for String {
    fn from(val: Shells) -> Self {
        format!("{}", val)
    }
}

impl From<Shells> for Box<dyn Shell> {
    fn from(shell: Shells) -> Box<dyn Shell> {
        match shell {
            Shells::Zsh => Box::from(super::zsh::Zsh),
            // Shells::Bash => Box::from(super::bash::Bash),
            Shells::Fish => Box::from(super::fish::Fish),
            // Shells::PowerShell => Box::from(super::powershell::PowerShell),
            // #[cfg(windows)]
            // Shells::Cmd => Box::from(super::windows_cmd::WindowsCmd),
        }
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
