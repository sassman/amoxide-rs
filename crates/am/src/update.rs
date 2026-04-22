pub use crate::app_model::AppModel;

use crate::display::render_listing;
use crate::effects::Effect;
use crate::env_vars;
use crate::init::{generate_init, generate_reload};
use crate::precedence::{self, Precedence};
use crate::profile::AliasCollection;
use crate::project::ProjectAliases;
use crate::shell::bash;
use crate::shell::zsh;
use crate::shell::Shell;
use crate::shell::ShellContext;
use crate::trust::ProjectTrust;
use crate::{profile, AliasDisplayFilter, AliasTarget, Message, Profile};

pub struct UpdateResult {
    pub next: Option<Message>,
    pub effects: Vec<Effect>,
}

impl UpdateResult {
    pub fn new(message: Message, effects: &[Effect]) -> Self {
        Self {
            next: Some(message),
            effects: effects.to_vec(),
        }
    }

    pub fn with_effects(effects: &[Effect]) -> Self {
        Self {
            next: None,
            effects: effects.to_vec(),
        }
    }

    pub fn message(message: Message) -> Self {
        Self {
            next: Some(message),
            effects: vec![],
        }
    }

    pub fn effect(effect: Effect) -> Self {
        Self {
            next: None,
            effects: vec![effect],
        }
    }

    pub fn done() -> Self {
        Self {
            next: None,
            effects: vec![],
        }
    }
}

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum UpdateError {
    #[error("project aliases are not trusted — run 'am trust'")]
    ProjectNotTrusted { path: std::path::PathBuf },

    #[error("alias '{name}' not found")]
    AliasNotFound { name: String, target: String },

    #[error("profile '{name}' not found")]
    ProfileNotFound { name: String },

    #[error("no .aliases file found in directory tree")]
    NoProjectFile,

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for UpdateError {
    fn from(e: anyhow::Error) -> Self {
        Self::Other(e.to_string())
    }
}

pub fn update(model: &mut AppModel, message: Message) -> Result<UpdateResult, UpdateError> {
    match message {
        Message::AddAlias(name, cmd, target, raw) => match target {
            AliasTarget::Global => {
                model.config.add_alias(name, cmd, raw);
                Ok(UpdateResult::effect(Effect::SaveConfig))
            }
            AliasTarget::Local => {
                if let Some(trust) = model.project_trust() {
                    if !trust.is_trusted() {
                        return Err(UpdateError::ProjectNotTrusted {
                            path: trust.path().to_path_buf(),
                        });
                    }
                }
                Ok(UpdateResult::effect(Effect::AddLocalAlias {
                    name,
                    cmd,
                    raw,
                }))
            }
            AliasTarget::ActiveProfile if model.session.active_profiles.is_empty() => {
                if model.project_path().is_some() {
                    if let Some(trust) = model.project_trust() {
                        if !trust.is_trusted() {
                            return Err(UpdateError::ProjectNotTrusted {
                                path: trust.path().to_path_buf(),
                            });
                        }
                    }
                    Ok(UpdateResult::effect(Effect::AddLocalAlias {
                        name,
                        cmd,
                        raw,
                    }))
                } else {
                    model.config.add_alias(name, cmd, raw);
                    Ok(UpdateResult::effect(Effect::SaveConfig))
                }
            }
            target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                let profile = resolve_profile_mut(model, &target)?;
                profile
                    .add_alias(name, cmd, raw)
                    .map_err(|e| UpdateError::Other(e.to_string()))?;
                Ok(UpdateResult::effect(Effect::SaveProfiles))
            }
        },
        Message::RemoveAlias(name, target) => match target {
            AliasTarget::Global => {
                model
                    .config
                    .remove_alias(&name)
                    .map_err(|e| UpdateError::Other(e.to_string()))?;
                Ok(UpdateResult::effect(Effect::SaveConfig))
            }
            AliasTarget::Local => {
                if let Some(trust) = model.project_trust() {
                    if !trust.is_trusted() {
                        return Err(UpdateError::ProjectNotTrusted {
                            path: trust.path().to_path_buf(),
                        });
                    }
                }
                Ok(UpdateResult::effect(Effect::RemoveLocalAlias { name }))
            }
            AliasTarget::ActiveProfile if model.session.active_profiles.is_empty() => {
                if model.project_path().is_some() {
                    if let Some(trust) = model.project_trust() {
                        if !trust.is_trusted() {
                            return Err(UpdateError::ProjectNotTrusted {
                                path: trust.path().to_path_buf(),
                            });
                        }
                    }
                    Ok(UpdateResult::effect(Effect::RemoveLocalAlias { name }))
                } else {
                    model.config.remove_alias(&name)?;
                    Ok(UpdateResult::effect(Effect::SaveConfig))
                }
            }
            target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                let profile = resolve_profile_mut(model, &target)?;
                profile
                    .remove_alias(&name)
                    .map_err(|e| UpdateError::Other(e.to_string()))?;
                Ok(UpdateResult::effect(Effect::SaveProfiles))
            }
        },
        Message::UpdateAlias {
            target,
            old_name,
            new_name,
            new_command,
            raw,
        } => {
            match target {
                AliasTarget::Global => {
                    let key = crate::AliasName::from(old_name.as_str());
                    model.config.aliases.remove(&key).ok_or_else(|| {
                        UpdateError::AliasNotFound {
                            name: old_name.clone(),
                            target: "global".to_string(),
                        }
                    })?;
                    model.config.add_alias(new_name, new_command, raw);
                    Ok(UpdateResult::effect(Effect::SaveConfig))
                }
                AliasTarget::Local => {
                    if let Some(trust) = model.project_trust() {
                        if !trust.is_trusted() {
                            return Err(UpdateError::ProjectNotTrusted {
                                path: trust.path().to_path_buf(),
                            });
                        }
                    }
                    Ok(UpdateResult::with_effects(&[
                        Effect::RemoveLocalAlias { name: old_name },
                        Effect::AddLocalAlias {
                            name: new_name,
                            cmd: new_command,
                            raw,
                        },
                    ]))
                }
                target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                    let profile = resolve_profile_mut(model, &target)?;
                    let key = crate::AliasName::from(old_name.as_str());
                    profile
                        .aliases
                        .remove(&key)
                        .ok_or_else(|| UpdateError::AliasNotFound {
                            name: old_name.clone(),
                            target: format!("{target}"),
                        })?;
                    profile
                        .add_alias(new_name, new_command, raw)
                        .map_err(|e| UpdateError::Other(e.to_string()))?;
                    Ok(UpdateResult::effect(Effect::SaveProfiles))
                }
            }
        }
        Message::AddSubcommandAlias(key, long_subcommands, target) => match target {
            AliasTarget::Global => {
                model.config.add_subcommand(key, long_subcommands);
                Ok(UpdateResult::effect(Effect::SaveConfig))
            }
            AliasTarget::Local => {
                if let Some(trust) = model.project_trust() {
                    if !trust.is_trusted() {
                        return Err(UpdateError::ProjectNotTrusted {
                            path: trust.path().to_path_buf(),
                        });
                    }
                }
                Ok(UpdateResult::effect(Effect::AddLocalSubcommand {
                    key,
                    long_subcommands,
                }))
            }
            AliasTarget::ActiveProfile if model.session.active_profiles.is_empty() => {
                if model.project_path().is_some() {
                    if let Some(trust) = model.project_trust() {
                        if !trust.is_trusted() {
                            return Err(UpdateError::ProjectNotTrusted {
                                path: trust.path().to_path_buf(),
                            });
                        }
                    }
                    Ok(UpdateResult::effect(Effect::AddLocalSubcommand {
                        key,
                        long_subcommands,
                    }))
                } else {
                    model.config.add_subcommand(key, long_subcommands);
                    Ok(UpdateResult::effect(Effect::SaveConfig))
                }
            }
            target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                let profile = resolve_profile_mut(model, &target)?;
                profile.add_subcommand(key, long_subcommands);
                Ok(UpdateResult::effect(Effect::SaveProfiles))
            }
        },
        Message::RemoveSubcommandAlias(key, target) => match target {
            AliasTarget::Global => {
                model.config.remove_subcommand(&key)?;
                Ok(UpdateResult::effect(Effect::SaveConfig))
            }
            AliasTarget::Local => {
                if let Some(trust) = model.project_trust() {
                    if !trust.is_trusted() {
                        return Err(UpdateError::ProjectNotTrusted {
                            path: trust.path().to_path_buf(),
                        });
                    }
                }
                Ok(UpdateResult::effect(Effect::RemoveLocalSubcommand { key }))
            }
            AliasTarget::ActiveProfile if model.session.active_profiles.is_empty() => {
                if model.project_path().is_some() {
                    if let Some(trust) = model.project_trust() {
                        if !trust.is_trusted() {
                            return Err(UpdateError::ProjectNotTrusted {
                                path: trust.path().to_path_buf(),
                            });
                        }
                    }
                    Ok(UpdateResult::effect(Effect::RemoveLocalSubcommand { key }))
                } else {
                    model.config.remove_subcommand(&key)?;
                    Ok(UpdateResult::effect(Effect::SaveConfig))
                }
            }
            target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                let profile = resolve_profile_mut(model, &target)?;
                profile.remove_subcommand(&key)?;
                Ok(UpdateResult::effect(Effect::SaveProfiles))
            }
        },
        Message::UpdateSubcommandAlias {
            original_key,
            new_key,
            long_subcommands,
            target,
        } => match target {
            AliasTarget::Global => {
                model.config.subcommands.remove(&original_key);
                model.config.add_subcommand(new_key, long_subcommands);
                Ok(UpdateResult::effect(Effect::SaveConfig))
            }
            AliasTarget::ActiveProfile if model.session.active_profiles.is_empty() => {
                if model.project_path().is_some() {
                    if let Some(trust) = model.project_trust() {
                        if !trust.is_trusted() {
                            return Err(UpdateError::ProjectNotTrusted {
                                path: trust.path().to_path_buf(),
                            });
                        }
                    }
                    Ok(UpdateResult::new(
                        Message::AddSubcommandAlias(new_key, long_subcommands, AliasTarget::Local),
                        &[Effect::RemoveLocalSubcommand { key: original_key }],
                    ))
                } else {
                    model.config.subcommands.remove(&original_key);
                    model.config.add_subcommand(new_key, long_subcommands);
                    Ok(UpdateResult::effect(Effect::SaveConfig))
                }
            }
            AliasTarget::Local => {
                if let Some(trust) = model.project_trust() {
                    if !trust.is_trusted() {
                        return Err(UpdateError::ProjectNotTrusted {
                            path: trust.path().to_path_buf(),
                        });
                    }
                }
                Ok(UpdateResult::new(
                    Message::AddSubcommandAlias(new_key, long_subcommands, AliasTarget::Local),
                    &[Effect::RemoveLocalSubcommand { key: original_key }],
                ))
            }
            target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                let profile = resolve_profile_mut(model, &target)?;
                profile.subcommands.remove(&original_key);
                profile.add_subcommand(new_key, long_subcommands);
                Ok(UpdateResult::effect(Effect::SaveProfiles))
            }
        },
        Message::CopySubcommandAliases { keys, from, to } => {
            let pairs: Vec<(String, Vec<String>)> = {
                let src_subcommands = match &from {
                    AliasTarget::Global => Some(&model.config.subcommands),
                    AliasTarget::Local => model.project_aliases().map(|p| &p.subcommands),
                    AliasTarget::ActiveProfile | AliasTarget::Profile(_) => {
                        Some(&resolve_profile(model, &from)?.subcommands)
                    }
                };
                match src_subcommands {
                    Some(subs) => keys
                        .iter()
                        .filter_map(|k| Some((k.clone(), subs.get(k)?.clone())))
                        .collect(),
                    None => vec![],
                }
            };

            if matches!(to, AliasTarget::Local) {
                if let Some(trust) = model.project_trust() {
                    if !trust.is_trusted() {
                        return Err(UpdateError::ProjectNotTrusted {
                            path: trust.path().to_path_buf(),
                        });
                    }
                }
            }

            match to {
                AliasTarget::Global => {
                    for (key, longs) in pairs {
                        model.config.add_subcommand(key, longs);
                    }
                    Ok(UpdateResult::effect(Effect::SaveConfig))
                }
                AliasTarget::Local => {
                    let effects: Vec<Effect> = pairs
                        .into_iter()
                        .map(|(key, long_subcommands)| Effect::AddLocalSubcommand {
                            key,
                            long_subcommands,
                        })
                        .collect();
                    Ok(UpdateResult::with_effects(&effects))
                }
                target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                    let profile = resolve_profile_mut(model, &target)?;
                    for (key, longs) in pairs {
                        profile.add_subcommand(key, longs);
                    }
                    Ok(UpdateResult::effect(Effect::SaveProfiles))
                }
            }
        }
        Message::MoveSubcommandAliases { keys, from, to } => {
            let pairs: Vec<(String, Vec<String>)> = {
                let src_subcommands = match &from {
                    AliasTarget::Global => Some(&model.config.subcommands),
                    AliasTarget::Local => model.project_aliases().map(|p| &p.subcommands),
                    AliasTarget::ActiveProfile | AliasTarget::Profile(_) => {
                        Some(&resolve_profile(model, &from)?.subcommands)
                    }
                };
                match src_subcommands {
                    Some(subs) => keys
                        .iter()
                        .filter_map(|k| Some((k.clone(), subs.get(k)?.clone())))
                        .collect(),
                    None => vec![],
                }
            };

            let need_local_src = matches!(from, AliasTarget::Local);
            let need_local_dst = matches!(to, AliasTarget::Local);
            // Pre-collect remove effects for Local source before pairs are moved into the
            // destination arms. Local→Local is excluded: we'd be removing what we're adding.
            let remove_local_effects: Vec<Effect> = if need_local_src && !need_local_dst {
                pairs
                    .iter()
                    .map(|(key, _)| Effect::RemoveLocalSubcommand { key: key.clone() })
                    .collect()
            } else {
                vec![]
            };
            if need_local_src || need_local_dst {
                if let Some(trust) = model.project_trust() {
                    if !trust.is_trusted() {
                        return Err(UpdateError::ProjectNotTrusted {
                            path: trust.path().to_path_buf(),
                        });
                    }
                }
            }

            // Remove from source
            match &from {
                AliasTarget::Global => {
                    for (key, _) in &pairs {
                        model.config.subcommands.remove(key);
                    }
                }
                AliasTarget::Local => {} // handled via effects below
                AliasTarget::Profile(name) => {
                    if let Some(profile) = model.profile_config_mut().get_profile_by_name_mut(name)
                    {
                        for (key, _) in &pairs {
                            profile.subcommands.remove(key);
                        }
                    }
                }
                AliasTarget::ActiveProfile => {
                    let active: Vec<String> = model.session.active_profiles.clone();
                    for name in &active {
                        if let Some(profile) =
                            model.profile_config_mut().get_profile_by_name_mut(name)
                        {
                            for (key, _) in &pairs {
                                profile.subcommands.remove(key);
                            }
                        }
                    }
                }
            }

            // Add to destination
            match to {
                AliasTarget::Global => {
                    for (key, longs) in pairs {
                        model.config.add_subcommand(key, longs);
                    }
                    let needs_profiles =
                        matches!(from, AliasTarget::Profile(_) | AliasTarget::ActiveProfile);
                    let mut effects = remove_local_effects;
                    if needs_profiles {
                        effects.extend([Effect::SaveConfig, Effect::SaveProfiles]);
                    } else {
                        effects.push(Effect::SaveConfig);
                    }
                    Ok(UpdateResult::with_effects(&effects))
                }
                AliasTarget::Local => {
                    let mut effects: Vec<Effect> = pairs
                        .into_iter()
                        .map(|(key, long_subcommands)| Effect::AddLocalSubcommand {
                            key,
                            long_subcommands,
                        })
                        .collect();
                    if matches!(from, AliasTarget::Profile(_) | AliasTarget::ActiveProfile) {
                        effects.push(Effect::SaveProfiles);
                    } else if matches!(from, AliasTarget::Global) {
                        effects.push(Effect::SaveConfig);
                    }
                    Ok(UpdateResult::with_effects(&effects))
                }
                target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                    let profile = resolve_profile_mut(model, &target)?;
                    for (key, longs) in pairs {
                        profile.add_subcommand(key, longs);
                    }
                    let needs_config = matches!(from, AliasTarget::Global);
                    let mut effects = remove_local_effects;
                    if needs_config {
                        effects.extend([Effect::SaveProfiles, Effect::SaveConfig]);
                    } else {
                        effects.push(Effect::SaveProfiles);
                    }
                    Ok(UpdateResult::with_effects(&effects))
                }
            }
        }
        Message::ListProfiles { used } => {
            let output = render_listing(
                &model.config.aliases,
                &model.config.subcommands,
                model.profile_config(),
                &model.session.active_profiles,
                model.project_trust(),
                {
                    if used {
                        Some(AliasDisplayFilter::Used)
                    } else {
                        None
                    }
                },
            );
            println!("{output}");
            Ok(UpdateResult::done())
        }
        Message::CreateProfile(name) => match model
            .profile_config_mut()
            .add_profile(&name)
            .map_err(|e| UpdateError::Other(e.to_string()))?
        {
            profile::Response::ProfileAdded(_) => {
                if !model.session.active_profiles.contains(&name) {
                    model.session.active_profiles.push(name);
                }
                Ok(UpdateResult::effect(Effect::SaveProfiles))
            }
            profile::Response::ProfileActivated(_) => {
                if !model.session.active_profiles.contains(&name) {
                    model.session.active_profiles.push(name);
                }
                Ok(UpdateResult::done())
            }
        },
        Message::InitShell(shell, force) => {
            let resolved = model
                .profile_config()
                .resolve_active_aliases(&model.session.active_profiles);
            let resolved_subs = model
                .profile_config()
                .resolve_active_subcommands(&model.session.active_profiles);
            let mut all_subs = model.config.subcommands.clone();
            for (k, v) in resolved_subs {
                all_subs.insert(k, v);
            }
            let (external_functions, external_aliases) = match shell {
                Shell::Zsh => (zsh::scan_external_functions(), zsh::scan_external_aliases()),
                Shell::Bash => (
                    bash::scan_external_functions(),
                    bash::scan_external_aliases(),
                ),
                _ => Default::default(),
            };
            let ctx = ShellContext {
                shell: &shell,
                cfg: &model.config.shell,
                cwd: &model.cwd,
                external_functions,
                external_aliases,
            };

            let mut output = String::new();

            if force {
                let shell_impl = shell.clone().as_shell(
                    &model.config.shell,
                    Default::default(),
                    Default::default(),
                );
                let prev_global = std::env::var(env_vars::AM_ALIASES).unwrap_or_default();
                let prev_project = std::env::var(env_vars::AM_PROJECT_ALIASES).unwrap_or_default();
                let all_prev: Vec<&str> = prev_global
                    .split(',')
                    .chain(prev_project.split(','))
                    .filter(|s| !s.is_empty())
                    // Strip |hash suffix from project aliases (name|hash format)
                    .map(|s| s.split_once('|').map_or(s, |(name, _)| name))
                    .collect();
                for name in all_prev {
                    output.push_str(&shell_impl.force_unalias(name));
                    output.push('\n');
                }
                // Clear project-alias tracking so __am_hook reloads them fresh
                // instead of assuming they're still loaded.
                output.push_str(&shell_impl.unset_env(env_vars::AM_PROJECT_ALIASES));
                output.push('\n');
                output.push_str(&shell_impl.unset_env(env_vars::AM_PROJECT_PATH));
                output.push('\n');
            }

            output.push_str(&generate_init(
                &ctx,
                &model.config.aliases,
                &resolved,
                &all_subs,
            ));
            print!("{output}");
            Ok(UpdateResult::done())
        }
        Message::Reload(shell) => {
            let resolved = model
                .profile_config()
                .resolve_active_aliases(&model.session.active_profiles);
            let resolved_subs = model
                .profile_config()
                .resolve_active_subcommands(&model.session.active_profiles);
            let mut all_subs = model.config.subcommands.clone();
            for (k, v) in resolved_subs {
                all_subs.insert(k, v);
            }
            let prev = std::env::var(env_vars::AM_ALIASES).ok();
            let ctx = ShellContext {
                shell: &shell,
                cfg: &model.config.shell,
                cwd: &model.cwd,
                external_functions: Default::default(),
                external_aliases: Default::default(),
            };
            let output = generate_reload(
                &ctx,
                &model.config.aliases,
                &resolved,
                &all_subs,
                prev.as_deref(),
            );
            if !output.is_empty() {
                print!("{output}");
            }
            Ok(UpdateResult::done())
        }
        Message::Hook(shell, quiet) => {
            let prev = std::env::var(env_vars::AM_PROJECT_ALIASES).ok();
            let prev_project_path = std::env::var(env_vars::AM_PROJECT_PATH).ok();
            let shell_cfg = model.config.shell.clone();
            let cwd = model.cwd.clone();
            let ctx = ShellContext {
                shell: &shell,
                cfg: &shell_cfg,
                cwd: &cwd,
                external_functions: Default::default(),
                external_aliases: Default::default(),
            };

            // Resolve global + active profile aliases for shadow restoration
            let resolved_profile = model
                .profile_config()
                .resolve_active_aliases(&model.session.active_profiles);
            let mut all_profile_aliases = model.config.aliases.clone();
            for (name, alias) in resolved_profile.iter() {
                all_profile_aliases.insert(name.clone(), alias.clone());
            }

            let (output, security_changed) = crate::hook::generate_hook_with_security(
                &ctx,
                prev.as_deref(),
                prev_project_path.as_deref(),
                model.security_config_mut(),
                quiet,
                &all_profile_aliases,
            )
            .map_err(|e| UpdateError::Other(e.to_string()))?;
            if !output.is_empty() {
                print!("{output}");
            }
            if security_changed {
                Ok(UpdateResult::effect(Effect::SaveSecurity))
            } else {
                Ok(UpdateResult::done())
            }
        }
        Message::Sync(shell, quiet) => {
            let prev_aliases = std::env::var(env_vars::AM_ALIASES).ok();
            let prev_subs = std::env::var(env_vars::AM_SUBCOMMANDS).ok();
            let legacy_project = std::env::var(env_vars::AM_PROJECT_ALIASES).ok();
            let merged_prev_aliases = match (prev_aliases.as_deref(), legacy_project.as_deref()) {
                (None, None) => None,
                (Some(a), None) => Some(a.to_string()),
                (None, Some(b)) => Some(b.to_string()),
                (Some(a), Some(b)) => Some(format!("{a},{b}")),
            };
            let prev_project_path = std::env::var(env_vars::AM_PROJECT_PATH).ok();

            let shell_cfg = model.config.shell.clone();
            let cwd = model.cwd.clone();
            let shell_impl = shell
                .clone()
                .as_shell(&shell_cfg, Default::default(), Default::default());

            let resolved_aliases = model
                .profile_config()
                .resolve_active_aliases(&model.session.active_profiles);
            let resolved_subs = model
                .profile_config()
                .resolve_active_subcommands(&model.session.active_profiles);

            // Decide project inclusion and evaluate trust warnings.
            let mut lines: Vec<String> = Vec::new();
            let mut security_changed = false;
            let (include_project, project_path) = match model.project_trust() {
                Some(crate::trust::ProjectTrust::Trusted(..)) => {
                    (true, model.project_path().map(|p| p.to_path_buf()))
                }
                Some(trust) => {
                    let path = trust.path().to_path_buf();
                    let is_direct = path.parent().is_some_and(|p| p == cwd);
                    let already_seen = prev_project_path
                        .as_deref()
                        .is_some_and(|p| std::path::Path::new(p) == path);
                    let show_msg = !quiet && is_direct && !already_seen;
                    match trust {
                        crate::trust::ProjectTrust::Unknown(_) if show_msg => {
                            lines.push(shell_impl.echo(
                                "am: .aliases found but not trusted. Run 'am trust' to review and allow.",
                            ));
                        }
                        crate::trust::ProjectTrust::Tampered(_) => {
                            security_changed = true;
                            if show_msg {
                                lines.push(shell_impl.echo(
                                    "am: .aliases was modified since last trusted. Run 'am trust' to review and allow.",
                                ));
                            }
                        }
                        _ => {}
                    }
                    (false, Some(path))
                }
                None => (false, None),
            };

            let (project_aliases, project_subs) = if include_project {
                model.project_alias_set_and_subcommands()
            } else {
                (crate::AliasSet::default(), crate::subcommand::SubcommandSet::new())
            };

            let is_fresh_load = merged_prev_aliases.as_deref().is_none_or(|s| s.is_empty())
                && prev_subs.as_deref().is_none_or(|s| s.is_empty());

            let diff = Precedence::new()
                .with_global(&model.config.aliases, &model.config.subcommands)
                .with_profiles(&resolved_aliases, &resolved_subs)
                .with_project(&project_aliases, &project_subs)
                .with_shell_state_from_env(merged_prev_aliases.as_deref(), prev_subs.as_deref())
                .resolve();

            // ── Human-readable messaging ────────────────────────
            if !quiet {
                if is_fresh_load && include_project {
                    if let Some(path) = project_path.as_deref() {
                        if let Ok(project) = crate::project::ProjectAliases::load(path) {
                            for line in crate::trust::render_load_message(
                                &project.aliases,
                                &project.subcommands,
                            )
                            .lines()
                            {
                                lines.push(shell_impl.echo(line));
                            }
                        }
                    }
                } else if !is_fresh_load
                    && (!diff.added.is_empty()
                        || !diff.changed.is_empty()
                        || !diff.removed.is_empty())
                {
                    let mut parts = Vec::new();
                    if !diff.added.is_empty() {
                        parts.push(format!("{} added", diff.added.len()));
                    }
                    if !diff.changed.is_empty() {
                        parts.push(format!("{} updated", diff.changed.len()));
                    }
                    if !diff.removed.is_empty() {
                        parts.push(format!("{} removed", diff.removed.len()));
                    }
                    if !parts.is_empty() {
                        lines.push(shell_impl.echo(&format!(
                            "am: aliases changed ({})",
                            parts.join(", ")
                        )));
                    }
                }
            }

            let rendered = precedence::render_diff(&diff, shell_impl.as_ref());
            if !rendered.is_empty() {
                lines.push(rendered);
            }

            // ── _AM_PROJECT_PATH bookkeeping ───────────────────
            if include_project {
                if prev_project_path.is_some() {
                    lines.push(shell_impl.unset_env(env_vars::AM_PROJECT_PATH));
                }
            } else if let Some(p) = project_path.as_deref() {
                lines.push(shell_impl.set_env(
                    env_vars::AM_PROJECT_PATH,
                    &p.display().to_string(),
                ));
            } else if prev_project_path.is_some() {
                lines.push(shell_impl.unset_env(env_vars::AM_PROJECT_PATH));
            }

            // ── Legacy env var cleanup on first sync after upgrade ──
            if legacy_project.is_some() {
                lines.push(shell_impl.unset_env(env_vars::AM_PROJECT_ALIASES));
            }

            let joined = lines
                .into_iter()
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            if !joined.is_empty() {
                print!("{joined}");
            }

            if security_changed {
                Ok(UpdateResult::effect(Effect::SaveSecurity))
            } else {
                Ok(UpdateResult::done())
            }
        }
        Message::ToggleProfiles(names) => {
            for name in &names {
                model
                    .profile_config()
                    .get_profile_by_name(name)
                    .ok_or_else(|| UpdateError::ProfileNotFound { name: name.clone() })?;
            }
            let mut effects = Vec::new();
            for name in names {
                let was_active = model.session.is_active(&name);
                let (total, list) = model
                    .profile_config()
                    .get_profile_by_name(&name)
                    .map(|p| (p.len(), p.short_list()))
                    .unwrap_or((0, String::new()));
                model.session.toggle_profile(name.clone());
                let action = if was_active {
                    "deactivated"
                } else {
                    "activated"
                };
                let msg = if was_active || list.is_empty() {
                    format!("{name} {action}, {total} aliases")
                } else {
                    format!("{name} {action} — {total} loaded: {list}")
                };
                effects.push(Effect::Print(msg));
            }
            effects.push(Effect::SaveSession);
            Ok(UpdateResult::with_effects(&effects))
        }
        Message::UseProfilesAt(names, priority) => {
            for name in &names {
                model
                    .profile_config()
                    .get_profile_by_name(name)
                    .ok_or_else(|| UpdateError::ProfileNotFound { name: name.clone() })?;
            }
            let mut effects = Vec::new();
            for (i, name) in names.into_iter().enumerate() {
                let (total, list) = model
                    .profile_config()
                    .get_profile_by_name(&name)
                    .map(|p| (p.len(), p.short_list()))
                    .unwrap_or((0, String::new()));
                model.session.use_profile_at(name.clone(), priority + i);
                let pos = priority + i;
                let msg = if list.is_empty() {
                    format!("{name} activated at position {pos}, {total} aliases loaded")
                } else {
                    format!("{name} activated at position {pos} — {total} loaded: {list}")
                };
                effects.push(Effect::Print(msg));
            }
            effects.push(Effect::SaveSession);
            Ok(UpdateResult::with_effects(&effects))
        }
        Message::RemoveProfile(name) => {
            model
                .profile_config_mut()
                .remove_profile(&name)
                .map_err(|e| UpdateError::Other(e.to_string()))?;
            model.session.active_profiles.retain(|p| p != &name);
            Ok(UpdateResult::with_effects(&[
                Effect::SaveProfiles,
                Effect::SaveSession,
            ]))
        }
        Message::Import(payload) => {
            let mut effects = Vec::new();
            if let Some(aliases) = payload.global_aliases {
                model.config.merge_aliases(aliases);
                effects.push(Effect::SaveConfig);
            }
            if let Some(subcommands) = payload.global_subcommands {
                for (key, longs) in subcommands {
                    model.config.subcommands.insert(key, longs);
                }
                effects.push(Effect::SaveConfig);
            }
            for profile in payload.profiles {
                model.profile_config_mut().merge_profile(profile);
                if !effects.contains(&Effect::SaveProfiles) {
                    effects.push(Effect::SaveProfiles);
                }
            }
            // local_aliases and local_subcommands are saved by the CLI layer (needs file path)
            Ok(UpdateResult::with_effects(&effects))
        }
        Message::Trust => {
            let path = model
                .project_trust()
                .map(|t| t.path().to_path_buf())
                .ok_or(UpdateError::NoProjectFile)?;

            if model
                .project_trust()
                .map(|t| t.is_trusted())
                .unwrap_or(false)
            {
                return Ok(UpdateResult::effect(Effect::Print(format!(
                    "Already trusted: {}",
                    path.display()
                ))));
            }

            let hash = crate::trust::compute_file_hash(&path)
                .map_err(|e| UpdateError::Other(e.to_string()))?;
            model.security_config_mut().trust(&path, &hash);

            // Reload project aliases now that it's trusted
            let aliases =
                ProjectAliases::load(&path).map_err(|e| UpdateError::Other(e.to_string()))?;
            model.project_trust = Some(ProjectTrust::Trusted(aliases, path));

            Ok(UpdateResult::effect(Effect::SaveSecurity))
        }
        Message::Untrust { forget } => {
            let path = model
                .project_trust()
                .map(|t| t.path().to_path_buf())
                .ok_or(UpdateError::NoProjectFile)?;

            if forget {
                model.security_config_mut().forget(&path);
                model.project_trust = Some(ProjectTrust::Unknown(path.clone()));
                Ok(UpdateResult::with_effects(&[
                    Effect::Print(format!("Removed {} from security tracking", path.display())),
                    Effect::SaveSecurity,
                ]))
            } else {
                model.security_config_mut().untrust(&path);
                model.project_trust = Some(ProjectTrust::Untrusted(path.clone()));
                Ok(UpdateResult::with_effects(&[
                    Effect::Print(format!("Untrusted: {}", path.display())),
                    Effect::SaveSecurity,
                ]))
            }
        }
        Message::MoveAliases { aliases, to } => transfer_aliases(model, &aliases, &to, true),
        Message::CopyAliases { aliases, to } => transfer_aliases(model, &aliases, &to, false),
        Message::RenameProfile { old_name, new_name } => {
            let profile = model
                .profile_config_mut()
                .get_profile_by_name_mut(&old_name)
                .ok_or_else(|| UpdateError::ProfileNotFound {
                    name: old_name.clone(),
                })?;
            profile.name = new_name.clone();
            for p in &mut model.session.active_profiles {
                if *p == old_name {
                    *p = new_name.clone();
                }
            }
            Ok(UpdateResult::with_effects(&[
                Effect::SaveProfiles,
                Effect::SaveSession,
            ]))
        }
    }
}

fn resolve_profile<'a>(
    model: &'a AppModel,
    target: &AliasTarget,
) -> Result<&'a Profile, UpdateError> {
    match target {
        AliasTarget::Profile(profile_name) => model
            .profile_config()
            .get_profile_by_name(profile_name)
            .ok_or_else(|| UpdateError::ProfileNotFound {
                name: profile_name.clone(),
            }),
        AliasTarget::ActiveProfile => {
            let active = model
                .session
                .active_profiles
                .last()
                .cloned()
                .ok_or_else(|| UpdateError::Other("No active profile".into()))?;
            model
                .profile_config()
                .get_profile_by_name(&active)
                .ok_or(UpdateError::ProfileNotFound { name: active })
        }
        _ => unreachable!(),
    }
}

fn resolve_profile_mut<'a>(
    model: &'a mut AppModel,
    target: &AliasTarget,
) -> Result<&'a mut Profile, UpdateError> {
    match target {
        AliasTarget::Profile(profile_name) => model
            .profile_config_mut()
            .get_profile_by_name_mut(profile_name)
            .ok_or_else(|| UpdateError::ProfileNotFound {
                name: profile_name.clone(),
            }),
        AliasTarget::ActiveProfile => {
            let active = model
                .session
                .active_profiles
                .last()
                .cloned()
                .ok_or_else(|| UpdateError::Other("No active profile".into()))?;
            model
                .profile_config_mut()
                .get_profile_by_name_mut(&active)
                .ok_or(UpdateError::ProfileNotFound { name: active })
        }
        _ => unreachable!(),
    }
}

fn transfer_aliases(
    model: &mut AppModel,
    aliases: &[crate::AliasId],
    to: &AliasTarget,
    delete_source: bool,
) -> Result<UpdateResult, UpdateError> {
    use crate::AliasId;

    // Trust-gate the destination if it's Local
    if *to == AliasTarget::Local {
        if let Some(trust) = model.project_trust() {
            if !trust.is_trusted() {
                return Err(UpdateError::ProjectNotTrusted {
                    path: trust.path().to_path_buf(),
                });
            }
        }
    }

    let mut effects: Vec<Effect> = Vec::new();
    let mut needs_save_config = false;
    let mut needs_save_profiles = false;

    for id in aliases {
        // Read alias from source
        let (cmd, raw) =
            read_alias_from_model(model, id).ok_or_else(|| UpdateError::AliasNotFound {
                name: id.name().to_string(),
                target: format!("{}", id.target()),
            })?;

        // Remove from source (for move)
        if delete_source {
            match id {
                AliasId::Global { alias_name } => {
                    let key = crate::AliasName::from(alias_name.as_str());
                    model.config.aliases.remove(&key);
                    needs_save_config = true;
                }
                AliasId::Profile {
                    profile_name,
                    alias_name,
                } => {
                    if let Some(p) = model
                        .profile_config_mut()
                        .get_profile_by_name_mut(profile_name)
                    {
                        let key = crate::AliasName::from(alias_name.as_str());
                        p.aliases.remove(&key);
                        needs_save_profiles = true;
                    }
                }
                AliasId::Project { alias_name } => {
                    effects.push(Effect::RemoveLocalAlias {
                        name: alias_name.clone(),
                    });
                }
                AliasId::Subcommand { .. } => {
                    // Subcommand aliases are handled by CopySubcommandAliases / MoveSubcommandAliases
                    unreachable!("subcommand aliases must not flow through copy_or_move_aliases");
                }
            }
        }

        // Add to destination
        let name = id.name().to_string();
        match to {
            AliasTarget::Global => {
                model.config.add_alias(name, cmd, raw);
                needs_save_config = true;
            }
            AliasTarget::Local => {
                effects.push(Effect::AddLocalAlias { name, cmd, raw });
            }
            target @ (AliasTarget::Profile(_) | AliasTarget::ActiveProfile) => {
                let profile = resolve_profile_mut(model, target)?;
                profile
                    .add_alias(name, cmd, raw)
                    .map_err(|e| UpdateError::Other(e.to_string()))?;
                needs_save_profiles = true;
            }
        }
    }

    if needs_save_config {
        effects.push(Effect::SaveConfig);
    }
    if needs_save_profiles {
        effects.push(Effect::SaveProfiles);
    }

    Ok(UpdateResult::with_effects(&effects))
}

fn read_alias_from_model(model: &AppModel, id: &crate::AliasId) -> Option<(String, bool)> {
    use crate::AliasId;
    match id {
        AliasId::Global { alias_name } => {
            let key = crate::AliasName::from(alias_name.as_str());
            model.config.aliases.get(&key).map(|a| {
                let raw = matches!(a, crate::TomlAlias::Detailed(d) if d.raw);
                (a.command().to_string(), raw)
            })
        }
        AliasId::Profile {
            profile_name,
            alias_name,
        } => {
            let key = crate::AliasName::from(alias_name.as_str());
            model
                .profile_config()
                .get_profile_by_name(profile_name)
                .and_then(|p| p.aliases.get(&key))
                .map(|a| {
                    let raw = matches!(a, crate::TomlAlias::Detailed(d) if d.raw);
                    (a.command().to_string(), raw)
                })
        }
        AliasId::Project { alias_name } => {
            let key = crate::AliasName::from(alias_name.as_str());
            model
                .project_aliases()
                .and_then(|p| p.aliases.get(&key))
                .map(|a| {
                    let raw = matches!(a, crate::TomlAlias::Detailed(d) if d.raw);
                    (a.command().to_string(), raw)
                })
        }
        AliasId::Subcommand { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::effects::Effect;
    use crate::security::SecurityConfig;
    use crate::ProfileConfig;

    #[test]
    fn update_subcommand_alias_replaces_key() {
        let mut model = AppModel::new(Config::default(), ProfileConfig::default());
        model
            .config
            .subcommands
            .insert("jj:ab".into(), vec!["abandon".into()]);
        let result = update(
            &mut model,
            Message::UpdateSubcommandAlias {
                original_key: "jj:ab".into(),
                new_key: "jj:a".into(),
                long_subcommands: vec!["abandon".into()],
                target: AliasTarget::Global,
            },
        )
        .unwrap();
        assert!(!model.config.subcommands.contains_key("jj:ab"));
        assert_eq!(
            model.config.subcommands.get("jj:a"),
            Some(&vec!["abandon".to_string()])
        );
        assert!(result
            .effects
            .iter()
            .any(|e| matches!(e, Effect::SaveConfig)));
    }

    #[test]
    fn copy_subcommand_aliases_adds_to_destination() {
        let mut model = AppModel::new(Config::default(), ProfileConfig::default());
        model
            .config
            .subcommands
            .insert("jj:ab".into(), vec!["abandon".into()]);
        model.profile_config_mut().add_profile("rust").unwrap();
        let _ = update(
            &mut model,
            Message::CopySubcommandAliases {
                keys: vec!["jj:ab".into()],
                from: AliasTarget::Global,
                to: AliasTarget::Profile("rust".into()),
            },
        )
        .unwrap();
        let profile = model.profile_config().get_profile_by_name("rust").unwrap();
        assert_eq!(
            profile.subcommands.get("jj:ab"),
            Some(&vec!["abandon".to_string()])
        );
        // Source preserved
        assert!(model.config.subcommands.contains_key("jj:ab"));
    }

    #[test]
    fn move_subcommand_aliases_removes_from_source() {
        let mut model = AppModel::new(Config::default(), ProfileConfig::default());
        model
            .config
            .subcommands
            .insert("jj:ab".into(), vec!["abandon".into()]);
        model.profile_config_mut().add_profile("rust").unwrap();
        let _ = update(
            &mut model,
            Message::MoveSubcommandAliases {
                keys: vec!["jj:ab".into()],
                from: AliasTarget::Global,
                to: AliasTarget::Profile("rust".into()),
            },
        )
        .unwrap();
        assert!(!model.config.subcommands.contains_key("jj:ab"));
        let profile = model.profile_config().get_profile_by_name("rust").unwrap();
        assert!(profile.subcommands.contains_key("jj:ab"));
    }

    #[test]
    fn update_result_done_has_no_message_or_effects() {
        let r = UpdateResult::done();
        assert!(r.next.is_none());
        assert!(r.effects.is_empty());
    }

    #[test]
    fn update_result_message_has_message_and_no_effects() {
        let r = UpdateResult::message(Message::ListProfiles { used: false });
        assert!(r.next.is_some());
        assert!(r.effects.is_empty());
    }

    #[test]
    fn update_result_effect_has_one_effect_and_no_message() {
        let r = UpdateResult::effect(Effect::SaveConfig);
        assert!(r.next.is_none());
        assert_eq!(r.effects.len(), 1);
    }

    #[test]
    fn update_result_new_has_message_and_effects() {
        let r = UpdateResult::new(
            Message::ListProfiles { used: false },
            &[Effect::SaveConfig, Effect::SaveProfiles],
        );
        assert!(r.next.is_some());
        assert_eq!(r.effects.len(), 2);
    }

    #[test]
    fn update_result_with_effects_has_effects_and_no_message() {
        let r = UpdateResult::with_effects(&[Effect::SaveConfig, Effect::SaveProfiles]);
        assert!(r.next.is_none());
        assert_eq!(r.effects.len(), 2);
    }

    #[test]
    fn add_global_alias_mutates_config_and_returns_save_config() {
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::AddAlias("ll".into(), "ls -lha".into(), AliasTarget::Global, false),
        )
        .unwrap();

        assert_eq!(model.config.aliases.iter().count(), 1);
        assert_eq!(result.effects, vec![Effect::SaveConfig]);
        assert!(result.next.is_none());
    }

    #[test]
    fn add_alias_active_profile_no_profiles_no_local_falls_back_to_global() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config).with_cwd(dir.path().to_path_buf());

        let result = update(
            &mut model,
            Message::AddAlias(
                "ll".into(),
                "ls -lha".into(),
                AliasTarget::ActiveProfile,
                false,
            ),
        )
        .unwrap();

        assert_eq!(model.config.aliases.iter().count(), 1);
        assert_eq!(result.effects, vec![Effect::SaveConfig]);
    }

    #[test]
    fn remove_global_alias_mutates_config_and_returns_save_config() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::RemoveAlias("ll".into(), AliasTarget::Global),
        )
        .unwrap();

        assert!(model.config.aliases.is_empty());
        assert_eq!(result.effects, vec![Effect::SaveConfig]);
    }

    #[test]
    fn remove_alias_active_profile_no_profiles_no_local_falls_back_to_global() {
        let dir = tempfile::tempdir().unwrap();
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config).with_cwd(dir.path().to_path_buf());

        let result = update(
            &mut model,
            Message::RemoveAlias("ll".into(), AliasTarget::ActiveProfile),
        )
        .unwrap();

        assert!(model.config.aliases.is_empty());
        assert_eq!(result.effects, vec![Effect::SaveConfig]);
    }

    #[test]
    fn add_local_alias_returns_add_local_effect() {
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::AddAlias("t".into(), "cargo test".into(), AliasTarget::Local, false),
        )
        .unwrap();

        assert_eq!(
            result.effects,
            vec![Effect::AddLocalAlias {
                name: "t".into(),
                cmd: "cargo test".into(),
                raw: false,
            }]
        );
        assert!(model.config.aliases.is_empty());
    }

    #[test]
    fn remove_local_alias_returns_remove_local_effect() {
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::RemoveAlias("t".into(), AliasTarget::Local),
        )
        .unwrap();

        assert_eq!(
            result.effects,
            vec![Effect::RemoveLocalAlias { name: "t".into() }]
        );
    }

    #[test]
    fn toggle_multiple_profiles_activates_all_in_order() {
        let config = Config::default();
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"git\"\n\n[[profiles]]\nname = \"rust\"\n")
                .unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::ToggleProfiles(vec!["git".into(), "rust".into()]),
        )
        .unwrap();

        assert_eq!(result.effects.len(), 3);
        assert!(
            matches!(&result.effects[0], Effect::Print(s) if s.contains("git") && s.contains("activated"))
        );
        assert!(matches!(&result.effects[2], Effect::SaveSession));
        assert_eq!(
            model.session.active_profiles,
            vec!["git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn toggle_multiple_profiles_fails_if_any_missing() {
        let config = Config::default();
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"git\"\n").unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::ToggleProfiles(vec!["git".into(), "nope".into()]),
        );

        assert!(matches!(result, Err(UpdateError::ProfileNotFound { name }) if name == "nope"));
        // git should NOT have been toggled since validation failed
        assert!(model.session.active_profiles.is_empty());
    }

    #[test]
    fn use_profiles_at_inserts_sequentially() {
        let config = Config::default();
        let profile_config: ProfileConfig = toml::from_str(
            "[[profiles]]\nname = \"git\"\n\n[[profiles]]\nname = \"rust\"\n\n[[profiles]]\nname = \"node\"\n",
        )
        .unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::UseProfilesAt(vec!["git".into(), "rust".into()], 1),
        )
        .unwrap();

        assert_eq!(result.effects.len(), 3);
        assert!(
            matches!(&result.effects[0], Effect::Print(s) if s.contains("git") && s.contains("position 1"))
        );
        assert!(
            matches!(&result.effects[1], Effect::Print(s) if s.contains("rust") && s.contains("position 2"))
        );
        assert!(matches!(&result.effects[2], Effect::SaveSession));
        assert_eq!(
            model.session.active_profiles,
            vec!["git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn use_profiles_at_inserts_at_offset() {
        let profile_config: ProfileConfig = toml::from_str(
            "[[profiles]]\nname = \"git\"\n\n[[profiles]]\nname = \"rust\"\n\n[[profiles]]\nname = \"node\"\n",
        )
        .unwrap();
        let mut model = AppModel::new(Config::default(), profile_config);
        model.session.active_profiles = vec!["node".to_string()];

        let result = update(
            &mut model,
            Message::UseProfilesAt(vec!["git".into(), "rust".into()], 2),
        )
        .unwrap();

        assert_eq!(result.effects.len(), 3);
        assert!(matches!(&result.effects[2], Effect::SaveSession));
        // node at 1, git at 2, rust at 3
        assert_eq!(
            model.session.active_profiles,
            vec!["node".to_string(), "git".to_string(), "rust".to_string()]
        );
    }

    #[test]
    fn use_profiles_at_fails_if_any_missing() {
        let config = Config::default();
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"git\"\n").unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::UseProfilesAt(vec!["git".into(), "nope".into()], 1),
        );

        assert!(result.is_err());
        assert!(model.session.active_profiles.is_empty());
    }

    #[test]
    fn add_alias_to_named_profile() {
        let config = Config::default();
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"rust\"\n").unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::AddAlias(
                "ct".into(),
                "cargo test".into(),
                AliasTarget::Profile("rust".into()),
                false,
            ),
        )
        .unwrap();

        assert_eq!(result.effects, vec![Effect::SaveProfiles]);
        let profile = model.profile_config().get_profile_by_name("rust").unwrap();
        assert_eq!(profile.aliases.iter().count(), 1);
    }

    #[test]
    fn model_with_trusted_project_returns_trusted_variant() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nb = \"make build\"\n",
        )
        .unwrap();

        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut security = SecurityConfig::default();
        let hash = crate::trust::compute_file_hash(&dir.path().join(".aliases")).unwrap();
        security.trust(&dir.path().join(".aliases"), &hash);

        let model = AppModel::new_with_security(config, profile_config, security)
            .with_cwd(dir.path().to_path_buf());
        assert!(model.project_trust().is_some());
        assert!(model.project_trust().unwrap().is_trusted());
        assert!(model.project_aliases().is_some());
    }

    #[test]
    fn model_with_unknown_project_returns_unknown_variant() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nb = \"make build\"\n",
        )
        .unwrap();

        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let security = SecurityConfig::default();

        let model = AppModel::new_with_security(config, profile_config, security)
            .with_cwd(dir.path().to_path_buf());
        assert!(model.project_trust().is_some());
        assert!(matches!(
            model.project_trust().unwrap(),
            ProjectTrust::Unknown(_)
        ));
        assert!(model.project_aliases().is_none());
    }

    #[test]
    fn model_without_aliases_file_has_no_project_trust() {
        let dir = tempfile::tempdir().unwrap();
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let security = SecurityConfig::default();

        let model = AppModel::new_with_security(config, profile_config, security)
            .with_cwd(dir.path().to_path_buf());
        assert!(model.project_trust().is_none());
    }

    #[test]
    fn trust_message_on_unknown_project_returns_save_security() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nb = \"make build\"\n",
        )
        .unwrap();

        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let security = SecurityConfig::default();
        let mut model = AppModel::new_with_security(config, profile_config, security)
            .with_cwd(dir.path().to_path_buf());

        assert!(matches!(
            model.project_trust(),
            Some(ProjectTrust::Unknown(_))
        ));

        let result = update(&mut model, Message::Trust).unwrap();
        assert!(result.effects.contains(&Effect::SaveSecurity));
        assert!(model.project_trust().unwrap().is_trusted());
    }

    #[test]
    fn untrust_message_moves_to_untrusted() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        std::fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();

        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut security = SecurityConfig::default();
        let hash = crate::trust::compute_file_hash(&aliases_path).unwrap();
        security.trust(&aliases_path, &hash);

        let mut model = AppModel::new_with_security(config, profile_config, security)
            .with_cwd(dir.path().to_path_buf());

        let result = update(&mut model, Message::Untrust { forget: false }).unwrap();
        assert!(result.effects.contains(&Effect::SaveSecurity));
        assert!(matches!(
            model.project_trust(),
            Some(ProjectTrust::Untrusted(_))
        ));
    }

    #[test]
    fn untrust_forget_removes_from_tracking() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        std::fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();

        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut security = SecurityConfig::default();
        let hash = crate::trust::compute_file_hash(&aliases_path).unwrap();
        security.trust(&aliases_path, &hash);

        let mut model = AppModel::new_with_security(config, profile_config, security)
            .with_cwd(dir.path().to_path_buf());

        let result = update(&mut model, Message::Untrust { forget: true }).unwrap();
        assert!(result.effects.contains(&Effect::SaveSecurity));
        assert!(matches!(
            model.project_trust(),
            Some(ProjectTrust::Unknown(_))
        ));
        assert!(!model.security_config().is_tracked(&aliases_path));
    }

    #[test]
    fn add_global_subcommand_alias() {
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::AddSubcommandAlias(
                "jj:ab".into(),
                vec!["abandon".into()],
                AliasTarget::Global,
            ),
        )
        .unwrap();

        assert_eq!(model.config.subcommands.len(), 1);
        assert_eq!(model.config.subcommands["jj:ab"], vec!["abandon"]);
        assert_eq!(result.effects, vec![Effect::SaveConfig]);
    }

    #[test]
    fn remove_global_subcommand_alias() {
        let mut config = Config::default();
        config.add_subcommand("jj:ab".into(), vec!["abandon".into()]);
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::RemoveSubcommandAlias("jj:ab".into(), AliasTarget::Global),
        )
        .unwrap();

        assert!(model.config.subcommands.is_empty());
        assert_eq!(result.effects, vec![Effect::SaveConfig]);
    }

    #[test]
    fn add_local_subcommand_alias_returns_effect() {
        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::AddSubcommandAlias("jj:ab".into(), vec!["abandon".into()], AliasTarget::Local),
        )
        .unwrap();

        assert_eq!(
            result.effects,
            vec![Effect::AddLocalSubcommand {
                key: "jj:ab".into(),
                long_subcommands: vec!["abandon".into()],
            }]
        );
    }

    #[test]
    fn add_subcommand_to_named_profile() {
        let config = Config::default();
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"rust\"\n").unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::AddSubcommandAlias(
                "cargo:t".into(),
                vec!["test".into()],
                AliasTarget::Profile("rust".into()),
            ),
        )
        .unwrap();

        assert_eq!(result.effects, vec![Effect::SaveProfiles]);
        let profile = model.profile_config().get_profile_by_name("rust").unwrap();
        assert_eq!(profile.subcommands.len(), 1);
    }

    #[test]
    fn add_local_alias_on_untrusted_project_errors() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        std::fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();

        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut security = SecurityConfig::default();
        security.untrust(&aliases_path);
        let mut model = AppModel::new_with_security(config, profile_config, security)
            .with_cwd(dir.path().to_path_buf());

        let result = update(
            &mut model,
            Message::AddAlias("t".into(), "cargo test".into(), AliasTarget::Local, false),
        );
        assert!(matches!(result, Err(UpdateError::ProjectNotTrusted { .. })));
    }

    #[test]
    fn remove_local_alias_on_unknown_project_errors() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nb = \"make build\"\n",
        )
        .unwrap();

        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let security = SecurityConfig::default();
        let mut model = AppModel::new_with_security(config, profile_config, security)
            .with_cwd(dir.path().to_path_buf());

        let result = update(
            &mut model,
            Message::RemoveAlias("b".into(), AliasTarget::Local),
        );
        assert!(matches!(result, Err(UpdateError::ProjectNotTrusted { .. })));
    }

    #[cfg(feature = "test-util")]
    #[test]
    fn add_local_alias_on_untrusted_returns_typed_error() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        std::fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();

        let config = Config::default();
        let profile_config = ProfileConfig::default();
        let mut security = SecurityConfig::default();
        security.untrust(&aliases_path);
        let mut model = AppModel::new_with_security(config, profile_config, security)
            .with_cwd(dir.path().to_path_buf());

        let result = update(
            &mut model,
            Message::AddAlias("t".into(), "cargo test".into(), AliasTarget::Local, false),
        );
        assert!(matches!(result, Err(UpdateError::ProjectNotTrusted { .. })));
    }

    #[test]
    fn update_alias_renames_global_alias() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let mut model = AppModel::new(config, ProfileConfig::default());

        let result = update(
            &mut model,
            Message::UpdateAlias {
                target: AliasTarget::Global,
                old_name: "ll".into(),
                new_name: "la".into(),
                new_command: "ls -lha".into(),
                raw: false,
            },
        )
        .unwrap();

        assert_eq!(result.effects, vec![Effect::SaveConfig]);
        let key = crate::AliasName::from("la");
        assert!(model.config.aliases.contains_key(&key));
        let key_old = crate::AliasName::from("ll");
        assert!(!model.config.aliases.contains_key(&key_old));
    }

    #[test]
    fn update_alias_changes_command_global() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let mut model = AppModel::new(config, ProfileConfig::default());

        update(
            &mut model,
            Message::UpdateAlias {
                target: AliasTarget::Global,
                old_name: "ll".into(),
                new_name: "ll".into(),
                new_command: "ls -la".into(),
                raw: false,
            },
        )
        .unwrap();

        let key = crate::AliasName::from("ll");
        let alias = model.config.aliases.get(&key).unwrap();
        assert_eq!(alias.command(), "ls -la");
    }

    #[test]
    fn update_alias_returns_error_for_missing_alias() {
        let mut model = AppModel::new(Config::default(), ProfileConfig::default());

        let result = update(
            &mut model,
            Message::UpdateAlias {
                target: AliasTarget::Global,
                old_name: "nope".into(),
                new_name: "nope".into(),
                new_command: "cmd".into(),
                raw: false,
            },
        );
        assert!(matches!(result, Err(UpdateError::AliasNotFound { .. })));
    }

    #[test]
    fn move_aliases_from_global_to_profile() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"nav\"\n").unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::MoveAliases {
                aliases: vec![crate::AliasId::Global {
                    alias_name: "ll".into(),
                }],
                to: AliasTarget::Profile("nav".into()),
            },
        )
        .unwrap();

        let key = crate::AliasName::from("ll");
        assert!(!model.config.aliases.contains_key(&key));
        let profile = model.profile_config().get_profile_by_name("nav").unwrap();
        assert!(profile.aliases.contains_key(&key));
        assert!(result.effects.contains(&Effect::SaveConfig));
        assert!(result.effects.contains(&Effect::SaveProfiles));
    }

    #[test]
    fn copy_aliases_preserves_source() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"nav\"\n").unwrap();
        let mut model = AppModel::new(config, profile_config);

        update(
            &mut model,
            Message::CopyAliases {
                aliases: vec![crate::AliasId::Global {
                    alias_name: "ll".into(),
                }],
                to: AliasTarget::Profile("nav".into()),
            },
        )
        .unwrap();

        let key = crate::AliasName::from("ll");
        assert!(model.config.aliases.contains_key(&key));
        let profile = model.profile_config().get_profile_by_name("nav").unwrap();
        assert!(profile.aliases.contains_key(&key));
    }

    #[test]
    fn move_to_untrusted_project_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        std::fs::write(&aliases_path, "[aliases]\n").unwrap();
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let mut security = SecurityConfig::default();
        security.untrust(&aliases_path);
        let mut model = AppModel::new_with_security(config, ProfileConfig::default(), security)
            .with_cwd(dir.path().to_path_buf());

        let result = update(
            &mut model,
            Message::MoveAliases {
                aliases: vec![crate::AliasId::Global {
                    alias_name: "ll".into(),
                }],
                to: AliasTarget::Local,
            },
        );
        assert!(matches!(result, Err(UpdateError::ProjectNotTrusted { .. })));
    }

    #[test]
    fn rename_profile_renames_in_config_and_active_list() {
        let profile_config: ProfileConfig = toml::from_str(
            "[[profiles]]\nname = \"git\"\n[profiles.aliases]\ngs = \"git status\"\n",
        )
        .unwrap();
        let mut model = AppModel::new(Config::default(), profile_config);
        model.session.active_profiles = vec!["git".to_string()];

        let result = update(
            &mut model,
            Message::RenameProfile {
                old_name: "git".into(),
                new_name: "vcs".into(),
            },
        )
        .unwrap();

        assert_eq!(
            result.effects,
            vec![Effect::SaveProfiles, Effect::SaveSession]
        );
        assert!(model.profile_config().get_profile_by_name("vcs").is_some());
        assert!(model.profile_config().get_profile_by_name("git").is_none());
        assert!(model.session.active_profiles.contains(&"vcs".to_string()));
        assert!(!model.session.active_profiles.contains(&"git".to_string()));
        // Aliases preserved
        let profile = model.profile_config().get_profile_by_name("vcs").unwrap();
        let key = crate::AliasName::from("gs");
        assert!(profile.aliases.contains_key(&key));
    }

    #[test]
    fn rename_profile_returns_error_for_missing_profile() {
        let config = Config::default();
        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"git\"\n").unwrap();
        let mut model = AppModel::new(config, profile_config);

        let result = update(
            &mut model,
            Message::RenameProfile {
                old_name: "nope".into(),
                new_name: "vcs".into(),
            },
        );
        assert!(matches!(result, Err(UpdateError::ProfileNotFound { .. })));
    }

    #[cfg(feature = "test-util")]
    mod load_from_tests {
        use super::*;

        #[test]
        fn app_model_load_from_reads_config_from_given_dir() {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(
                dir.path().join("config.toml"),
                "[aliases]\nll = \"ls -lha\"\n",
            )
            .unwrap();

            let model = AppModel::load_from(dir.path().to_path_buf());
            assert_eq!(model.config.aliases.iter().count(), 1);
            assert_eq!(model.config_dir(), dir.path());
        }

        #[test]
        fn app_model_load_from_returns_default_for_empty_dir() {
            let dir = tempfile::tempdir().unwrap();
            let model = AppModel::load_from(dir.path().to_path_buf());
            assert!(model.config.aliases.is_empty());
            assert_eq!(model.config_dir(), dir.path());
        }
    }

    #[cfg(feature = "test-util")]
    mod save_tests {
        use super::*;

        #[test]
        fn save_config_writes_to_config_dir() {
            let dir = tempfile::tempdir().unwrap();
            let mut model = AppModel::load_from(dir.path().to_path_buf());
            model.config.add_alias("ll".into(), "ls -lha".into(), false);
            model.save_config().unwrap();

            let saved = Config::load_from(dir.path()).unwrap();
            assert_eq!(saved.aliases.iter().count(), 1);
        }

        #[test]
        fn save_profiles_writes_to_config_dir() {
            let dir = tempfile::tempdir().unwrap();
            let mut model = AppModel::load_from(dir.path().to_path_buf());
            let _ = model.profile_config_mut().add_profile("rust");
            model.save_profiles().unwrap();

            let saved = ProfileConfig::load_from(dir.path()).unwrap();
            assert!(saved.get_profile_by_name("rust").is_some());
        }

        #[test]
        fn save_security_writes_to_config_dir() {
            let dir = tempfile::tempdir().unwrap();
            let aliases_path = dir.path().join(".aliases");
            std::fs::write(&aliases_path, "[aliases]\n").unwrap();
            let mut model = AppModel::load_from(dir.path().to_path_buf());
            let hash = crate::trust::compute_file_hash(&aliases_path).unwrap();
            model.security_config_mut().trust(&aliases_path, &hash);
            model.save_security().unwrap();

            let saved = SecurityConfig::load_from(dir.path()).unwrap();
            assert!(saved.is_tracked(&aliases_path));
        }
    }
}
