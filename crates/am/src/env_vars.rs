/// Tracks globally-loaded alias names (global + active-profile aliases).
/// Value: comma-separated list, e.g. `"gs,ll"`.
pub const AM_ALIASES: &str = "_AM_ALIASES";

/// Tracks project-level aliases loaded by the cd hook.
/// Value: comma-separated entries of `name|short_hash`, e.g. `"b|a132b21,t|1241ab1"`.
/// The short hash is the first 7 hex chars of the BLAKE3 hash of the alias value,
/// enabling per-alias change detection on reload.
pub const AM_PROJECT_ALIASES: &str = "_AM_PROJECT_ALIASES";

/// Tracks effective subcommand-key state in the shell.
/// Value: comma-separated entries of `name|short_hash`, covering both
/// program-level wrapper hashes (e.g. `jj|abc1234`) and per-key entry
/// hashes (e.g. `jj:ab|def5678`).
pub const AM_SUBCOMMANDS: &str = "_AM_SUBCOMMANDS";

/// Path of the `.aliases` file currently in scope, used to suppress
/// duplicate hook messages when navigating into subdirectories.
pub const AM_PROJECT_PATH: &str = "_AM_PROJECT_PATH";

/// Set during `zsh -i -c alias` alias-detection scans to prevent recursive
/// `am` invocations from triggering another scan.  When present, the `am`
/// binary exits immediately with no output so that `eval "$(...)"` in shell
/// startup scripts is a no-op.
pub const AM_DETECTING_ALIASES: &str = "_AM_DETECTING_ALIASES";

/// Legacy tracking variable replaced by `AM_ALIASES`.  Unset on startup so
/// that old installations do not leave stale state.
pub const AM_PROFILE_ALIASES_LEGACY: &str = "_AM_PROFILE_ALIASES";
