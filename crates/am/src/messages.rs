use std::fmt::Display;

use crate::shell::Shells;

#[derive(Debug, PartialEq)]
pub enum AddAliasProfile {
    Profile(String),
    ActiveProfile,
}

impl Display for AddAliasProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddAliasProfile::Profile(profile) => write!(f, "Profile: `{}`", profile),
            AddAliasProfile::ActiveProfile => write!(f, "Active Profile"),
        }
    }
}

#[derive(Debug)]
pub enum Message {
    AddAlias(String, String, AddAliasProfile, bool),
    RemoveAlias(String, AddAliasProfile),
    InitShell(Shells),
    Hook(Shells),
    Reload(Shells),

    ToggleProfile(String),
    UseProfileAt(String, usize),
    RemoveProfile(String),
    ListProfiles,
    CreateProfile(String),
    SaveProfiles,
    SaveConfig,

    DoNothing,
}
