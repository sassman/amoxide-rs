pub use clap::{Args, Parser, Subcommand};

/// The Alias-Manager
///
/// The alias-manager (`aman`) is only for the most laziest among you.
/// It helps you to manage your aliases on the shell,
///  - maintain profiles of aliases like work or home
///  - introduce project / directory specific specific aliases
///  - simple backup and restore of profiles and project aliases
///  - synchronize aliases across multiple machines
///  - one approach to manage aliases for all shells
///  - suggestions for aliases based on your history
///  - Terminal UI for managing aliases
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
    #[command(alias = "a")]
    Add(Alias),

    /// Add or activate a profile
    #[command(alias = "p")]
    Profile(Profile),

    /// Print and set up required environment variables for aman
    #[command(alias = "e")]
    Env { shell: String },
}

#[derive(Args)]
pub struct Alias {
    /// The name of the alias
    pub name: String,

    /// The command to be aliased, if not provided, the last command fromm the history will be used
    pub command: Option<String>,
}

#[derive(Args)]
pub struct Profile {
    /// The name of the profile
    pub name: Option<String>,

    /// The optional base profile to inherit from
    #[arg(short, long)]
    pub inherits: Option<String>,

    #[arg(long)]
    pub list: bool,
}
