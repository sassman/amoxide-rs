/// Proof-of-concept: `zsh -i -c 'alias'` can enumerate pre-existing aliases.
///
/// This test is intentionally environment-specific — it runs against the current
/// user's real zsh startup files.  It proves that the mechanism works before any
/// production implementation is added to the crate.
///
/// Expected: the test machine has `la='ls -lAh'` defined in ~/.zshrc (or a
/// sourced plugin).  If that alias is ever removed from the system config the
/// assertion at the bottom should be updated accordingly.
use std::process::Command;

/// Proof that the `_AM_DETECTING_ALIASES` guard works: when `am hook` is invoked
/// while the env var is set (as it happens during alias scanning), the binary must
/// exit cleanly with no stdout so that `eval "$(...)"` in shell startup scripts
/// is a no-op — preventing infinite recursion.
///
/// Without the guard `am hook zsh` in a directory containing an `.aliases` file
/// outputs shell code (at minimum a trust warning), which would be eval'd by the
/// child zsh and could trigger another scan cycle.
#[test]
#[ignore]
fn am_hook_is_silent_when_am_detecting_aliases_guard_is_active() {
    // Create a directory with a (deliberately untrusted) .aliases file.
    // Without the guard this would cause `am hook zsh` to emit shell code.
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    std::fs::write(
        dir.path().join(".aliases"),
        "[aliases]\nb = \"make build\"\n",
    )
    .unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_am"))
        .args(["hook", "zsh"])
        .env("_AM_DETECTING_ALIASES", "1")
        .env_remove("_AM_PROJECT_ALIASES")
        .env_remove("_AM_PROJECT_PATH")
        .current_dir(dir.path())
        .output()
        .expect("failed to spawn am binary");

    assert!(
        output.status.success(),
        "am should exit 0 when the guard is active, got: {}",
        output.status
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.is_empty(),
        "am hook must produce no stdout when _AM_DETECTING_ALIASES is set\n\
         (any output would be eval'd by the shell and could cause recursion)\n\
         got: {stdout:?}"
    );
}

/// Parse the raw stdout of `alias` into a list of alias names.
///
/// zsh `alias` output lines follow the grammar:
///   name=value            (no special chars)
///   name='quoted value'   (spaces / special chars)
///   name='it'\''s here'   (embedded single-quote, escaped as `'\''`)
///
/// Only the key (left of the first `=`) is needed here.
fn parse_zsh_alias_keys(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            // Everything before the first `=` is the alias name.
            line.split('=').next().map(|k| k.to_string())
        })
        .collect()
}

#[test]
fn zsh_interactive_alias_output_contains_la() {
    // Spawn an interactive zsh, run `alias`, capture stdout.
    // stderr is inherited (or can be suppressed) – we only process stdout.
    let output = Command::new("zsh")
        .args(["-i", "-c", "alias"])
        .env("_AM_DETECTING_ALIASES", "1") // guard: prevents recursive `am` invocation
        .output()
        .expect("failed to spawn zsh — is zsh installed?");

    assert!(
        output.status.success() || output.status.code() == Some(1),
        "zsh exited with unexpected status: {}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        !stdout.is_empty(),
        "zsh produced no alias output — is ~/.zshrc sourced correctly?"
    );

    let keys = parse_zsh_alias_keys(&stdout);

    // Sanity: should have captured many aliases (interactive zsh loads plugins)
    assert!(
        keys.len() > 5,
        "expected many aliases from interactive zsh, got only {}: check that zsh \
         sources ~/.zshrc in interactive mode",
        keys.len()
    );

    // Concrete proof: `la` must be present (defined as `la='ls -lAh'` in the
    // user's zsh config).
    assert!(
        keys.contains(&"la".to_string()),
        "`la` alias not found among {} aliases: {:?}",
        keys.len(),
        keys
    );
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
