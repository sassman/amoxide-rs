//! Background update check against crates.io.
//!
//! The user runs `am ls` (or `am la`, or `am profile list`) and pays no
//! network latency: we read a local TOML cache and, if it's stale, fire off
//! a detached child process that does the actual check and writes a fresh
//! cache. The result is shown on the *next* invocation.
//!
//! Layout:
//! - `${cache_dir}/amoxide/update-check.toml`  — `{ checked_at_secs, latest_version }`
//! - `${cache_dir}/amoxide/update-check.lock`  — touched while a check is in flight
//!
//! Decision logic in `decide_effect` is pure and tested; the spawn and HTTP
//! call live in `perform_check`, invoked by the hidden `__update-check`
//! subcommand from `bin/am.rs`.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const CACHE_FILE: &str = "update-check.toml";
const LOCK_FILE: &str = "update-check.lock";
const STALE_AFTER_SECS: u64 = 60 * 60 * 24;
const LOCK_TTL_SECS: u64 = 60 * 5;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const RELEASES_URL: &str = "https://github.com/sassman/amoxide-rs/releases";

/// On-disk shape of the cached check result.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateCache {
    pub checked_at_secs: u64,
    pub latest_version: String,
}

impl UpdateCache {
    pub fn load_from(dir: &Path) -> Option<Self> {
        let body = std::fs::read_to_string(dir.join(CACHE_FILE)).ok()?;
        toml::from_str(&body).ok()
    }

    pub fn save_to(&self, dir: &Path) -> anyhow::Result<()> {
        std::fs::create_dir_all(dir)?;
        let final_path = dir.join(CACHE_FILE);
        let tmp_path = dir.join(format!("{CACHE_FILE}.tmp"));
        std::fs::write(&tmp_path, toml::to_string(self)?)?;
        std::fs::rename(tmp_path, final_path)?;
        Ok(())
    }

    pub fn is_stale(&self, now_secs: u64) -> bool {
        now_secs.saturating_sub(self.checked_at_secs) > STALE_AFTER_SECS
    }
}

/// What `update()` should emit for the listing path, given the cache state.
///
/// Pure: takes the cache, the running version, and "now" — returns the
/// decision without touching disk or clock.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Show this nudge line to the user (cache is fresh and there's a newer release).
    Print(String),
    /// Cache is missing or stale — kick off a background check.
    Spawn,
    /// Cache is fresh and we're up to date — say nothing.
    Quiet,
}

pub fn decide_effect(
    cache: Option<&UpdateCache>,
    current_version: &str,
    now_secs: u64,
) -> Decision {
    let Some(cache) = cache else {
        return Decision::Spawn;
    };
    if cache.is_stale(now_secs) {
        return Decision::Spawn;
    }
    match nudge_message(&cache.latest_version, current_version) {
        Some(msg) => Decision::Print(msg),
        None => Decision::Quiet,
    }
}

fn nudge_message(latest: &str, current: &str) -> Option<String> {
    is_newer(latest, current)
        .then(|| format!("am: 💡 a new version is available: v{latest} -> visit {RELEASES_URL}"))
}

/// Compare two dotted-numeric version strings. Strips a leading `v`, parses
/// the leading numeric components, and yields `latest > current`.
///
/// Non-numeric suffixes (`-rc.1`, `+build.42`) are ignored — we only care
/// about the stable numeric prefix, which is what crates.io's
/// `max_stable_version` returns.
fn is_newer(latest: &str, current: &str) -> bool {
    parse_numeric(latest) > parse_numeric(current)
}

fn parse_numeric(s: &str) -> Vec<u64> {
    // Split on dots; the first non-numeric segment (e.g. "0-rc", "1+build")
    // terminates the version, so pre-release and build metadata are dropped.
    s.trim_start_matches('v')
        .split('.')
        .map_while(|p| p.parse::<u64>().ok())
        .collect()
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn cache_dir() -> Option<PathBuf> {
    dirs_lite::cache_dir().map(|p| p.join("amoxide"))
}

/// Lock-aware check: returns true if the caller should spawn a background
/// check now. A lock file younger than 5 minutes means another check is
/// (probably) in flight — skip to avoid hammering crates.io on a burst of
/// `am ls` calls.
pub fn should_spawn(dir: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(dir.join(LOCK_FILE)) else {
        return true;
    };
    let age = meta.modified().ok().and_then(|t| t.elapsed().ok());
    age.is_none_or(|d| d.as_secs() > LOCK_TTL_SECS)
}

/// Hidden subcommand body. Hits crates.io, parses the response, writes the
/// cache atomically. Best-effort throughout — failures stay silent because
/// nobody is reading stdout/stderr (we were spawned detached).
pub fn perform_check(name: &str) -> anyhow::Result<()> {
    let Some(dir) = cache_dir() else {
        return Ok(());
    };
    std::fs::create_dir_all(&dir)?;
    let lock_path = dir.join(LOCK_FILE);
    let _ = std::fs::write(&lock_path, "");

    let result = fetch_latest_version(name);

    let _ = std::fs::remove_file(&lock_path);

    let latest_version = result?;
    UpdateCache {
        checked_at_secs: now_secs(),
        latest_version,
    }
    .save_to(&dir)
}

fn fetch_latest_version(name: &str) -> anyhow::Result<String> {
    let url = format!("https://crates.io/api/v1/crates/{name}");
    let ua = format!("{name}/{} (+{RELEASES_URL})", env!("CARGO_PKG_VERSION"));
    let agent = ureq::Agent::config_builder()
        .timeout_connect(Some(REQUEST_TIMEOUT))
        .timeout_global(Some(REQUEST_TIMEOUT))
        .https_only(true)
        .user_agent(ua)
        .build()
        .new_agent();

    let mut response = agent
        .get(&url)
        .call()
        .map_err(|e| anyhow::anyhow!("crates.io request failed: {e}"))?;
    let body = response
        .body_mut()
        .with_config()
        .limit(64 * 1024)
        .read_to_string()
        .map_err(|e| anyhow::anyhow!("crates.io response read failed: {e}"))?;

    parse_max_stable_version(&body).ok_or_else(|| {
        anyhow::anyhow!("could not parse max_stable_version from crates.io response")
    })
}

fn parse_max_stable_version(body: &str) -> Option<String> {
    let re = regex::Regex::new(r#""max_stable_version"\s*:\s*"([^"]+)""#).ok()?;
    re.captures(body)?.get(1).map(|m| m.as_str().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_newer_compares_numeric_prefix() {
        assert!(is_newer("0.9.0", "0.8.1"));
        assert!(is_newer("0.10.0", "0.9.9"));
        assert!(is_newer("1.0.0", "0.99.99"));
        assert!(!is_newer("0.8.1", "0.8.1"));
        assert!(!is_newer("0.8.0", "0.8.1"));
    }

    #[test]
    fn is_newer_strips_v_prefix_and_suffixes() {
        assert!(is_newer("v0.9.0", "0.8.1"));
        assert!(is_newer("0.9.0-rc.1", "0.8.1"));
        assert!(!is_newer("0.8.1+build.42", "0.8.1"));
    }

    #[test]
    fn cache_is_stale_after_24h() {
        let cache = UpdateCache {
            checked_at_secs: 1_000_000,
            latest_version: "0.9.0".into(),
        };
        assert!(!cache.is_stale(1_000_000 + 60 * 60 * 23));
        assert!(cache.is_stale(1_000_000 + 60 * 60 * 25));
    }

    #[test]
    fn decide_no_cache_spawns() {
        assert_eq!(decide_effect(None, "0.8.1", 0), Decision::Spawn);
    }

    #[test]
    fn decide_stale_cache_spawns() {
        let cache = UpdateCache {
            checked_at_secs: 0,
            latest_version: "0.9.0".into(),
        };
        assert_eq!(
            decide_effect(Some(&cache), "0.8.1", STALE_AFTER_SECS + 1),
            Decision::Spawn
        );
    }

    #[test]
    fn decide_fresh_cache_with_newer_prints_nudge() {
        let cache = UpdateCache {
            checked_at_secs: 1_000_000,
            latest_version: "0.9.0".into(),
        };
        let Decision::Print(msg) = decide_effect(Some(&cache), "0.8.1", 1_000_100) else {
            panic!("expected Print decision");
        };
        assert!(msg.contains("v0.9.0"), "msg: {msg}");
        assert!(msg.contains(RELEASES_URL), "msg: {msg}");
    }

    #[test]
    fn decide_fresh_cache_same_version_is_quiet() {
        let cache = UpdateCache {
            checked_at_secs: 1_000_000,
            latest_version: "0.8.1".into(),
        };
        assert_eq!(
            decide_effect(Some(&cache), "0.8.1", 1_000_100),
            Decision::Quiet
        );
    }

    #[test]
    fn cache_round_trip_through_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        let cache = UpdateCache {
            checked_at_secs: 1_715_587_200,
            latest_version: "0.9.0".into(),
        };
        cache.save_to(dir.path()).unwrap();
        let loaded = UpdateCache::load_from(dir.path()).unwrap();
        assert_eq!(loaded, cache);
    }

    #[test]
    fn cache_load_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(UpdateCache::load_from(dir.path()), None);
    }

    #[test]
    fn cache_load_corrupt_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CACHE_FILE), "this is not toml ===").unwrap();
        assert_eq!(UpdateCache::load_from(dir.path()), None);
    }

    #[test]
    fn should_spawn_with_no_lock() {
        let dir = tempfile::tempdir().unwrap();
        assert!(should_spawn(dir.path()));
    }

    #[test]
    fn should_spawn_skips_when_lock_is_fresh() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(LOCK_FILE), "").unwrap();
        assert!(!should_spawn(dir.path()));
    }

    #[test]
    fn parse_max_stable_version_extracts_field() {
        let body = r#"{"crate":{"name":"amoxide","max_stable_version":"0.8.1","max_version":"0.9.0-rc.1"}}"#;
        assert_eq!(parse_max_stable_version(body), Some("0.8.1".to_string()));
    }

    #[test]
    fn parse_max_stable_version_returns_none_on_missing() {
        assert_eq!(parse_max_stable_version("{}"), None);
    }
}
