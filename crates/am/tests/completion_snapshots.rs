//! Tab-completion integration tests.
//!
//! Two layers of coverage:
//!
//! 1. **Registration shim snapshots** — invoke `COMPLETE=<shell> am` and
//!    snapshot the stdout. clap_complete's `unstable-dynamic` API has
//!    churned across minor versions; if the shim changes shape (env var
//!    names, escaping, etc.) the snapshot diff makes that loud at PR time.
//!
//! 2. **Candidate-list assertions** — set up a fixture config under
//!    `AMOXIDE_CONFIG_DIR` and invoke the binary the way the bash shim
//!    would (with the `_CLAP_*` env vars + argv layout), then assert the
//!    candidates returned for profiles/aliases/subcommands/variables.

use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

use regex::Regex;
use tempfile::TempDir;

const AM: &str = env!("CARGO_BIN_EXE_am");

// ---------------- Registration shim snapshots ----------------

/// Matches any filesystem path the shim might embed for the binary:
/// `/Users/.../am`, `D:\a\…\am.exe`, `\\?\D:\…\am.exe`. Both separators
/// + optional `.exe` so snapshots stay portable across Unix and Windows.
static AM_PATH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?:\\\\\?\\)?(?:[A-Za-z]:)?[\\/](?:[^\s"'`]+[\\/])+am(?:\.exe)?"#).unwrap()
});

/// clap_complete shell-quotes Windows paths (they can contain `:`, spaces,
/// backslashes); Unix paths come through bare. Strip adjacent quote chars
/// so the snapshot reads the same on both.
static AM_BIN_QUOTES_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"['"]*<AM_BIN>['"]*"#).unwrap());

fn registration_shim(shell: &str) -> String {
    let output = Command::new(AM).env("COMPLETE", shell).output().unwrap();
    assert!(
        output.status.success(),
        "COMPLETE={shell} am failed: stderr={}",
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let masked_path = AM_PATH_RE.replace_all(&stdout, "<AM_BIN>");
    AM_BIN_QUOTES_RE
        .replace_all(&masked_path, "<AM_BIN>")
        .into_owned()
}

#[test]
fn shim_bash_registration() {
    insta::assert_snapshot!(registration_shim("bash"));
}

#[test]
fn shim_zsh_registration() {
    insta::assert_snapshot!(registration_shim("zsh"));
}

#[test]
fn shim_fish_registration() {
    insta::assert_snapshot!(registration_shim("fish"));
}

#[test]
fn shim_powershell_registration() {
    insta::assert_snapshot!(registration_shim("powershell"));
}

// ---------------- Fixture-driven candidate tests ----------------

const FIXTURE_PROFILES_TOML: &str = r#"
[[profiles]]
name = "rust"
[profiles.aliases]
b = "cargo build"
t = "cargo test"
[profiles.vars]
opts = "release"

[[profiles]]
name = "git"
[profiles.aliases]
gs = "git status"
[profiles.subcommands]
"jj:b:l" = ["branch", "list"]
"jj:b:n" = ["branch", "new"]
"jj:ab" = ["abandon"]
"#;

const FIXTURE_CONFIG_TOML: &str = r#"
[aliases]
ll = "ls -lha"
la = "ls -A"

[vars]
greeting = "hello"

[subcommands]
"jj:st" = ["status"]
"#;

const FIXTURE_SESSION_TOML: &str = r#"
active_profiles = ["rust"]
"#;

fn fixture() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("profiles.toml"), FIXTURE_PROFILES_TOML).unwrap();
    std::fs::write(dir.path().join("config.toml"), FIXTURE_CONFIG_TOML).unwrap();
    std::fs::write(dir.path().join("session.toml"), FIXTURE_SESSION_TOML).unwrap();
    dir
}

/// Invoke `am` the way the bash shim would: with `COMPLETE=bash`, the
/// `_CLAP_*` env vars, and `["--", "am", <partial words...>]` as argv.
/// `cursor_index` is the 0-based index within `[am, <partial words...>]`
/// of the word the user is completing.
fn complete(args: &[&str], cursor_index: usize, config_dir: &Path) -> Vec<String> {
    let mut argv: Vec<String> = vec!["--".to_string()];
    argv.extend(args.iter().map(|s| (*s).to_string()));

    let output = Command::new(AM)
        .args(&argv)
        .env("COMPLETE", "bash")
        .env("_CLAP_IFS", "\x0b")
        .env("_CLAP_COMPLETE_INDEX", cursor_index.to_string())
        .env("_CLAP_COMPLETE_COMP_TYPE", "9")
        .env("_CLAP_COMPLETE_SPACE", "true")
        .env("AMOXIDE_CONFIG_DIR", config_dir)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "completion call failed: stderr={}",
        String::from_utf8_lossy(&output.stderr),
    );
    String::from_utf8(output.stdout)
        .unwrap()
        .split('\x0b')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

#[test]
fn completes_profile_names_for_am_use() {
    let dir = fixture();
    let got = complete(&["am", "use", ""], 2, dir.path());
    assert!(got.contains(&"rust".to_string()), "got: {got:?}");
    assert!(got.contains(&"git".to_string()), "got: {got:?}");
}

#[test]
fn completes_profile_names_with_prefix() {
    let dir = fixture();
    let got = complete(&["am", "use", "ru"], 2, dir.path());
    assert!(got.contains(&"rust".to_string()), "got: {got:?}");
    assert!(!got.contains(&"git".to_string()), "got: {got:?}");
}

#[test]
fn completes_profile_names_after_dash_p() {
    let dir = fixture();
    let got = complete(&["am", "remove", "-p", ""], 3, dir.path());
    assert!(got.contains(&"rust".to_string()), "got: {got:?}");
    assert!(got.contains(&"git".to_string()), "got: {got:?}");
}

#[test]
fn completes_alias_names_global_only_with_dash_g() {
    let dir = fixture();
    let got = complete(&["am", "remove", "-g", ""], 3, dir.path());
    assert!(got.contains(&"ll".to_string()), "got: {got:?}");
    assert!(got.contains(&"la".to_string()), "got: {got:?}");
    assert!(!got.contains(&"b".to_string()), "got: {got:?}");
    assert!(!got.contains(&"gs".to_string()), "got: {got:?}");
}

#[test]
fn completes_alias_names_scoped_to_profile() {
    let dir = fixture();
    let got = complete(&["am", "remove", "-p", "rust", ""], 4, dir.path());
    assert!(got.contains(&"b".to_string()), "got: {got:?}");
    assert!(got.contains(&"t".to_string()), "got: {got:?}");
    assert!(!got.contains(&"gs".to_string()), "got: {got:?}");
    assert!(!got.contains(&"ll".to_string()), "got: {got:?}");
}

#[test]
fn completes_alias_names_union_when_no_scope() {
    let dir = fixture();
    // No scope flag → union of global + active profiles + local.
    // session.toml activates "rust" → expect global (ll, la) + rust (b, t).
    let got = complete(&["am", "remove", ""], 2, dir.path());
    assert!(got.contains(&"ll".to_string()), "got: {got:?}");
    assert!(got.contains(&"b".to_string()), "got: {got:?}");
}

#[test]
fn completes_var_names_scoped_to_profile() {
    let dir = fixture();
    let got = complete(&["am", "var", "get", "-p", "rust", ""], 5, dir.path());
    assert!(got.contains(&"opts".to_string()), "got: {got:?}");
    assert!(!got.contains(&"greeting".to_string()), "got: {got:?}");
}

#[test]
fn completes_sub_segments_first_level() {
    let dir = fixture();
    // `am remove jj --sub <TAB>` — first segment under "jj".
    // Active profile is "rust" which has no jj:* keys, so this taps
    // global (jj:st) only — completion ignores inactive profile "git".
    let got = complete(&["am", "remove", "jj", "--sub", ""], 4, dir.path());
    assert!(got.contains(&"st".to_string()), "got: {got:?}");
    assert!(!got.contains(&"b".to_string()), "got: {got:?}");
}

#[test]
fn completes_sub_segments_second_level_with_profile_scope() {
    let dir = fixture();
    // `am remove -p git jj --sub b --sub <TAB>` — second segment under
    // "jj:b" from the git profile. Expects "l" and "n".
    let got = complete(
        &["am", "remove", "-p", "git", "jj", "--sub", "b", "--sub", ""],
        8,
        dir.path(),
    );
    assert!(got.contains(&"l".to_string()), "got: {got:?}");
    assert!(got.contains(&"n".to_string()), "got: {got:?}");
}
