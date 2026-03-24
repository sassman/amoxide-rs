use am::display::{render_listing, render_profile_tree};
use am::hook::generate_hook;
use am::init::{generate_init, generate_reload};
use am::shell::Shells;
use am::{AliasName, AliasSet, ProfileConfig, TomlAlias};
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

// ─── Test bed: git → git-conventional inheritance ───────────────────────

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
        inherits = "git"
        [profiles.aliases]
        cmf = "cm feat: {{@}}"
    "#})
}

// ─── Test bed: deep chain base → git → rust ─────────────────────────────

fn deep_chain_config() -> ProfileConfig {
    profiles(indoc! {r#"
        [[profiles]]
        name = "base"
        [profiles.aliases]
        ll = "ls -lha"

        [[profiles]]
        name = "git"
        inherits = "base"
        [profiles.aliases]
        gs = "git status"

        [[profiles]]
        name = "rust"
        inherits = "git"
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
    let resolved = config.resolve_aliases("default");
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
    let resolved = config.resolve_aliases("default");
    let output = generate_init(&Shells::Zsh, &AliasSet::default(), &resolved);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_inherited_profile() {
    let config = git_conventional_config();
    let resolved = config.resolve_aliases("git-conventional");
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
    let resolved = config.resolve_aliases("rust");
    let output = generate_init(&Shells::Fish, &globals, &resolved);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_deep_chain() {
    let config = deep_chain_config();
    let resolved = config.resolve_aliases("rust");
    let output = generate_init(&Shells::Fish, &AliasSet::default(), &resolved);
    insta::assert_snapshot!(output);
}

// ═══════════════════════════════════════════════════════════════════════
// Reload snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_reload_fish_switch_profile() {
    let config = git_conventional_config();
    let resolved = config.resolve_aliases("git-conventional");
    let output = generate_reload(&Shells::Fish, &resolved, Some("gs,cm"));
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_zsh_switch_profile() {
    let config = git_conventional_config();
    let resolved = config.resolve_aliases("git-conventional");
    let output = generate_reload(&Shells::Zsh, &resolved, Some("gs,cm"));
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
fn snapshot_display_inheritance_tree() {
    let config = git_conventional_config();
    let output = render_profile_tree(&config, "git-conventional");
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_display_deep_chain() {
    let config = deep_chain_config();
    let output = render_profile_tree(&config, "rust");
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

    let output = render_listing(&globals, &config, "rust", dir.path());
    insta::assert_snapshot!(output);
}
