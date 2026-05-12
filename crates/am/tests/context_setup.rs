//! Integration tests for `am context --setup claude`.
//!
//! Each test operates on a temp file path passed to `run_claude_setup`
//! directly — no real `~/.claude/` is ever touched.

use amoxide::setup::{
    claude_settings_already_wired, run_claude_setup, write_settings_atomic, SetupOutcome,
};
use serde_json::{json, Value};

fn read_json(path: &std::path::Path) -> Value {
    let contents = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&contents).unwrap()
}

#[test]
fn setup_creates_settings_file_when_absent() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(".claude/settings.json");
    let outcome = run_claude_setup(&path).unwrap();
    assert!(matches!(outcome, SetupOutcome::Created(ref p) if p == &path));
    assert!(path.exists());
    assert!(claude_settings_already_wired(&read_json(&path)));
}

#[test]
fn setup_adds_hook_to_existing_settings_without_hooks_key() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    write_settings_atomic(&path, &json!({ "model": "claude-opus-4" })).unwrap();

    let outcome = run_claude_setup(&path).unwrap();
    assert!(matches!(outcome, SetupOutcome::Updated(_)));

    let after = read_json(&path);
    assert_eq!(after["model"], "claude-opus-4", "other keys preserved");
    assert!(claude_settings_already_wired(&after));
}

#[test]
fn setup_adds_session_start_alongside_other_hook_events() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    write_settings_atomic(
        &path,
        &json!({
            "hooks": {
                "PreToolUse": [{
                    "matcher": "*",
                    "hooks": [{ "type": "command", "command": "echo pre" }]
                }]
            }
        }),
    )
    .unwrap();

    let outcome = run_claude_setup(&path).unwrap();
    assert!(matches!(outcome, SetupOutcome::Updated(_)));

    let after = read_json(&path);
    assert_eq!(
        after["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "echo pre",
        "other hook events untouched"
    );
    assert!(claude_settings_already_wired(&after));
}

#[test]
fn setup_appends_to_existing_session_start() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    write_settings_atomic(
        &path,
        &json!({
            "hooks": {
                "SessionStart": [{
                    "matcher": "startup",
                    "hooks": [{ "type": "command", "command": "echo other" }]
                }]
            }
        }),
    )
    .unwrap();

    let outcome = run_claude_setup(&path).unwrap();
    assert!(matches!(outcome, SetupOutcome::Updated(_)));

    let after = read_json(&path);
    let session_start = after["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(session_start.len(), 2, "appended, not replaced");
    assert_eq!(
        session_start[0]["hooks"][0]["command"], "echo other",
        "existing entry preserved at same position"
    );
}

#[test]
fn setup_is_idempotent_when_already_wired() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");

    let first = run_claude_setup(&path).unwrap();
    assert!(matches!(first, SetupOutcome::Created(_)));
    let after_first = read_json(&path);

    let second = run_claude_setup(&path).unwrap();
    assert!(matches!(second, SetupOutcome::AlreadyConfigured(_)));
    let after_second = read_json(&path);

    assert_eq!(after_first, after_second, "second run is a no-op");
}

#[test]
fn setup_aborts_on_parse_failure_without_overwriting() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    std::fs::write(&path, "{ not valid json").unwrap();

    let result = run_claude_setup(&path);
    assert!(
        result.is_err(),
        "should refuse to overwrite unparseable file"
    );
    let err = result.unwrap_err().to_string();
    assert!(err.contains("could not parse"), "got: {err}");

    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(after, "{ not valid json", "must not overwrite");
}

#[test]
fn setup_outcome_render_includes_path() {
    let path = std::path::PathBuf::from("/tmp/x/settings.json");
    let created = SetupOutcome::Created(path.clone()).render();
    assert!(created.contains("/tmp/x/settings.json"));
    assert!(created.contains("created"));

    let updated = SetupOutcome::Updated(path.clone()).render();
    assert!(updated.contains("/tmp/x/settings.json"));
    assert!(updated.contains("added"));

    let already = SetupOutcome::AlreadyConfigured(path).render();
    assert!(already.contains("/tmp/x/settings.json"));
    assert!(already.contains("already wired"));
}
