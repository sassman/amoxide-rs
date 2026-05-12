//! Round-trip tests for variables through the export/import pipeline.

use amoxide::exchange::{parse_import, ExportAll, ParseSource, ScopeBundle};
use amoxide::vars::{VarName, VarSet};
use amoxide::{AliasSet, Profile, TomlAlias};

fn var(name: &str, value: &str) -> (VarName, String) {
    (VarName::parse(name).unwrap(), value.to_string())
}

fn varset(entries: &[(&str, &str)]) -> VarSet {
    let mut s = VarSet::default();
    for (n, v) in entries {
        let (name, value) = var(n, v);
        s.insert(name, value);
    }
    s
}

#[test]
fn vars_round_trip_v2_at_every_scope() {
    let export = ExportAll {
        global: ScopeBundle {
            aliases: AliasSet::default(),
            subcommands: Default::default(),
            vars: varset(&[("editor", "hx")]),
        },
        profiles: vec![Profile {
            name: "k8s".into(),
            aliases: {
                let mut a = AliasSet::default();
                a.insert(
                    "klogs".into(),
                    TomlAlias::Command("kubectl -n {{ns}} logs -f {{1}}".into()),
                );
                a
            },
            subcommands: Default::default(),
            vars: varset(&[("ns", "default")]),
        }],
        local: ScopeBundle {
            aliases: AliasSet::default(),
            subcommands: Default::default(),
            vars: varset(&[("target", "x86_64-unknown-linux-musl")]),
        },
        ..Default::default()
    };

    let toml_str = toml::to_string(&export).unwrap();
    let parsed = parse_import(&toml_str).unwrap();

    assert_eq!(parsed.source, ParseSource::V2);
    assert_eq!(parsed.export.meta.version, 2);

    assert_eq!(
        parsed
            .export
            .global
            .vars
            .get(&VarName::parse("editor").unwrap())
            .map(String::as_str),
        Some("hx"),
    );
    assert_eq!(
        parsed.export.profiles[0]
            .vars
            .get(&VarName::parse("ns").unwrap())
            .map(String::as_str),
        Some("default"),
    );
    assert_eq!(
        parsed
            .export
            .local
            .vars
            .get(&VarName::parse("target").unwrap())
            .map(String::as_str),
        Some("x86_64-unknown-linux-musl"),
    );
}

#[test]
fn legacy_v1_imports_with_empty_global_and_local_vars() {
    // amoxide < 0.9.0 wire format — no [meta], flat field names.
    let input = "\
[global_aliases]
ll = \"ls -lha\"

[[profiles]]
name = \"git\"

[profiles.aliases]
gs = \"git status\"

[local_aliases]
t = \"cargo test\"
";

    let parsed = parse_import(input).unwrap();
    assert_eq!(parsed.source, ParseSource::LegacyV1);
    assert!(parsed.export.global.vars.is_empty());
    assert!(parsed.export.local.vars.is_empty());
    assert!(parsed.export.profiles[0].vars.is_empty());

    // Aliases survive the lift.
    assert_eq!(parsed.export.global.aliases.iter().count(), 1);
    assert_eq!(parsed.export.profiles[0].aliases.iter().count(), 1);
    assert_eq!(parsed.export.local.aliases.iter().count(), 1);
}

#[test]
fn v2_unsupported_version_errors_with_helpful_message() {
    let input = "\
[meta]
version = 5

[global.aliases]
ll = \"ls -lha\"
";
    let err = parse_import(input).unwrap_err().to_string();
    assert!(err.contains("unsupported"));
    assert!(err.contains('5'));
    assert!(err.contains('2'));
}

#[test]
fn v2_export_emits_meta_block() {
    let export = ExportAll::default();
    let s = toml::to_string(&export).unwrap();
    assert!(s.contains("[meta]"));
    assert!(s.contains("version = 2"));
}
