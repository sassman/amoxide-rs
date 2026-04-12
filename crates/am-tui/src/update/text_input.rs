use crate::model::{
    AliasField, AliasId, AliasTarget, Mode, NodeKind, SubcommandField, TextInputState, TuiMessage,
    TuiModel,
};
use amoxide::AliasName;

pub fn handle(model: &mut TuiModel, msg: TuiMessage) {
    match msg {
        TuiMessage::StartCreateProfile => {
            if model.mode == Mode::Normal {
                model.mode = Mode::TextInput(TextInputState::NewProfile(String::new()));
            }
        }
        TuiMessage::StartAddAlias => {
            if model.mode != Mode::Normal {
                return;
            }
            // If cursor is on a subcommand node, open the subcommand editor
            let node_kind = model.tree.get(model.cursor).map(|n| n.kind.clone());
            match node_kind {
                Some(NodeKind::SubcommandProgramHeader) => {
                    let program = model
                        .tree
                        .get(model.cursor)
                        .and_then(|n| n.label.split_whitespace().next().map(|s| s.to_string()))
                        .unwrap_or_default();
                    let target = resolve_scope_from_ancestors(model).unwrap_or(AliasTarget::Global);
                    model.mode = Mode::TextInput(TextInputState::SubcommandInput {
                        program,
                        pairs: vec![("".into(), "".into())],
                        active_pair: 0,
                        active_field: SubcommandField::Short,
                        target,
                        original_key: None,
                    });
                    return;
                }
                Some(NodeKind::SubcommandGroupNode) => {
                    let target = resolve_scope_from_ancestors(model).unwrap_or(AliasTarget::Global);
                    let program = find_parent_program(model).unwrap_or_default();
                    let pairs = collect_pairs_to_cursor(model);
                    let active_pair = pairs.len().saturating_sub(1);
                    model.mode = Mode::TextInput(TextInputState::SubcommandInput {
                        program,
                        pairs,
                        active_pair,
                        active_field: SubcommandField::Short,
                        target,
                        original_key: None,
                    });
                    return;
                }
                Some(NodeKind::SubcommandItem) => {
                    let target = resolve_scope_from_ancestors(model).unwrap_or(AliasTarget::Global);
                    let program = find_parent_program(model).unwrap_or_default();
                    let pairs = collect_pairs_to_cursor(model);
                    let active_pair = pairs.len().saturating_sub(1);
                    model.mode = Mode::TextInput(TextInputState::SubcommandInput {
                        program,
                        pairs,
                        active_pair,
                        active_field: SubcommandField::Short,
                        target,
                        original_key: None,
                    });
                    return;
                }
                _ => {}
            }
            // Determine target scope from cursor position
            let target = resolve_alias_target(model);
            if let Some(target) = target {
                model.mode = Mode::TextInput(TextInputState::NewAlias {
                    name: String::new(),
                    command: String::new(),
                    active_field: AliasField::Name,
                    target,
                });
            }
        }
        TuiMessage::EditItem => {
            if model.mode != Mode::Normal {
                return;
            }
            let node = match model.tree.get(model.cursor) {
                Some(n) => n.clone(),
                None => return,
            };
            match node.kind {
                NodeKind::ProfileHeader => {
                    model.mode = Mode::TextInput(TextInputState::EditProfile {
                        original_name: node.label.clone(),
                        name: node.label.clone(),
                        error: None,
                    });
                }
                NodeKind::AliasItem => {
                    if let (Some(id), Some(cmd)) =
                        (node.alias_id.clone(), node.alias_command.clone())
                    {
                        model.mode = Mode::TextInput(TextInputState::EditAlias {
                            alias_id: id,
                            name: node.label.clone(),
                            command: cmd,
                            active_field: AliasField::Name,
                            error: None,
                        });
                    }
                }
                NodeKind::SubcommandItem => {
                    if let Some(AliasId::Subcommand { scope, key }) = &node.alias_id {
                        let target = match scope {
                            amoxide::SubcommandScope::Global => AliasTarget::Global,
                            amoxide::SubcommandScope::Profile(n) => AliasTarget::Profile(n.clone()),
                            amoxide::SubcommandScope::Project => AliasTarget::Project,
                        };
                        let pairs = collect_pairs_to_cursor(model);
                        let program = key.split(':').next().unwrap_or("").to_string();
                        let active_pair = pairs.len().saturating_sub(1);
                        model.mode = Mode::TextInput(TextInputState::SubcommandInput {
                            program,
                            pairs,
                            active_pair,
                            active_field: SubcommandField::Short,
                            target,
                            original_key: Some(key.clone()),
                        });
                    }
                }
                NodeKind::SubcommandGroupNode | NodeKind::SubcommandProgramHeader => {
                    handle(model, TuiMessage::StartAddAlias);
                }
                _ => {}
            }
        }
        TuiMessage::TextInputChar(c) => {
            if let Mode::TextInput(ref mut state) = model.mode {
                match state {
                    TextInputState::NewProfile(buf) => {
                        buf.push(c);
                    }
                    TextInputState::NewAlias {
                        name,
                        command,
                        active_field,
                        ..
                    } => match active_field {
                        AliasField::Name => {
                            name.push(c);
                        }
                        AliasField::Command => {
                            command.push(c);
                        }
                    },
                    TextInputState::EditProfile { name, error, .. } => {
                        *error = None;
                        name.push(c);
                    }
                    TextInputState::EditAlias {
                        name,
                        command,
                        active_field,
                        error,
                        ..
                    } => {
                        *error = None;
                        match active_field {
                            AliasField::Name => name.push(c),
                            AliasField::Command => command.push(c),
                        }
                    }
                    TextInputState::SubcommandInput {
                        pairs,
                        active_pair,
                        active_field,
                        ..
                    } => {
                        if let Some((short, long)) = pairs.get_mut(*active_pair) {
                            match active_field {
                                SubcommandField::Short => short.push(c),
                                SubcommandField::Long => long.push(c),
                            }
                        }
                    }
                }
            }
        }
        TuiMessage::TextInputBackspace => {
            if let Mode::TextInput(ref mut state) = model.mode {
                match state {
                    TextInputState::NewProfile(buf) => {
                        buf.pop();
                    }
                    TextInputState::NewAlias {
                        name,
                        command,
                        active_field,
                        ..
                    } => match active_field {
                        AliasField::Name => {
                            name.pop();
                        }
                        AliasField::Command => {
                            command.pop();
                        }
                    },
                    TextInputState::EditProfile { name, error, .. } => {
                        *error = None;
                        name.pop();
                    }
                    TextInputState::EditAlias {
                        name,
                        command,
                        active_field,
                        error,
                        ..
                    } => {
                        *error = None;
                        match active_field {
                            AliasField::Name => {
                                name.pop();
                            }
                            AliasField::Command => {
                                command.pop();
                            }
                        }
                    }
                    TextInputState::SubcommandInput {
                        pairs,
                        active_pair,
                        active_field,
                        ..
                    } => {
                        if let Some((short, long)) = pairs.get_mut(*active_pair) {
                            match active_field {
                                SubcommandField::Short => {
                                    short.pop();
                                }
                                SubcommandField::Long => {
                                    long.pop();
                                }
                            }
                        }
                    }
                }
            }
        }
        TuiMessage::TextInputSwitchField => match &mut model.mode {
            Mode::TextInput(
                TextInputState::NewAlias { active_field, .. }
                | TextInputState::EditAlias { active_field, .. },
            ) => {
                *active_field = match active_field {
                    AliasField::Name => AliasField::Command,
                    AliasField::Command => AliasField::Name,
                };
            }
            Mode::TextInput(TextInputState::SubcommandInput {
                pairs,
                active_pair,
                active_field,
                ..
            }) => match active_field {
                SubcommandField::Short => {
                    *active_field = SubcommandField::Long;
                }
                SubcommandField::Long => {
                    if *active_pair + 1 < pairs.len() {
                        *active_pair += 1;
                        *active_field = SubcommandField::Short;
                    } else {
                        *active_field = SubcommandField::Short;
                    }
                }
            },
            _ => {}
        },
        TuiMessage::TextInputConfirm => {
            let state = match &model.mode {
                Mode::TextInput(s) => s.clone(),
                _ => return,
            };
            match state {
                TextInputState::NewProfile(name) => {
                    if name.is_empty() {
                        return;
                    }
                    if model
                        .app_model
                        .profile_config()
                        .get_profile_by_name(&name)
                        .is_some()
                    {
                        return;
                    }
                    let _ =
                        super::delegation::dispatch(model, amoxide::Message::CreateProfile(name));
                    model.mode = Mode::Normal;
                }
                TextInputState::NewAlias {
                    name,
                    command,
                    target,
                    ..
                } => {
                    let name = name.trim().to_string();
                    let command = command.trim().to_string();
                    if name.is_empty() || command.is_empty() {
                        return;
                    }
                    let lib_target = match &target {
                        AliasTarget::Global => amoxide::AliasTarget::Global,
                        AliasTarget::Profile(n) => amoxide::AliasTarget::Profile(n.clone()),
                        AliasTarget::Project => amoxide::AliasTarget::Local,
                    };
                    let _ = super::delegation::dispatch(
                        model,
                        amoxide::Message::AddAlias(name, command, lib_target, false),
                    );
                    model.mode = Mode::Normal;
                }
                TextInputState::EditProfile {
                    original_name,
                    name,
                    ..
                } => {
                    let name = name.trim().to_string();
                    if name.is_empty() {
                        return;
                    }
                    if name == original_name {
                        model.mode = Mode::Normal;
                        return;
                    }
                    if model
                        .app_model
                        .profile_config()
                        .get_profile_by_name(&name)
                        .is_some()
                    {
                        if let Mode::TextInput(TextInputState::EditProfile { error, .. }) =
                            &mut model.mode
                        {
                            *error = Some(format!("profile '{}' already exists", name));
                        }
                        return;
                    }
                    let _ = super::delegation::dispatch(
                        model,
                        amoxide::Message::RenameProfile {
                            old_name: original_name,
                            new_name: name,
                        },
                    );
                    model.mode = Mode::Normal;
                }
                TextInputState::EditAlias {
                    alias_id,
                    name,
                    command,
                    ..
                } => {
                    let new_name = name.trim().to_string();
                    let new_command = command.trim().to_string();
                    if new_name.is_empty() || new_command.is_empty() {
                        return;
                    }
                    let original_name = match &alias_id {
                        AliasId::Global { alias_name }
                        | AliasId::Profile { alias_name, .. }
                        | AliasId::Project { alias_name } => alias_name.clone(),
                        AliasId::Subcommand { key, .. } => key.clone(),
                    };
                    // No change — just exit
                    if new_name == original_name {
                        let key = AliasName::from(original_name.as_str());
                        let original_command = match &alias_id {
                            AliasId::Global { .. } => model
                                .app_model
                                .config
                                .aliases
                                .get(&key)
                                .map(|a| a.command().to_string()),
                            AliasId::Profile { profile_name, .. } => model
                                .app_model
                                .profile_config()
                                .get_profile_by_name(profile_name)
                                .and_then(|p| p.aliases.get(&key).map(|a| a.command().to_string())),
                            AliasId::Project { .. } => model
                                .app_model
                                .project_aliases()
                                .and_then(|p| p.aliases.get(&key).map(|a| a.command().to_string())),
                            AliasId::Subcommand { .. } => None,
                        };
                        if original_command.as_deref() == Some(new_command.as_str()) {
                            model.mode = Mode::Normal;
                            return;
                        }
                    }
                    // Name collision check
                    if new_name != original_name {
                        let key = AliasName::from(new_name.as_str());
                        let exists = match &alias_id {
                            AliasId::Global { .. } => {
                                model.app_model.config.aliases.contains_key(&key)
                            }
                            AliasId::Profile { profile_name, .. } => model
                                .app_model
                                .profile_config()
                                .get_profile_by_name(profile_name)
                                .is_some_and(|p| p.aliases.contains_key(&key)),
                            AliasId::Project { .. } => model
                                .app_model
                                .project_aliases()
                                .is_some_and(|p| p.aliases.contains_key(&key)),
                            AliasId::Subcommand { .. } => false,
                        };
                        if exists {
                            if let Mode::TextInput(TextInputState::EditAlias { error, .. }) =
                                &mut model.mode
                            {
                                *error = Some(format!(
                                    "name '{}' already exists in this scope",
                                    new_name
                                ));
                            }
                            return;
                        }
                    }
                    // Apply via dispatch
                    let lib_target = match &alias_id {
                        AliasId::Global { .. } => amoxide::AliasTarget::Global,
                        AliasId::Profile { profile_name, .. } => {
                            amoxide::AliasTarget::Profile(profile_name.clone())
                        }
                        AliasId::Project { .. } => amoxide::AliasTarget::Local,
                        AliasId::Subcommand { .. } => return, // handled by SubcommandInput arm
                    };
                    let _ = super::delegation::dispatch(
                        model,
                        amoxide::Message::UpdateAlias {
                            target: lib_target,
                            old_name: original_name,
                            new_name,
                            new_command,
                            raw: false,
                        },
                    );
                    model.mode = Mode::Normal;
                }
                TextInputState::SubcommandInput {
                    program,
                    pairs,
                    target,
                    original_key,
                    ..
                } => {
                    let valid = pairs.iter().all(|(s, l)| !s.is_empty() && !l.is_empty());
                    if !valid {
                        return;
                    }
                    let key = format!(
                        "{}:{}",
                        program,
                        pairs
                            .iter()
                            .map(|(s, _)| s.as_str())
                            .collect::<Vec<_>>()
                            .join(":")
                    );
                    let long_subcommands: Vec<String> =
                        pairs.iter().map(|(_, l)| l.clone()).collect();
                    let lib_target = match &target {
                        AliasTarget::Global => amoxide::AliasTarget::Global,
                        AliasTarget::Profile(n) => amoxide::AliasTarget::Profile(n.clone()),
                        AliasTarget::Project => amoxide::AliasTarget::Local,
                    };
                    let msg = if let Some(orig) = original_key.clone() {
                        amoxide::Message::UpdateSubcommandAlias {
                            original_key: orig,
                            new_key: key,
                            long_subcommands,
                            target: lib_target,
                        }
                    } else {
                        amoxide::Message::AddSubcommandAlias(key, long_subcommands, lib_target)
                    };
                    let _ = super::delegation::dispatch(model, msg);
                    model.mode = Mode::Normal;
                }
            }
        }
        TuiMessage::TextInputCancel => {
            model.mode = Mode::Normal;
        }
        TuiMessage::SubcommandAddPair => {
            if let Mode::TextInput(TextInputState::SubcommandInput {
                pairs,
                active_pair,
                active_field,
                ..
            }) = &mut model.mode
            {
                pairs.push(("".into(), "".into()));
                *active_pair = pairs.len() - 1;
                *active_field = SubcommandField::Short;
            }
        }
        _ => {}
    }
}

/// Determine the alias target scope from the current cursor position.
/// Returns None if cursor is not on a node that implies a scope.
pub(super) fn resolve_alias_target(model: &TuiModel) -> Option<AliasTarget> {
    let node = model.tree.get(model.cursor)?;
    match &node.kind {
        NodeKind::GlobalHeader => Some(AliasTarget::Global),
        NodeKind::ProjectHeader => Some(AliasTarget::Project),
        NodeKind::ProfileHeader => Some(AliasTarget::Profile(node.label.clone())),
        NodeKind::AliasItem => {
            // Derive target from the alias's scope
            match &node.alias_id {
                Some(AliasId::Global { .. }) => Some(AliasTarget::Global),
                Some(AliasId::Profile { profile_name, .. }) => {
                    Some(AliasTarget::Profile(profile_name.clone()))
                }
                Some(AliasId::Project { .. }) => Some(AliasTarget::Project),
                Some(AliasId::Subcommand { .. }) | None => None,
            }
        }
        NodeKind::SubcommandProgramHeader
        | NodeKind::SubcommandGroupNode
        | NodeKind::SubcommandItem => resolve_scope_from_ancestors(model),
    }
}

/// Walk backward through the tree to find the parent scope header.
pub(super) fn resolve_scope_from_ancestors(model: &TuiModel) -> Option<AliasTarget> {
    for i in (0..=model.cursor).rev() {
        match &model.tree[i].kind {
            NodeKind::GlobalHeader => return Some(AliasTarget::Global),
            NodeKind::ProjectHeader => return Some(AliasTarget::Project),
            NodeKind::ProfileHeader => {
                return Some(AliasTarget::Profile(model.tree[i].label.clone()))
            }
            _ => {}
        }
    }
    Some(AliasTarget::Global)
}

/// Walk backward to find the nearest SubcommandProgramHeader and return its program name.
fn find_parent_program(model: &TuiModel) -> Option<String> {
    for i in (0..=model.cursor).rev() {
        if model.tree[i].kind == NodeKind::SubcommandProgramHeader {
            return model.tree[i]
                .label
                .split_whitespace()
                .next()
                .map(|s| s.to_string());
        }
    }
    None
}

/// Collect short→long pairs representing the path from the SubcommandProgramHeader to the cursor.
/// Only ancestor GroupNodes (identified by having a strictly shorter prefix than the previous
/// collected node) are included, avoiding sibling contamination.
fn collect_pairs_to_cursor(model: &TuiModel) -> Vec<(String, String)> {
    let header_idx = (0..=model.cursor)
        .rev()
        .find(|&i| model.tree[i].kind == NodeKind::SubcommandProgramHeader);
    let Some(start) = header_idx else {
        return vec![("".into(), "".into())];
    };

    let cursor_node = &model.tree[model.cursor];
    let cursor_pair = match cursor_node.kind {
        NodeKind::SubcommandItem => {
            if let Some((short, long)) = cursor_node.label.split_once(" \u{2192} ") {
                (short.trim().to_string(), long.trim().to_string())
            } else {
                (cursor_node.label.clone(), String::new())
            }
        }
        NodeKind::SubcommandGroupNode => (cursor_node.label.clone(), String::new()),
        _ => ("".into(), "".into()),
    };

    // Walk backwards from just before the cursor.
    // A GroupNode is an ancestor of the cursor if its prefix is strictly shorter
    // than the last collected node's prefix (deeper nodes have longer prefixes).
    let mut path = vec![cursor_pair];
    let mut min_prefix_len = cursor_node.prefix.len();

    for i in (start + 1..model.cursor).rev() {
        let node = &model.tree[i];
        if node.kind == NodeKind::SubcommandGroupNode && node.prefix.len() < min_prefix_len {
            path.push((node.label.clone(), String::new()));
            min_prefix_len = node.prefix.len();
        }
    }

    path.reverse();

    if path.is_empty() {
        path.push(("".into(), "".into()));
    }
    path
}
