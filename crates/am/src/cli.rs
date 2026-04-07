pub use clap::{Args, Parser, Subcommand};

use crate::shell::Shells;

/// amoxide — the alias manager
///
/// Manage your shell aliases — globally, via profiles, or per-project via .aliases files.
/// Activate multiple profiles simultaneously with `am profile use`.
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
        #[arg(short, long, conflicts_with = "global")]
        profile: Option<String>,

        /// Remove a global alias
        #[arg(short, long)]
        global: bool,

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
    ///   Bash        ~/.bashrc                     eval "$(am init bash)"
    ///   Brush       ~/.brushrc                    eval "$(am init brush)"
    ///   Fish        ~/.config/fish/config.fish    am init fish | source
    ///   Nushell     ~/.config/nushell/config.nu   am init nu | source
    ///   PowerShell  $PROFILE                      (am init powershell) -join "`n" | Invoke-Expression
    ///   Zsh         ~/.zshrc                      eval "$(am init zsh)"
    ///
    /// Note: Nushell is not yet supported.
    #[command(verbatim_doc_comment)]
    Init { shell: Shells },

    /// Guided setup — adds amoxide to your shell profile
    Setup { shell: Shells },

    /// Shortcut for `am profile use` — toggle one or more profiles
    #[command(alias = "u")]
    Use {
        /// Profile names
        names: Vec<String>,
        /// Activate at specific priority position (1-based). Repositions if already active.
        #[arg(short = 'n', long = "priority", conflicts_with = "inverse")]
        priority: Option<usize>,
        /// Reverse the processing order (first listed = highest priority)
        #[arg(short, long, conflicts_with = "priority")]
        inverse: bool,
    },

    /// Launch the interactive TUI for managing aliases and profiles
    #[command(alias = "t")]
    Tui,

    /// Export aliases to stdout as TOML
    #[command(alias = "e")]
    Export(ExportArgs),

    /// Import aliases from a URL or file
    #[command(alias = "i")]
    Import(ImportArgs),

    /// Generate a share command for posting aliases to a pastebin service
    #[command(alias = "s")]
    Share(ShareArgs),

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
    },

    /// Toggle one or more profiles as active/inactive, optionally at a specific priority
    #[command(alias = "u")]
    Use {
        /// Profile names
        names: Vec<String>,
        /// Activate at specific priority position (1-based). Repositions if already active.
        #[arg(short = 'n', long = "priority", conflicts_with = "inverse")]
        priority: Option<usize>,
        /// Reverse the processing order (first listed = highest priority)
        #[arg(short, long, conflicts_with = "priority")]
        inverse: bool,
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
    #[arg(short, long, conflicts_with_all = ["local", "global"])]
    pub profile: Option<String>,

    /// Add to the project's .aliases file instead of a profile
    #[arg(short, long, conflicts_with = "global")]
    pub local: bool,

    /// Add as a global alias (always loaded, independent of profile)
    #[arg(short, long)]
    pub global: bool,

    /// Disable {{N}} template detection (treat command as literal)
    #[arg(long)]
    pub raw: bool,

    /// The alias name
    pub name: String,

    /// The command to alias
    pub command: Option<Vec<String>>,
}

/// Shared scope flags for export/import/share commands.
/// Flags can be combined (e.g. `-l -g -p git`) to select multiple scopes.
/// `--all` is a shortcut for all scopes and cannot be combined with others.
#[derive(Args, Debug, Clone)]
pub struct ScopeArgs {
    /// Operate on project-local aliases
    #[arg(short, long, conflicts_with = "all")]
    pub local: bool,

    /// Operate on global aliases
    #[arg(short, long, conflicts_with = "all")]
    pub global: bool,

    /// Operate on specific profile(s) — can be repeated
    #[arg(short, long, conflicts_with = "all")]
    pub profile: Vec<String>,

    /// Operate on everything (global + all profiles + local)
    #[arg(long, conflicts_with_all = ["local", "global", "profile"])]
    pub all: bool,
}

#[derive(Args)]
pub struct ExportArgs {
    #[command(flatten)]
    pub scope: ScopeArgs,

    /// Encode output as base64
    #[arg(short = 'b', long, alias = "b64")]
    pub base64: bool,
}

#[derive(Args)]
pub struct ShareArgs {
    #[command(flatten)]
    pub scope: ScopeArgs,

    /// Generate command for termbin.com (netcat)
    #[arg(long, conflicts_with = "paste_rs")]
    pub termbin: bool,

    /// Generate command for paste.rs (curl)
    #[arg(long, conflicts_with = "termbin")]
    pub paste_rs: bool,
}

#[derive(Args)]
pub struct ImportArgs {
    /// URL or file path to import from
    pub source: String,

    #[command(flatten)]
    pub scope: ScopeArgs,

    /// Decode base64 input before parsing
    #[arg(short = 'b', long, alias = "b64")]
    pub base64: bool,

    /// Skip all confirmation prompts
    #[arg(short = 'y', long = "yes")]
    pub yes: bool,

    /// DANGER: Skip safety checks for suspicious content (escape sequences).
    /// Only use for your own exports. Never trust external input blindly —
    /// it can carry invisible escape sequences that hide malicious commands.
    #[arg(long, requires = "yes")]
    pub trust: bool,
}
