//! End-to-end test that var substitution + sync produces correct shell output.

use amoxide::{
    alias::{AliasName, AliasSet, TomlAlias},
    precedence::{Precedence, ProfileLayer},
    subcommand::SubcommandSet,
    vars::{VarName, VarSet},
};

fn aset(pairs: &[(&str, &str)]) -> AliasSet {
    let mut s = AliasSet::default();
    for (n, c) in pairs {
        s.insert(AliasName::from(*n), TomlAlias::Command((*c).into()));
    }
    s
}

fn vset(pairs: &[(&str, &str)]) -> VarSet {
    let mut v = VarSet::default();
    for (k, val) in pairs {
        v.insert(VarName::parse(k).unwrap(), (*val).to_string());
    }
    v
}

#[test]
fn global_alias_with_global_var_substitutes_in_shell_output() {
    let aliases = aset(&[("hello", "echo {{who}}")]);
    let vars = vset(&[("who", "world")]);
    let outcome = Precedence::new()
        .with_global(&aliases, &SubcommandSet::new(), &vars)
        .resolve();
    assert_eq!(outcome.diff.added.len(), 1);
    let entry = &outcome.diff.added[0];
    match &entry.kind {
        amoxide::precedence::EntryKind::Alias(a) => assert_eq!(a.command(), "echo world"),
        _ => panic!("unexpected entry kind"),
    }
    assert!(outcome.diagnostics.is_empty());
}

#[test]
fn alias_with_missing_var_emits_diagnostic() {
    let aliases = aset(&[("cc", "compile {{flags}}")]);
    let vars = VarSet::default();
    let outcome = Precedence::new()
        .with_global(&aliases, &SubcommandSet::new(), &vars)
        .resolve();
    assert_eq!(outcome.diagnostics.len(), 1);
    let msg = &outcome.diagnostics[0].message;
    assert!(msg.contains("cc"), "{msg}");
    assert!(msg.contains("flags"), "{msg}");
    assert!(outcome.diff.invalid.len() == 1);
}

#[test]
fn switching_active_profile_changes_alias_value() {
    let aliases_p1 = aset(&[("run", "exec {{path}}/run.sh")]);
    let vars_p1 = vset(&[("path", "/v1")]);
    let aliases_p2 = aset(&[("run", "exec {{path}}/run.sh")]);
    let vars_p2 = vset(&[("path", "/v2")]);

    let outcome_p1 = Precedence::new()
        .with_profiles(&[ProfileLayer {
            name: "p1".into(),
            aliases: aliases_p1,
            subcommands: SubcommandSet::new(),
            vars: vars_p1,
        }])
        .resolve();
    let run_p1 = outcome_p1
        .diff
        .added
        .iter()
        .find(|e| e.name == "run")
        .unwrap();
    match &run_p1.kind {
        amoxide::precedence::EntryKind::Alias(a) => assert_eq!(a.command(), "exec /v1/run.sh"),
        _ => panic!(),
    }

    let outcome_p2 = Precedence::new()
        .with_profiles(&[ProfileLayer {
            name: "p2".into(),
            aliases: aliases_p2,
            subcommands: SubcommandSet::new(),
            vars: vars_p2,
        }])
        .resolve();
    let run_p2 = outcome_p2
        .diff
        .added
        .iter()
        .find(|e| e.name == "run")
        .unwrap();
    assert_ne!(run_p1.hash, run_p2.hash, "hash must differ across profiles");
}
