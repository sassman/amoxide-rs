pub use clap::{Args, Parser, Subcommand};

use crate::shell::Shell;

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
        #[arg(short, long, conflicts_with_all = ["local", "global"])]
        profile: Option<String>,

        /// Remove from the project's .aliases file instead of a profile
        #[arg(short, long, conflicts_with = "global")]
        local: bool,

        /// Remove a global alias
        #[arg(short, long)]
        global: bool,

        /// The alias name to remove
        name: String,

        /// Subcommand path segments to complete the key (e.g. --sub b --sub l removes jj:b:l)
        #[arg(long = "sub")]
        sub: Vec<String>,
    },

    /// List all profiles and project aliases
    #[command(alias = "l")]
    Ls {
        /// Show only active profiles and loaded project aliases
        #[arg(short, long, default_value_t = false)]
        used: bool,
    },

    /// Check if the shell is set up correctly
    Status,

    /// Manage profiles (defaults to listing when no subcommand given)
    #[command(alias = "p")]
    Profile {
        #[command(subcommand)]
        action: Option<ProfileAction>,
    },

    /// Manage alias variables — substituted as `{{name}}` in alias commands.
    #[command(alias = "v")]
    Var {
        #[command(subcommand)]
        action: VarAction,
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
    Init {
        /// Force re-initialisation: unload all previously tracked aliases (both
        /// alias and function forms) before re-loading. Use after config changes
        /// such as toggling `use_abbr`.
        #[arg(short = 'f', long)]
        force: bool,
        shell: Shell,
    },

    /// Guided setup — adds amoxide to your shell profile
    Setup { shell: Shell },

    /// Shortcut for `am profile use` — toggle one or more profiles
    #[command(alias = "u")]
    Use(ProfileUse),

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

    /// Review and trust the project .aliases file in the current directory
    Trust,

    /// Remove trust for the project .aliases file in the current directory
    Untrust {
        /// Forget the path entirely (remove from security tracking instead of marking untrusted)
        #[arg(short, long)]
        forget: bool,
    },

    /// Internal: compute and emit the minimal shell ops to sync the shell with
    /// the effective merged alias state (global + profile + project).
    #[command(hide = true)]
    Sync {
        /// Suppress info and warning messages (still unloads/loads aliases).
        #[arg(short, long)]
        quiet: bool,
        shell: Shell,
    },
}

#[derive(Args)]
pub struct ProfileUse {
    /// Enable given profile(s), does not toggle
    #[arg(short = 'e', long = "enable", conflicts_with = "disable")]
    pub enable: bool,
    /// Disable given profile(s), does not toggle
    #[arg(short = 'd', long = "disable", conflicts_with = "enable")]
    pub disable: bool,
    /// Activate at specific priority position (1-based). Repositions if already active.
    #[arg(short = 'n', long = "priority", conflicts_with = "inverse")]
    pub priority: Option<usize>,
    /// Reverse the processing order (first listed = highest priority)
    #[arg(short, long, conflicts_with = "priority")]
    pub inverse: bool,
    /// Profile names
    pub names: Vec<String>,
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
    Use(ProfileUse),

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
    List {
        /// Show only active profiles and loaded project aliases
        #[arg(short, long, default_value_t = false)]
        used: bool,
    },
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

    /// Define a subcommand alias (repeatable: --sub short long)
    #[arg(long = "sub", num_args = 2, value_names = ["SHORT", "LONG"], action = clap::ArgAction::Append)]
    pub sub: Vec<String>,
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

/// Variable scope flags (mirrors `Alias` flags).
#[derive(Args, Debug, Clone)]
pub struct VarScopeArgs {
    /// Operate on a specific profile (defaults to active profile)
    #[arg(short, long, conflicts_with_all = ["local", "global"])]
    pub profile: Option<String>,

    /// Operate on the project's .aliases file
    #[arg(short, long, conflicts_with = "global")]
    pub local: bool,

    /// Operate on global vars
    #[arg(short, long)]
    pub global: bool,
}

#[derive(Subcommand)]
pub enum VarAction {
    /// Set a variable's value (upsert)
    Set {
        #[command(flatten)]
        scope: VarScopeArgs,
        /// Variable name
        name: String,
        /// Variable value
        value: String,
    },
    /// Remove a variable
    Unset {
        #[command(flatten)]
        scope: VarScopeArgs,
        /// Variable name
        name: String,
    },
    /// Print a variable's value
    Get {
        #[command(flatten)]
        scope: VarScopeArgs,
        /// Variable name
        name: String,
    },
    /// List variables (all scopes if no flag given)
    List {
        #[command(flatten)]
        scope: VarScopeArgs,
    },
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
