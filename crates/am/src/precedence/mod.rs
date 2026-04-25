//! Precedence resolution: merge global/profile/project alias layers against
//! the current shell's loaded state to produce a diff that tells the shell
//! exactly what to load, reload, or unload.
//!
//! Split into three submodules:
//!   * `env_state` — `_AM_ALIASES` / `_AM_SUBCOMMANDS` wire format.
//!   * `diff` — the `PrecedenceDiff` output and how it renders to shell code.
//!   * `engine` — the `Precedence` builder and `resolve()` logic.

mod diff;
mod engine;
mod env_state;

pub(crate) use diff::format_change_summary;
pub use diff::{
    Diagnostic, EffectiveEntry, EntryKind, InvalidEntry, InvalidReason, OriginScope,
    PrecedenceDiff, ResolveOutcome,
};
pub use engine::Precedence;
pub use env_state::{AliasWithHash, AliasWithHashList};
