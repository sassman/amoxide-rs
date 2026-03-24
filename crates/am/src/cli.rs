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

    /// Remove an alias
    #[command(alias = "r")]
    Remove {
        /// Profile to remove the alias from (defaults to active profile)
        #[arg(short, long)]
        profile: Option<String>,

        /// The alias name to remove
        name: String,
    },

    /// List all profiles and project aliases
    #[command(alias = "l")]
    Ls,

    /// Check if the shell is set up correctly
    Status,

    /// Manage profiles (defaults to listing when no subcommand given)
    #[command(alias = "p")]
    Profile {
        #[command(subcommand)]
        action: Option<ProfileAction>,
    },

    /// Print shell init code
    ///
    /// This outputs shell code that loads your profile aliases and installs
    /// a cd hook for automatic project alias loading. Add one of these lines
    /// to your shell's config file:
    ///
    ///   Fish        ~/.config/fish/config.fish    am init fish | source
    ///   Zsh         ~/.zshrc                      eval "$(am init zsh)"
    ///   Bash        ~/.bashrc                     eval "$(am init bash)"
    ///   Nushell     ~/.config/nushell/config.nu   am init nu | source
    ///   PowerShell  $PROFILE                      am init powershell | Invoke-Expression
    ///
    /// Note: Only fish and zsh are currently supported. Others are planned.
    #[command(alias = "i", verbatim_doc_comment)]
    Init { shell: Shells },

    /// Internal: called by the cd hook to load/unload project aliases
    #[command(hide = true)]
    Hook { shell: Shells },

    /// Internal: called by the am wrapper to reload profile aliases after switching
    #[command(hide = true)]
    Reload { shell: Shells },
}

#[derive(Subcommand)]
pub enum ProfileAction {
    /// Add a new profile
    #[command(alias = "a")]
    Add {
        /// Profile name
        name: String,

        /// Base profile to inherit from
        #[arg(short, long)]
        inherits: Option<String>,
    },

    /// Set the active profile
    #[command(alias = "s")]
    Set {
        /// Profile name
        name: String,
    },

    /// Remove a profile
    #[command(alias = "r")]
    Remove {
        /// Profile name
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },

    /// List all profiles
    #[command(alias = "l")]
    List,
}

#[derive(Args)]
pub struct Alias {
    /// Profile to add the alias to (defaults to active profile)
    #[arg(short, long, conflicts_with = "local")]
    pub profile: Option<String>,

    /// Add to the project's .aliases file instead of a profile
    #[arg(short, long)]
    pub local: bool,

    /// Disable {{N}} template detection (treat command as literal)
    #[arg(long)]
    pub raw: bool,

    /// The alias name
    pub name: String,

    /// The command to alias
    pub command: Option<Vec<String>>,
}
