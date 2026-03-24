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
    AddAlias(String, String, AddAliasProfile),
    RemoveAlias(String, AddAliasProfile),
    InitShell(Shells),
    Hook(Shells),

    ActivateProfile(String),
    RemoveProfile(String),
    ListProfiles,
    CreateOrUpdateProfile(String, Option<String>),
    SaveProfiles,
    SaveConfig,

    DoNothing,
}
