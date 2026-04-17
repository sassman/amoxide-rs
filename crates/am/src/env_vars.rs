/// Tracks globally-loaded alias names (global + active-profile aliases).
/// Value: comma-separated list, e.g. `"gs,ll"`.
pub const AM_ALIASES: &str = "_AM_ALIASES";

/// Tracks project-level alias names loaded by the cd hook.
/// Value: comma-separated list, e.g. `"b,t"`.
pub const AM_PROJECT_ALIASES: &str = "_AM_PROJECT_ALIASES";

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
