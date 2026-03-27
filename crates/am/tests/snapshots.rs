use amoxide::display::{render_listing, render_profiles};
use amoxide::hook::generate_hook;
use amoxide::init::{generate_init, generate_reload};
use amoxide::shell::Shells;
use amoxide::{AliasName, AliasSet, ProfileConfig, TomlAlias};
use indoc::indoc;
use std::fs;

/// Build a ProfileConfig from TOML string.
fn profiles(toml_str: &str) -> ProfileConfig {
    toml::from_str(toml_str).unwrap()
}

/// Build an AliasSet with given (name, command) pairs.
fn aliases(pairs: &[(&str, &str)]) -> AliasSet {
    let mut set = AliasSet::default();
    for (name, cmd) in pairs {
        set.insert(AliasName::from(*name), TomlAlias::Command(cmd.to_string()));
    }
    set
}

// ─── Test bed: git + git-conventional as separate profiles ──────────────

fn git_conventional_config() -> ProfileConfig {
    profiles(indoc! {r#"
        [[profiles]]
        name = "default"

        [[profiles]]
        name = "git"
        [profiles.aliases]
        gs = "git status"
        cm = "git commit -sm"

        [[profiles]]
        name = "git-conventional"
        [profiles.aliases]
        cmf = "cm feat: {{@}}"
    "#})
}

// ─── Test bed: base, git, rust as separate profiles ─────────────────────

fn deep_chain_config() -> ProfileConfig {
    profiles(indoc! {r#"
        [[profiles]]
        name = "base"
        [profiles.aliases]
        ll = "ls -lha"

        [[profiles]]
        name = "git"
        [profiles.aliases]
        gs = "git status"

        [[profiles]]
        name = "rust"
        [profiles.aliases]
        ct = "cargo test"
    "#})
}

// ═══════════════════════════════════════════════════════════════════════
// Init snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_init_fish_simple_profile() {
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "default"
        [profiles.aliases]
        ll = "ls -lha"
        gs = "git status"
    "#});
    let resolved = config.resolve_active_aliases(&["default"]);
    let output = generate_init(&Shells::Fish, &AliasSet::default(), &resolved);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_zsh_simple_profile() {
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "default"
        [profiles.aliases]
        ll = "ls -lha"
        gs = "git status"
    "#});
    let resolved = config.resolve_active_aliases(&["default"]);
    let output = generate_init(&Shells::Zsh, &AliasSet::default(), &resolved);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_powershell_simple_profile() {
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "default"
        [profiles.aliases]
        ll = "ls -lha"
        gs = "git status"
    "#});
    let resolved = config.resolve_active_aliases(&["default"]);
    let output = generate_init(&Shells::Powershell, &AliasSet::default(), &resolved);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_multi_profile() {
    let config = git_conventional_config();
    let resolved =
        config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_init(&Shells::Fish, &AliasSet::default(), &resolved);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_with_globals() {
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "rust"
        [profiles.aliases]
        ct = "cargo test"
    "#});
    let globals = aliases(&[("ll", "ls -lha")]);
    let resolved = config.resolve_active_aliases(&["rust"]);
    let output = generate_init(&Shells::Fish, &globals, &resolved);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_deep_chain() {
    let config = deep_chain_config();
    let resolved =
        config.resolve_active_aliases(&["base", "git", "rust"]);
    let output = generate_init(&Shells::Fish, &AliasSet::default(), &resolved);
    insta::assert_snapshot!(output);
}

// ═══════════════════════════════════════════════════════════════════════
// Reload snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_reload_fish_switch_profile() {
    let config = git_conventional_config();
    let resolved =
        config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_reload(
        &Shells::Fish,
        &AliasSet::default(),
        &resolved,
        Some("gs,cm"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_zsh_switch_profile() {
    let config = git_conventional_config();
    let resolved =
        config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_reload(&Shells::Zsh, &AliasSet::default(), &resolved, Some("gs,cm"));
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_powershell_switch_profile() {
    let config = git_conventional_config();
    let resolved = config.resolve_active_aliases(&["git-conventional"]);
    let output = generate_reload(
        &Shells::Powershell,
        &AliasSet::default(),
        &resolved,
        Some("gs,cm"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_fish_after_global_add() {
    // Simulates: user had profile aliases loaded, then adds a global alias
    let globals = aliases(&[("ll", "ls -lha")]);
    let config = git_conventional_config();
    let resolved =
        config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_reload(
        &Shells::Fish,
        &globals,
        &resolved,
        Some("cm,cmf,gs"), // previously tracked aliases
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_fish_globals_only_no_profile() {
    // No active profile, only globals
    let globals = aliases(&[("ll", "ls -lha"), ("gs", "git status")]);
    let output = generate_reload(&Shells::Fish, &globals, &AliasSet::default(), Some("old"));
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_zsh_after_global_add() {
    let globals = aliases(&[("ll", "ls -lha")]);
    let config = git_conventional_config();
    let resolved =
        config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_reload(&Shells::Zsh, &globals, &resolved, Some("cm,cmf,gs"));
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_globals_and_multi_profile() {
    // Full scenario: globals + multiple active profiles
    let globals = aliases(&[("ll", "ls -lha")]);
    let config = git_conventional_config();
    let resolved =
        config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_init(&Shells::Fish, &globals, &resolved);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_after_profile_removed() {
    // Scenario: rust profile had its own aliases, then was removed from active set
    // Now only git's aliases should remain
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "git"
        [profiles.aliases]
        gs = "git status"
        cm = "git commit -sm"

        [[profiles]]
        name = "rust"
        [profiles.aliases]
        ct = "cargo test"
    "#});
    // rust is no longer in the active set
    let resolved = config.resolve_active_aliases(&["git"]);
    // Previously tracked: cm,ct,gs (git's + rust's aliases were all loaded)
    let output = generate_reload(
        &Shells::Fish,
        &AliasSet::default(),
        &resolved,
        Some("cm,ct,gs"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_after_parent_profile_removed() {
    // Scenario: git was removed, git-conventional is now standalone
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "git-conventional"
        [profiles.aliases]
        cmf = "cm feat: {{@}}"
    "#});
    // git was removed, only git-conventional active
    let resolved = config.resolve_active_aliases(&["git-conventional"]);
    // Previously tracked: cm,cmf,gs (git's + git-conventional's were loaded)
    let output = generate_reload(
        &Shells::Fish,
        &AliasSet::default(),
        &resolved,
        Some("cm,cmf,gs"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_after_active_set_changed() {
    // Scenario: previously had git+rust active, now changed to base+rust
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "base"
        [profiles.aliases]
        ll = "ls -lha"

        [[profiles]]
        name = "git"
        [profiles.aliases]
        gs = "git status"

        [[profiles]]
        name = "rust"
        [profiles.aliases]
        ct = "cargo test"
    "#});
    let resolved = config.resolve_active_aliases(&["base", "rust"]);
    // Previously tracked: ct,gs (from rust + git)
    // Now should have: ct,ll (from rust + base)
    let output = generate_reload(
        &Shells::Fish,
        &AliasSet::default(),
        &resolved,
        Some("ct,gs"),
    );
    insta::assert_snapshot!(output);
}

// ═══════════════════════════════════════════════════════════════════════
// Hook snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_hook_fish_with_aliases() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join(".aliases"),
        indoc! {r#"
            [aliases]
            t = "cargo test"
            b = "cargo build"
        "#},
    )
    .unwrap();

    let output = generate_hook(&Shells::Fish, dir.path(), None).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_hook_zsh_with_aliases() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join(".aliases"),
        indoc! {r#"
            [aliases]
            t = "cargo test"
            b = "cargo build"
        "#},
    )
    .unwrap();

    let output = generate_hook(&Shells::Zsh, dir.path(), None).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_hook_powershell_with_aliases() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join(".aliases"),
        indoc! {r#"
            [aliases]
            t = "cargo test"
            b = "cargo build"
        "#},
    )
    .unwrap();

    let output = generate_hook(&Shells::Powershell, dir.path(), None).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_hook_fish_transition() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join(".aliases"),
        indoc! {r#"
            [aliases]
            t = "make test"
        "#},
    )
    .unwrap();

    let output = generate_hook(&Shells::Fish, dir.path(), Some("old_a,old_b")).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_hook_fish_leaving_project() {
    let dir = tempfile::tempdir().unwrap();
    // No .aliases file
    let output = generate_hook(&Shells::Fish, dir.path(), Some("old_a,old_b")).unwrap();
    insta::assert_snapshot!(output);
}

// ═══════════════════════════════════════════════════════════════════════
// Display snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_display_profile_list() {
    let config = git_conventional_config();
    let output = render_profiles(&config, &["git-conventional".to_string()]);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_display_multi_active() {
    let config = deep_chain_config();
    let output = render_profiles(&config, &["base".to_string(), "rust".to_string()]);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_display_listing_with_globals_and_project() {
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "rust"
        [profiles.aliases]
        ct = "cargo test"
    "#});
    let globals = aliases(&[("ll", "ls -lha")]);

    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join(".aliases"),
        indoc! {r#"
            [aliases]
            b = "make build"
        "#},
    )
    .unwrap();

    let output = render_listing(&globals, &config, &["rust".to_string()], dir.path());
    insta::assert_snapshot!(output);
}
