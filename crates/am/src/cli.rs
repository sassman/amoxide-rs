pub use clap::{Args, Parser, Subcommand};

use crate::shell::Shells;

/// The Alias-Manager
///
/// Manage your shell aliases — globally via profiles or per-project via .aliases files.
#[derive(Parser)]
#[command(name = "am")]
#[command(about = "The Alias-Manager", long_about = None, version, author)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new alias
    #[command(alias = "a", trailing_var_arg = true)]
    Add(Alias),

    /// Add or activate a profile
    #[command(alias = "p")]
    Profile(Profile),

    /// List all profiles
    #[command(alias = "l")]
    Profiles,

    /// Print shell init code (eval in your shell rc file)
    #[command(alias = "i")]
    Init { shell: Shells },

    /// Internal: called by the cd hook to load/unload project aliases
    #[command(hide = true)]
    Hook { shell: Shells },
}

#[derive(Args)]
pub struct Alias {
    /// Profile to add the alias to (defaults to active profile)
    #[arg(short, long)]
    pub profile: Option<String>,

    /// The alias name
    pub name: String,

    /// The command to alias
    pub command: Option<Vec<String>>,
}

#[derive(Args)]
pub struct Profile {
    /// Profile name
    pub name: Option<String>,

    /// Base profile to inherit from
    #[arg(short, long)]
    pub inherits: Option<String>,

    /// List all profiles
    #[arg(long)]
    pub list: bool,
}
