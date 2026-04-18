use std::fmt::Display;

use crate::exchange::ImportPayload;
use crate::shell::Shell;

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
    /// Update an alias in place — renames, changes command, or both.
    UpdateAlias {
        target: AliasTarget,
        old_name: String,
        new_name: String,
        new_command: String,
        raw: bool,
    },
    InitShell(Shell, bool),
    Hook(Shell, bool),
    Reload(Shell),

    ToggleProfiles(Vec<String>),
    UseProfilesAt(Vec<String>, usize),
    RemoveProfile(String),
    ListProfiles {
        used: bool,
    },
    CreateProfile(String),
    /// Rename a profile, preserving its aliases and activation state.
    RenameProfile {
        old_name: String,
        new_name: String,
    },

    Import(ImportPayload),

    AddSubcommandAlias(String, Vec<String>, AliasTarget),
    RemoveSubcommandAlias(String, AliasTarget),
    UpdateSubcommandAlias {
        original_key: String,
        new_key: String,
        long_subcommands: Vec<String>,
        target: AliasTarget,
    },
    CopySubcommandAliases {
        keys: Vec<String>,
        from: AliasTarget,
        to: AliasTarget,
    },
    MoveSubcommandAliases {
        keys: Vec<String>,
        from: AliasTarget,
        to: AliasTarget,
    },

    Trust,
    Untrust {
        forget: bool,
    },

    /// Move one or more aliases to another scope (source deleted).
    MoveAliases {
        aliases: Vec<crate::AliasId>,
        to: AliasTarget,
    },

    /// Copy one or more aliases to another scope (source retained).
    CopyAliases {
        aliases: Vec<crate::AliasId>,
        to: AliasTarget,
    },
}
