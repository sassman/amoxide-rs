use crate::model::{
    AliasField, AliasId, AliasTarget, Mode, NodeKind, TextInputState, TuiMessage, TuiModel,
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
                }
            }
        }
        TuiMessage::TextInputSwitchField => {
            if let Mode::TextInput(
                TextInputState::NewAlias { active_field, .. }
                | TextInputState::EditAlias { active_field, .. },
            ) = &mut model.mode
            {
                *active_field = match active_field {
                    AliasField::Name => AliasField::Command,
                    AliasField::Command => AliasField::Name,
                };
            }
        }
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
                    let _ = super::delegation::dispatch(model, amoxide::Message::CreateProfile(name));
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
            }
        }
        TuiMessage::TextInputCancel => {
            model.mode = Mode::Normal;
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
                None => None,
            }
        }
    }
}
