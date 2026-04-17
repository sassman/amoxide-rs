/// End-to-end tests that require a real shell environment on the host machine.
///
/// All tests here are marked `#[ignore]` so they are skipped by the default
/// `cargo test` run.  Run them explicitly with:
///
///   cargo test --test e2e -- --ignored
///
/// CI primes the shell environment before invoking this suite.
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
#[ignore = "e2e: requires the am binary and a zsh installation"]
fn am_hook_is_silent_when_am_detecting_aliases_guard_is_active() {
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

/// Proof that `zsh -i -c 'alias'` enumerates pre-existing aliases from the
/// user's shell config.
///
/// CI primes `~/.zshrc` with `alias la='ls -lAh'` before running this suite.
/// On a local machine the alias must be present in `~/.zshrc` (or a sourced
/// plugin) for this test to pass.
#[test]
#[ignore = "e2e: requires zsh and la='ls -lAh' defined in ~/.zshrc"]
fn zsh_interactive_alias_output_contains_la() {
    let output = Command::new("zsh")
        .args(["-i", "-c", "alias"])
        .env("_AM_DETECTING_ALIASES", "1")
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

    let keys: Vec<String> = stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            line.split('=').next().map(|k| k.to_string())
        })
        .collect();

    assert!(
        keys.len() > 5,
        "expected many aliases from interactive zsh, got only {}: \
         check that zsh sources ~/.zshrc in interactive mode",
        keys.len()
    );

    assert!(
        keys.contains(&"la".to_string()),
        "`la` alias not found among {} aliases: {:?}",
        keys.len(),
        keys
    );
}
