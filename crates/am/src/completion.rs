//! Dynamic shell tab-completion candidates.
//!
//! Each completer is a small, pure function that reads the current
//! amoxide config (via the existing loaders) and returns matching
//! candidates for an arg in `cli.rs`. They are attached to clap args via
//! `add = ArgValueCompleter::new(...)` and invoked at completion time by
//! `clap_complete`'s dynamic engine.
//!
//! `clap_complete::engine::ArgValueCompleter` only sees the partial value
//! of the current arg — not previously-parsed flags. To support
//! context-aware completion like `am remove -p rust <TAB>` (filter
//! aliases to the `rust` profile), the completers reach for
//! `std::env::args_os()` and re-parse the prior tokens themselves via
//! `CompletionCtx::from_env`. Ugly but localised to one helper.

use std::ffi::OsString;

use clap_complete::engine::CompletionCandidate;

use crate::config::Config;
use crate::profile::ProfileConfig;
use crate::project::ProjectAliases;
use crate::session::Session;

/// Scope as inferred from the partial command line at completion time.
///
/// Mirrors the flag combinations recognised by `TargetScopeArgs` and
/// `ScopeArgs` in `cli.rs`, but evaluated *outside* clap (clap's parse
/// state isn't available to a value completer).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CompletionCtx {
    /// `-l` / `--local` was seen.
    pub local: bool,
    /// `-g` / `--global` was seen.
    pub global: bool,
    /// Profile name(s) given after `-p` / `--profile`.
    pub profiles: Vec<String>,
    /// Positional values seen so far on the current subcommand (e.g.,
    /// `am remove jj` → `["jj"]`). Used to look up which alias the user
    /// is extending with `--sub`.
    pub positionals: Vec<String>,
    /// Values already given for `--sub` on the current invocation.
    pub subs: Vec<String>,
}

impl CompletionCtx {
    /// Build a context from `std::env::args_os()`.
    ///
    /// The shim invokes `am` at completion time with the user's partial
    /// command line as argv. We do a one-pass scan: capture `-p VALUE`,
    /// `-l`/`-g`, `--sub VALUE`, and any positionals.
    pub fn from_env() -> Self {
        Self::from_args(std::env::args_os())
    }

    fn from_args<I, S>(args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut ctx = Self::default();
        let args: Vec<String> = args
            .into_iter()
            .filter_map(|a| a.into().into_string().ok())
            .collect();

        let mut iter = args.into_iter().peekable();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-l" | "--local" => ctx.local = true,
                "-g" | "--global" => ctx.global = true,
                "-p" | "--profile" => {
                    if let Some(value) = iter.next() {
                        push_nonempty(&mut ctx.profiles, value);
                    }
                }
                "--sub" => {
                    if let Some(value) = iter.next() {
                        push_nonempty(&mut ctx.subs, value);
                    }
                }
                s if s.starts_with("--profile=") => {
                    push_nonempty(&mut ctx.profiles, s["--profile=".len()..].to_string());
                }
                s if s.starts_with("--sub=") => {
                    push_nonempty(&mut ctx.subs, s["--sub=".len()..].to_string());
                }
                s if !s.starts_with('-') => ctx.positionals.push(s.to_string()),
                _ => {}
            }
        }
        ctx
    }
}

fn matches(name: &str, prefix: &str) -> bool {
    prefix.is_empty() || name.starts_with(prefix)
}

/// Drops empty strings — the trailing empty `--sub`/`-p` value is the
/// placeholder for what's being completed right now, not a prior entry.
fn push_nonempty(target: &mut Vec<String>, value: String) {
    if !value.is_empty() {
        target.push(value);
    }
}

fn prefix_str(current: &std::ffi::OsStr) -> &str {
    current.to_str().unwrap_or("")
}

/// All profile names from `profiles.toml`. Used for `am use`,
/// `am profile use/remove`, and the `-p` flag everywhere.
pub fn profile_names(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    let prefix = prefix_str(current);
    let Ok(profiles) = ProfileConfig::load() else {
        return Vec::new();
    };
    profiles
        .iter()
        .filter(|p| matches(&p.name, prefix))
        .map(|p| CompletionCandidate::new(&p.name))
        .collect()
}

/// Alias names from the scope inferred by `CompletionCtx`. Used for
/// `am remove`.
pub fn alias_names(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    alias_names_with_ctx(prefix_str(current), &CompletionCtx::from_env())
}

pub(crate) fn alias_names_with_ctx(prefix: &str, ctx: &CompletionCtx) -> Vec<CompletionCandidate> {
    let mut names: Vec<String> = Vec::new();

    if ctx.global {
        names.extend(load_global_alias_names());
        names.extend(load_subcommand_keys_global());
    } else if ctx.local {
        names.extend(load_local_alias_names());
        names.extend(load_subcommand_keys_local());
    } else if !ctx.profiles.is_empty() {
        names.extend(load_profile_alias_names(&ctx.profiles));
        names.extend(load_subcommand_keys_profiles(&ctx.profiles));
    } else {
        // No scope flag: union of global + active profiles + local.
        names.extend(load_global_alias_names());
        names.extend(load_active_profile_alias_names());
        names.extend(load_local_alias_names());
        names.extend(load_subcommand_keys_global());
        names.extend(load_subcommand_keys_active_profiles());
        names.extend(load_subcommand_keys_local());
    }

    names.sort();
    names.dedup();
    names
        .into_iter()
        .filter(|n| matches(n, prefix))
        .map(CompletionCandidate::new)
        .collect()
}

/// Variable names from the scope inferred by `CompletionCtx`. Used for
/// `am var get/unset`.
pub fn var_names(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    var_names_with_ctx(prefix_str(current), &CompletionCtx::from_env())
}

pub(crate) fn var_names_with_ctx(prefix: &str, ctx: &CompletionCtx) -> Vec<CompletionCandidate> {
    let mut names: Vec<String> = Vec::new();

    if ctx.global {
        names.extend(load_global_var_names());
    } else if ctx.local {
        names.extend(load_local_var_names());
    } else if !ctx.profiles.is_empty() {
        names.extend(load_profile_var_names(&ctx.profiles));
    } else {
        names.extend(load_global_var_names());
        names.extend(load_active_profile_var_names());
        names.extend(load_local_var_names());
    }

    names.sort();
    names.dedup();
    names
        .into_iter()
        .filter(|n| matches(n, prefix))
        .map(CompletionCandidate::new)
        .collect()
}

/// Next-segment candidates for a subcommand-alias chain.
///
/// Example: `am remove jj --sub b --sub <TAB>` — the alias name (`jj`)
/// is the first positional under `am remove`, the prior `--sub` values
/// (`["b"]`) narrow the chain, and the candidates are the short tokens
/// that already exist as children of `jj:b:` in the configured
/// subcommand set.
pub fn sub_segments(current: &std::ffi::OsStr) -> Vec<CompletionCandidate> {
    sub_segments_with_ctx(prefix_str(current), &CompletionCtx::from_env())
}

pub(crate) fn sub_segments_with_ctx(prefix: &str, ctx: &CompletionCtx) -> Vec<CompletionCandidate> {
    // The alias's program name is the last positional after `am
    // <verb>`. We can't tell `am remove jj` apart from `am remove` for
    // a partial `jj<TAB>`, but `--sub` only makes sense once a program
    // is fixed, so if no positional is present we have nothing useful
    // to offer.
    let Some(program) = ctx.positionals.last() else {
        return Vec::new();
    };
    // Strip a trailing colon if the user wrote `jj:` as the program —
    // colon notation is for alias names, the program is still `jj`.
    let program = program.split(':').next().unwrap_or(program);

    let mut all_keys: Vec<String> = Vec::new();
    if ctx.global {
        all_keys.extend(load_subcommand_keys_global());
    } else if ctx.local {
        all_keys.extend(load_subcommand_keys_local());
    } else if !ctx.profiles.is_empty() {
        all_keys.extend(load_subcommand_keys_profiles(&ctx.profiles));
    } else {
        all_keys.extend(load_subcommand_keys_global());
        all_keys.extend(load_subcommand_keys_active_profiles());
        all_keys.extend(load_subcommand_keys_local());
    }
    all_keys.sort();
    all_keys.dedup();

    let mut want_prefix = String::from(program);
    for sub in &ctx.subs {
        want_prefix.push(':');
        want_prefix.push_str(sub);
    }
    want_prefix.push(':');

    let mut candidates: Vec<String> = all_keys
        .into_iter()
        .filter_map(|k| k.strip_prefix(&want_prefix).map(str::to_owned))
        .filter_map(|tail| tail.split(':').next().map(str::to_owned))
        .filter(|seg| !seg.is_empty())
        .collect();
    candidates.sort();
    candidates.dedup();
    candidates
        .into_iter()
        .filter(|s| matches(s, prefix))
        .map(CompletionCandidate::new)
        .collect()
}

// ---------- internal loaders ----------

fn load_global_alias_names() -> Vec<String> {
    Config::load()
        .map(|c| {
            c.aliases
                .iter()
                .map(|(n, _)| n.as_ref().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn load_local_alias_names() -> Vec<String> {
    let Ok(cwd) = std::env::current_dir() else {
        return Vec::new();
    };
    ProjectAliases::find(&cwd)
        .ok()
        .flatten()
        .map(|p| {
            p.aliases
                .iter()
                .map(|(n, _)| n.as_ref().to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn load_profile_alias_names(profile_names: &[String]) -> Vec<String> {
    let Ok(profiles) = ProfileConfig::load() else {
        return Vec::new();
    };
    profile_names
        .iter()
        .filter_map(|name| profiles.get_profile_by_name(name))
        .flat_map(|p| p.aliases.iter().map(|(n, _)| n.as_ref().to_string()))
        .collect()
}

fn load_active_profile_alias_names() -> Vec<String> {
    let Ok(session) = Session::load() else {
        return Vec::new();
    };
    load_profile_alias_names(&session.active_profiles)
}

fn load_global_var_names() -> Vec<String> {
    Config::load()
        .map(|c| c.vars.iter().map(|(n, _)| n.to_string()).collect())
        .unwrap_or_default()
}

fn load_local_var_names() -> Vec<String> {
    let Ok(cwd) = std::env::current_dir() else {
        return Vec::new();
    };
    ProjectAliases::find(&cwd)
        .ok()
        .flatten()
        .map(|p| p.vars.iter().map(|(n, _)| n.to_string()).collect())
        .unwrap_or_default()
}

fn load_profile_var_names(profile_names: &[String]) -> Vec<String> {
    let Ok(profiles) = ProfileConfig::load() else {
        return Vec::new();
    };
    profile_names
        .iter()
        .filter_map(|name| profiles.get_profile_by_name(name))
        .flat_map(|p| p.vars.iter().map(|(n, _)| n.to_string()))
        .collect()
}

fn load_active_profile_var_names() -> Vec<String> {
    let Ok(session) = Session::load() else {
        return Vec::new();
    };
    load_profile_var_names(&session.active_profiles)
}

fn load_subcommand_keys_global() -> Vec<String> {
    Config::load()
        .map(|c| c.subcommands.as_ref().keys().cloned().collect())
        .unwrap_or_default()
}

fn load_subcommand_keys_local() -> Vec<String> {
    let Ok(cwd) = std::env::current_dir() else {
        return Vec::new();
    };
    ProjectAliases::find(&cwd)
        .ok()
        .flatten()
        .map(|p| p.subcommands.as_ref().keys().cloned().collect())
        .unwrap_or_default()
}

fn load_subcommand_keys_active_profiles() -> Vec<String> {
    let Ok(session) = Session::load() else {
        return Vec::new();
    };
    load_subcommand_keys_profiles(&session.active_profiles)
}

fn load_subcommand_keys_profiles(profile_names: &[String]) -> Vec<String> {
    let Ok(profiles) = ProfileConfig::load() else {
        return Vec::new();
    };
    profile_names
        .iter()
        .filter_map(|name| profiles.get_profile_by_name(name))
        .flat_map(|p| p.subcommands.as_ref().keys().cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(args: &[&str]) -> CompletionCtx {
        CompletionCtx::from_args(args.iter().map(|s| OsString::from(*s)))
    }

    #[test]
    fn ctx_picks_up_local_global() {
        assert!(ctx(&["am", "remove", "-l"]).local);
        assert!(ctx(&["am", "remove", "--local"]).local);
        assert!(ctx(&["am", "remove", "-g"]).global);
        assert!(ctx(&["am", "remove", "--global"]).global);
    }

    #[test]
    fn ctx_picks_up_profile_spaced_and_equals() {
        assert_eq!(ctx(&["am", "remove", "-p", "rust"]).profiles, vec!["rust"]);
        assert_eq!(
            ctx(&["am", "remove", "--profile", "rust"]).profiles,
            vec!["rust"]
        );
        assert_eq!(
            ctx(&["am", "remove", "--profile=rust"]).profiles,
            vec!["rust"]
        );
    }

    #[test]
    fn ctx_picks_up_multiple_subs() {
        let c = ctx(&["am", "remove", "jj", "--sub", "b", "--sub", "l"]);
        assert_eq!(c.subs, vec!["b", "l"]);
    }

    #[test]
    fn ctx_collects_positionals_excluding_flag_values() {
        let c = ctx(&["am", "remove", "-p", "rust", "jj"]);
        // "rust" is consumed as -p's value; only "jj" remains positional.
        assert_eq!(c.positionals, vec!["am", "remove", "jj"]);
    }

    #[test]
    fn ctx_handles_combined_scope_and_subs() {
        let c = ctx(&[
            "am", "remove", "-p", "rust", "jj", "--sub", "b", "--sub", "l",
        ]);
        assert_eq!(c.profiles, vec!["rust"]);
        assert_eq!(c.subs, vec!["b", "l"]);
        assert_eq!(c.positionals, vec!["am", "remove", "jj"]);
    }
}
