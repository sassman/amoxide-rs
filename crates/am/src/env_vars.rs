/// Tracks effective alias state in the shell (aliases + subcommand wrappers).
/// Value: comma-separated entries of `name|short_hash`, enabling per-entry
/// change detection on sync.
pub const AM_ALIASES: &str = "_AM_ALIASES";

/// Tracks effective subcommand-key state in the shell.
/// Value: comma-separated entries of `name|short_hash`.
pub const AM_SUBCOMMANDS: &str = "_AM_SUBCOMMANDS";

/// Path of the `.aliases` file currently in scope, used to suppress
/// duplicate warnings when navigating into subdirectories.
pub const AM_PROJECT_PATH: &str = "_AM_PROJECT_PATH";

/// Set during shell-scanning to prevent recursive am invocation.
pub const AM_DETECTING_ALIASES: &str = "_AM_DETECTING_ALIASES";
