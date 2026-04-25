//! End-to-end CLI tests for `am var set/unset/get/list`.

use amoxide::{
    config::Config,
    update::{update, AppModel},
    AliasTarget, Message, ProfileConfig,
};

#[test]
fn set_then_get_global_var_roundtrip() {
    let mut model = AppModel::new(Config::default(), ProfileConfig::default());
    update(
        &mut model,
        Message::SetVar {
            target: AliasTarget::Global,
            name: "path".into(),
            value: "/opt/v1".into(),
        },
    )
    .unwrap();

    let result = update(
        &mut model,
        Message::GetVar {
            target: AliasTarget::Global,
            name: "path".into(),
        },
    )
    .unwrap();

    match result.effects.first() {
        Some(amoxide::effects::Effect::Print(v)) => assert_eq!(v, "/opt/v1"),
        other => panic!("expected Print, got {other:?}"),
    }
}

#[test]
fn set_then_unset_then_get_errors() {
    let mut model = AppModel::new(Config::default(), ProfileConfig::default());
    update(
        &mut model,
        Message::SetVar {
            target: AliasTarget::Global,
            name: "x".into(),
            value: "1".into(),
        },
    )
    .unwrap();
    update(
        &mut model,
        Message::UnsetVar {
            target: AliasTarget::Global,
            name: "x".into(),
        },
    )
    .unwrap();
    let err = update(
        &mut model,
        Message::GetVar {
            target: AliasTarget::Global,
            name: "x".into(),
        },
    )
    .unwrap_err();
    assert!(err.to_string().contains("x"));
}

#[test]
fn list_with_no_vars_prints_empty_message() {
    let mut model = AppModel::new(Config::default(), ProfileConfig::default());
    let result = update(&mut model, Message::ListVars { target: None }).unwrap();
    match result.effects.first() {
        Some(amoxide::effects::Effect::Print(s)) => {
            assert!(s.contains("no variables"), "got: {s}");
        }
        other => panic!("expected Print, got {other:?}"),
    }
}

#[test]
fn config_vars_persist_through_save_load_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.set_var(
        amoxide::vars::VarName::parse("opt-flags").unwrap(),
        "-C opt-level=3".into(),
    );
    config.save_to(dir.path()).unwrap();
    let loaded = Config::load_from(dir.path()).unwrap();
    let v = loaded
        .vars
        .get(&amoxide::vars::VarName::parse("opt-flags").unwrap())
        .map(String::as_str);
    assert_eq!(v, Some("-C opt-level=3"));
}
