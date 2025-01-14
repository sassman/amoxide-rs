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

#[derive(Debug, PartialEq)]
pub enum Message<'a> {
    AddAlias(String, String, AddAliasProfile),
    AddProfile(String, Option<String>),
    SetEnv(String),
    InitShell(&'a Shells),

    ListProfiles,
    LoadOrCreateProfile(&'a str, &'a Option<String>),
    SaveProfiles,
    ListAliasesForShell(&'a Shells),
}
