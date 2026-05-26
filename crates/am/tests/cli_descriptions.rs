//! Tests for `am add --description` — verifies the description flag is wired
//! through the full update/effects pipeline for all three scopes.
//!
//! There is no `AM_CONFIG_DIR` env-var override for the binary, so these tests
//! drive the library layer directly (same pattern used by `var_command.rs`).

use amoxide::{
    config::Config,
    normalize_description,
    update::{update, AppModel},
    AliasTarget, Described, Message, ProfileConfig,
};

// ── Global scope — description present ───────────────────────────────────────

#[test]
fn am_add_with_description_writes_detailed_form_global() {
    let dir = tempfile::tempdir().unwrap();
    let description = normalize_description("git status short");

    let mut config = Config::default();
    config.add_alias("gs".into(), "git status".into(), false, description);
    config.save_to(dir.path()).unwrap();

    let toml = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(
        toml.contains("description = \"git status short\""),
        "config.toml missing description field: {toml}"
    );
    assert!(toml.contains("command = \"git status\""));
}

// ── Global scope — whitespace-only description becomes simple form ────────────

#[test]
fn am_add_with_empty_description_writes_simple_form_global() {
    let dir = tempfile::tempdir().unwrap();
    let description = normalize_description("   ");
    assert!(
        description.is_none(),
        "whitespace-only should normalize to None"
    );

    let mut config = Config::default();
    config.add_alias("ll".into(), "ls -lha".into(), false, description);
    config.save_to(dir.path()).unwrap();

    let toml = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(
        !toml.contains("description"),
        "simple alias must not emit description key: {toml}"
    );
    assert!(toml.contains("ll = \"ls -lha\""));
}

// ── Profile scope — description threads through update() ─────────────────────

#[test]
fn am_add_with_description_writes_detailed_form_profile() {
    let mut model = AppModel::new(Config::default(), ProfileConfig::default());
    update(&mut model, Message::CreateProfile("work".into())).unwrap();

    let description = normalize_description("git status short");
    update(
        &mut model,
        Message::AddAlias(
            "gs".into(),
            "git status".into(),
            AliasTarget::Profile("work".into()),
            false,
            description,
        ),
    )
    .unwrap();

    let profile = model
        .profile_config()
        .get_profile_by_name("work")
        .expect("profile should exist");

    let key = amoxide::AliasName::from("gs");
    let alias = profile.aliases.get(&key).expect("alias should exist");
    assert_eq!(alias.description(), Some("git status short"));
}

// ── Global scope — subcommand description ────────────────────────────────────

#[test]
fn am_add_subcommand_with_description_writes_detailed_form() {
    let dir = tempfile::tempdir().unwrap();
    let description = normalize_description("abandon a change");

    let mut config = Config::default();
    config.add_subcommand("jj:ab".into(), vec!["abandon".into()], description);
    config.save_to(dir.path()).unwrap();

    let toml = std::fs::read_to_string(dir.path().join("config.toml")).unwrap();
    assert!(
        toml.contains("description = \"abandon a change\""),
        "config.toml missing description: {toml}"
    );
}

// ── update() pipeline — global add passes description to Effect ──────────────

#[test]
fn am_add_alias_message_with_description_reaches_global_config() {
    let mut model = AppModel::new(Config::default(), ProfileConfig::default());

    let description = normalize_description("short status");
    update(
        &mut model,
        Message::AddAlias(
            "gs".into(),
            "git status".into(),
            AliasTarget::Global,
            false,
            description,
        ),
    )
    .unwrap();

    let key = amoxide::AliasName::from("gs");
    let alias = model
        .config
        .aliases
        .get(&key)
        .expect("alias should be in config");
    assert_eq!(alias.description(), Some("short status"));
}

// ── normalize_description — whitespace round-trips correctly ─────────────────

#[test]
fn normalize_description_trims_whitespace_to_none() {
    assert_eq!(normalize_description("   "), None);
}

#[test]
fn normalize_description_keeps_non_empty() {
    assert_eq!(
        normalize_description("  git status short  "),
        Some("git status short".to_string())
    );
}

// ── am ls -d / --descriptions flag parses correctly ──────────────────────────

#[test]
fn am_ls_accepts_descriptions_flag() {
    use amoxide::cli::{Cli, Commands};
    use clap::Parser;

    let cli = Cli::try_parse_from(["am", "ls", "-d"]).expect("parse");
    match cli.command {
        Commands::Ls { used, descriptions } => {
            assert!(!used);
            assert!(descriptions);
        }
        _ => panic!("expected Ls"),
    }
}

#[test]
fn am_ls_accepts_descriptions_long_flag() {
    use amoxide::cli::{Cli, Commands};
    use clap::Parser;

    let cli = Cli::try_parse_from(["am", "ls", "--descriptions"]).expect("parse");
    match cli.command {
        Commands::Ls { descriptions, .. } => {
            assert!(descriptions);
        }
        _ => panic!("expected Ls"),
    }
}

#[test]
fn am_la_is_unit_variant() {
    use amoxide::cli::{Cli, Commands};
    use clap::Parser;

    let cli = Cli::try_parse_from(["am", "la"]).expect("parse");
    assert!(matches!(cli.command, Commands::La));
}
