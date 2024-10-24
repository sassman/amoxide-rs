use clap::{Parser, Subcommand};

/// The Shell-Manager
///
/// The shell-manager (`sm`) is only for the most laziest among you.
/// It helps you to manage your aliases, paths and secret env variables on the shell,
/// either globally or project (like directory) specific.
#[derive(Parser)]
#[command(name = "sm")]
#[command(about = "The Shell-Manager", long_about = None, version, author)]
#[command(propagate_version = true)]
pub struct Cli {
    /// The current shell sm runing in
    #[arg(long, env = "SHELL_MANAGER_SHELL")]
    pub current_shell: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new alias, path, or secret
    #[command(subcommand, alias = "a")]
    Add(AddCommands),

    /// Load environment variables into the current shell
    #[command(alias = "e")]
    Env {
        /// The shell to initialize
        #[arg(env = "SHELL_MANAGER_SHELL")]
        shell: String,
    },

    /// Initialize the shell-manager for your shell, usually put `eval "$(sm init)"` in your shell rc file
    #[command(alias = "i")]
    Init {
        /// The shell to initialize
        #[arg()]
        shell: String,
    },
}

#[derive(Subcommand)]
pub enum AddCommands {
    /// Add a new alias
    #[command(alias = "a")]
    Alias {
        /// The name of the alias
        name: String,

        /// The command to be aliased, if not provided, the last command fromm the history will be used
        value: Option<String>,

        /// Directory-specific flag
        #[arg(short, long)]
        directory: bool,

        /// Long alias flag
        #[arg(short, long)]
        long: bool,
    },
    /// Add a new path
    Path {
        /// The name of the path
        name: String,

        /// The path to be added
        value: String,

        /// Directory-specific flag
        #[arg(short, long)]
        directory: bool,
    },
    /// Add a new secret
    Secret {
        /// The name of the secret
        name: String,

        /// The secret value
        value: String,

        /// Directory-specific flag
        #[arg(short, long)]
        directory: bool,
    },
}
