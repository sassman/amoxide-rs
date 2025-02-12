use std::fmt::Display;

use crate::shell::Shells;

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq)]
pub enum Message<'a> {
    AddAlias(String, String, AddAliasProfile),
    AddProfile(String, Option<String>),
    SetEnv(String),
    InitShell(&'a Shells),

    ActivateProfile(&'a str),
    SetShell(&'a Shells),

    ListProfiles,
    CreateOrUpdateProfile(&'a str, &'a Option<String>),
    SaveProfiles,

    ListActiveAliases,
    DoNothing,

    RestoreState(&'a str), // by session key
    SaveState(&'a str),    // by session key
}
