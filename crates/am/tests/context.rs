//! E2E tests for `am context` output. Uses insta snapshots to lock in the
//! rendered markdown shape — if the format ever changes, regenerate snapshots
//! with `cargo insta review` after a deliberate decision.

use amoxide::alias::{AliasName, AliasSet, TomlAlias};
use amoxide::context::{render, ChainLayer, LayerInputs, PrecedenceChain, RenderOptions};
use amoxide::precedence::{OriginScope, Precedence, ProfileLayer};
use amoxide::subcommand::SubcommandSet;
use amoxide::vars::VarSet;
use std::path::PathBuf;

fn aset(pairs: &[(&str, &str)]) -> AliasSet {
    let mut s = AliasSet::default();
    for (n, c) in pairs {
        s.insert(AliasName::from(*n), TomlAlias::Command((*c).into()));
    }
    s
}

#[test]
fn snapshot_context_brief() {
    let global = aset(&[
        ("ll", "ls -lha"),
        ("claude-ft", "claude --dangerously-skip-permissions"),
    ]);
    let git = ProfileLayer {
        name: "git".into(),
        aliases: aset(&[("gm", "git commit -S --signoff -m"), ("gst", "git status")]),
        subcommands: SubcommandSet::new(),
        vars: VarSet::default(),
    };
    let rust = ProfileLayer {
        name: "rust".into(),
        aliases: aset(&[
            ("b", "cargo build --release"),
            ("f", "cargo fmt --check"), // shadowed by project's `f`
        ]),
        subcommands: SubcommandSet::new(),
        vars: VarSet::default(),
    };
    let project = aset(&[
        ("f", "cargo fmt"),
        ("docs", "fnm use 24 && cd website/ && npm run dev"),
    ]);
    let global_subs = SubcommandSet::new();
    let global_vars = VarSet::default();
    let project_subs = SubcommandSet::new();
    let project_vars = VarSet::default();

    let outcome = Precedence::new()
        .with_global(&global, &global_subs, &global_vars)
        .with_profiles(&[git.clone(), rust.clone()])
        .with_project(&project, &project_subs, &project_vars)
        .resolve();

    let profile_layers = vec![git, rust];
    let layers = LayerInputs {
        global_aliases: &global,
        global_subcommands: &global_subs,
        global_vars: &global_vars,
        profile_layers: &profile_layers,
        project_aliases: &project,
        project_subcommands: &project_subs,
        project_vars: &project_vars,
    };

    let chain = PrecedenceChain {
        layers: vec![
            ChainLayer {
                scope: OriginScope::Project,
                priority: None,
            },
            ChainLayer {
                scope: OriginScope::Profile("git".into()),
                priority: Some(1),
            },
            ChainLayer {
                scope: OriginScope::Profile("rust".into()),
                priority: Some(2),
            },
            ChainLayer {
                scope: OriginScope::Global,
                priority: None,
            },
        ],
    };

    let out = render(
        &PathBuf::from("/tmp/fixture-cwd"),
        &chain,
        &outcome,
        &layers,
        None,
        RenderOptions { verbose: false },
    );

    insta::assert_snapshot!(out);
}

#[test]
fn snapshot_context_verbose() {
    let global = aset(&[("f", "cargo fmt --check")]); // shadowed by rust + project
    let rust = ProfileLayer {
        name: "rust".into(),
        aliases: aset(&[("f", "cargo fmt --all")]), // shadowed by project
        subcommands: SubcommandSet::new(),
        vars: VarSet::default(),
    };
    let project = aset(&[("f", "cargo fmt")]);
    let global_subs = SubcommandSet::new();
    let global_vars = VarSet::default();
    let project_subs = SubcommandSet::new();
    let project_vars = VarSet::default();

    let outcome = Precedence::new()
        .with_global(&global, &global_subs, &global_vars)
        .with_profiles(std::slice::from_ref(&rust))
        .with_project(&project, &project_subs, &project_vars)
        .resolve();

    let profile_layers = vec![rust];
    let layers = LayerInputs {
        global_aliases: &global,
        global_subcommands: &global_subs,
        global_vars: &global_vars,
        profile_layers: &profile_layers,
        project_aliases: &project,
        project_subcommands: &project_subs,
        project_vars: &project_vars,
    };

    let chain = PrecedenceChain {
        layers: vec![
            ChainLayer {
                scope: OriginScope::Project,
                priority: None,
            },
            ChainLayer {
                scope: OriginScope::Profile("rust".into()),
                priority: Some(1),
            },
            ChainLayer {
                scope: OriginScope::Global,
                priority: None,
            },
        ],
    };

    let out = render(
        &PathBuf::from("/tmp/fixture-cwd"),
        &chain,
        &outcome,
        &layers,
        None,
        RenderOptions { verbose: true },
    );

    insta::assert_snapshot!(out);
}
