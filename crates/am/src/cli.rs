pub use clap::{Args, Parser, Subcommand};

use crate::shell::Shells;

/// The Alias-Manager
///
/// The alias-manager (`am`) is only for the most laziest among you.
/// It helps you to manage your aliases on the shell,
///  - [WIP] maintain profiles of aliases like `work`, `home` or technology specific profiles
///  - [ ] introduce project / directory specific specific aliases
///  - [ ] simple backup and restore of profiles and project aliases
///  - [ ] synchronize aliases across multiple machines
///  - [ ] one approach to manage aliases for all shells
///  - [ ] suggestions for aliases based on your history
///  - [ ] Terminal UI for managing aliases
#[derive(Parser)]
#[command(name = "am")]
#[command(about = "The Alias-Manager", long_about = None, version, author)]
#[command(propagate_version = true)]
pub struct Cli {
    #[arg(env = "_AM_SHELL", hide = true)]
    pub shell: Option<Shells>,

    #[arg(env = "_AM_SESSION_KEY", hide = true)]
    pub session_key: Option<String>,

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

    /// Print and set up required environment variables for am
    #[command(alias = "e")]
    #[clap(value_enum)]
    Env { shell: Shells },

    #[command(alias = "i")]
    #[clap(value_enum)]
    Init { shell: Shells },
}

#[derive(Args)]
pub struct Alias {
    /// The name of the profile to add the alias to, if not provided, the active profile will be used.
    /// If no profile is active, the default profile will be used.
    ///
    /// :warning: The active profile is not yet implemented, so the default profile will be used.
    #[arg(short, long)]
    pub profile: Option<String>,

    /// The name of the alias
    pub name: String,

    /// The command to be aliased, if not provided, the last command fromm the history will be used
    pub command: Option<Vec<String>>,
}

#[derive(Args)]
pub struct Profile {
    /// The name of the profile
    /// If omitted, the active profile will be used from the env var `_AM_ACTIVE_PROFILE`
    #[arg(env = "_AM_ACTIVE_PROFILE", hide = true)]
    pub name: Option<String>,

    /// The optional base profile to inherit from
    #[arg(short, long)]
    pub inherits: Option<String>,

    /// Execute this on activation of the profile
    #[arg(long)]
    pub on_activate: Option<String>,

    #[arg(long)]
    pub list: bool,

    #[arg(long)]
    pub print_full_init: bool,
}
