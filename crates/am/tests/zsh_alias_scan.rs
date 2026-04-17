/// Unit tests for the zsh alias-output parser helper.
///
/// Environment-specific / binary-level tests live in `tests/e2e.rs` and are
/// excluded from the default test run (`#[ignore]`).
use std::process::Command;

/// Parse the raw stdout of `alias` into a list of alias names.
fn parse_zsh_alias_keys(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            line.split('=').next().map(|k| k.to_string())
        })
        .collect()
}

#[test]
fn parse_zsh_alias_keys_handles_various_formats() {
    let raw = r#"
gs='git status'
ll='ls -lh'
complex='it'\''s a value'
simple=value
"#;

    let keys = parse_zsh_alias_keys(raw);

    assert!(keys.contains(&"gs".to_string()));
    assert!(keys.contains(&"ll".to_string()));
    assert!(keys.contains(&"complex".to_string()));
    assert!(keys.contains(&"simple".to_string()));
    assert_eq!(keys.len(), 4);
}
