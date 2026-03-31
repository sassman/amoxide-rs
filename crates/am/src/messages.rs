use std::fmt::Display;

use crate::shell::Shells;

#[derive(Debug, PartialEq)]
pub enum AliasTarget {
    Profile(String),
    ActiveProfile,
    Global,
    Local,
}

impl Display for AliasTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AliasTarget::Profile(profile) => write!(f, "Profile: `{}`", profile),
            AliasTarget::ActiveProfile => write!(f, "Active Profile"),
            AliasTarget::Global => write!(f, "Global"),
            AliasTarget::Local => write!(f, "Local"),
        }
    }
}

#[derive(Debug)]
pub enum Message {
    AddAlias(String, String, AliasTarget, bool),
    RemoveAlias(String, AliasTarget),
    InitShell(Shells),
    Hook(Shells),
    Reload(Shells),

    ToggleProfile(String),
    UseProfileAt(String, usize),
    RemoveProfile(String),
    ListProfiles,
    CreateProfile(String),
}
