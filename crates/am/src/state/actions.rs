use crate::{shell::Shells, AddAliasProfile};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    DoNothing,
    AddAlias(String, String, AddAliasProfile),
    AddProfile(String, Option<String>),
    SetEnv(String),
    ListProfiles,
    CreateOrUpdateProfile(String, Option<String>),
    SaveProfiles,
    InitShell(Shells),
    SetShell(Shells),
    ActivateProfile(String),
    ListActiveAliases,
    RestoreState(String), // this is a session key, todo: introduce a new type for this
    SaveState(String),    // same as above
}
