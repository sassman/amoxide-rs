use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::display::render_listing;
use crate::effects::Effect;
use crate::init::{generate_init, generate_reload};
use crate::project::ProjectAliases;
use crate::security::{SecurityConfig, TrustStatus};
use crate::trust::ProjectTrust;
use crate::{profile, AliasSet, AliasTarget, Message, Profile, ProfileConfig, Session};

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

pub struct AppModel {
    pub config: Config,
    pub session: Session,
    pub cwd: std::path::PathBuf,
    config_dir: PathBuf,
    profile_config: ProfileConfig,
    security_config: SecurityConfig,
    project_trust: Option<ProjectTrust>,
}

fn resolve_project_trust(cwd: &Path, security_config: &mut SecurityConfig) -> Option<ProjectTrust> {
    let project_path = ProjectAliases::find_path(cwd).ok().flatten()?;
    let hash = crate::trust::compute_file_hash(&project_path).ok()?;
    let status = security_config.check(&project_path, &hash);

    Some(match status {
        TrustStatus::Trusted => {
            let aliases = ProjectAliases::load(&project_path).ok()?;
            ProjectTrust::Trusted(aliases, project_path)
        }
        TrustStatus::Untrusted => ProjectTrust::Untrusted(project_path),
        TrustStatus::Tampered => ProjectTrust::Tampered(project_path),
        TrustStatus::Unknown => ProjectTrust::Unknown(project_path),
    })
}

impl Default for AppModel {
    fn default() -> Self {
        Self::load_from_internal(crate::dirs::config_dir())
    }
}

impl AppModel {
    fn load_from_internal(config_dir: PathBuf) -> Self {
        let config = Config::load_from(&config_dir).unwrap_or_default();
        let session = Session::load_from(&config_dir).unwrap_or_default();
        let profile_config = ProfileConfig::load_from(&config_dir).unwrap_or_default();
        let mut security_config = SecurityConfig::load_from(&config_dir).unwrap_or_default();
        let cwd = std::env::current_dir().unwrap_or_default();
        let project_trust = resolve_project_trust(&cwd, &mut security_config);
        Self {
            config,
            session,
            cwd,
            config_dir,
            profile_config,
            security_config,
            project_trust,
        }
    }

    #[cfg(feature = "test-util")]
    pub fn load_from(config_dir: PathBuf) -> Self {
        Self::load_from_internal(config_dir)
    }

    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    pub fn new(config: Config, profile_config: ProfileConfig) -> Self {
        Self {
            config,
            session: Session::default(),
            cwd: std::env::current_dir().unwrap_or_default(),
            config_dir: PathBuf::new(),
            profile_config,
            security_config: SecurityConfig::default(),
            project_trust: None,
        }
    }

    pub fn new_with_security(
        config: Config,
        profile_config: ProfileConfig,
        security_config: SecurityConfig,
    ) -> Self {
        Self {
            config,
            session: Session::default(),
            cwd: std::env::current_dir().unwrap_or_default(),
            config_dir: PathBuf::new(),
            profile_config,
            security_config,
            project_trust: None,
        }
    }

    pub fn with_cwd(mut self, cwd: std::path::PathBuf) -> Self {
        self.project_trust = resolve_project_trust(&cwd, &mut self.security_config);
        self.cwd = cwd;
        self
    }

    pub fn project_trust(&self) -> Option<&ProjectTrust> {
        self.project_trust.as_ref()
    }

    pub fn project_aliases(&self) -> Option<&ProjectAliases> {
        self.project_trust.as_ref().and_then(|t| t.aliases())
    }

    pub fn project_path(&self) -> Option<&Path> {
        self.project_trust.as_ref().map(|t| t.path())
    }

    /// Get project aliases' AliasSet, or empty default
    pub fn project_alias_set(&self) -> AliasSet {
        self.project_aliases()
            .map(|p| p.aliases.clone())
            .unwrap_or_default()
    }

    /// Get or create the project path (for saving new .aliases files).
    /// If no .aliases exists, returns cwd/.aliases
    pub fn project_path_or_create(&self) -> PathBuf {
        self.project_path()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.cwd.join(crate::project::ALIASES_FILE))
    }

    /// Merge aliases into project aliases and save.
    pub fn save_project_aliases(&mut self, aliases: AliasSet) -> anyhow::Result<()> {
        let path = self.project_path_or_create();
        let current_aliases = self.project_aliases().cloned().unwrap_or_default();
        let mut project = current_aliases;
        project.merge_aliases(aliases);
        project.save(&path)?;
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        Ok(())
    }

    pub fn security_config(&self) -> &SecurityConfig {
        &self.security_config
    }

    pub fn security_config_mut(&mut self) -> &mut SecurityConfig {
        &mut self.security_config
    }

    pub fn profile_config_mut(&mut self) -> &mut ProfileConfig {
        &mut self.profile_config
    }

    pub fn profile_config(&self) -> &ProfileConfig {
        &self.profile_config
    }

    pub fn get_active_profiles(&self) -> Vec<&Profile> {
        self.session
            .active_profiles
            .iter()
            .filter_map(|name| self.profile_config.get_profile_by_name(name))
            .collect()
    }

    pub fn save_config(&self) -> crate::Result<()> {
        if self.config_dir.as_os_str().is_empty() {
            return Ok(());
        }
        self.config.save_to(&self.config_dir)
    }

    pub fn save_session(&self) -> crate::Result<()> {
        if self.config_dir.as_os_str().is_empty() {
            return Ok(());
        }
        self.session.save_to(&self.config_dir)
    }

    pub fn save_profiles(&self) -> crate::Result<()> {
        if self.config_dir.as_os_str().is_empty() {
            return Ok(());
        }
        self.profile_config.save_to(&self.config_dir)
    }

    pub fn save_security(&self) -> crate::Result<()> {
        if self.config_dir.as_os_str().is_empty() {
            return Ok(());
        }
        self.security_config.save_to(&self.config_dir)
    }

    /// Add an alias to the project .aliases file, saving to disk.
    /// Updates project_trust in-memory to reflect the new content.
    pub fn save_project_aliases_add(
        &mut self,
        name: &str,
        cmd: &str,
        raw: bool,
    ) -> crate::Result<()> {
        let path = self.project_path_or_create();
        let mut project = self.project_aliases().cloned().unwrap_or_default();
        project.add_alias(name.to_string(), cmd.to_string(), raw);
        project.save(&path)?;
        let hash = crate::trust::compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        Ok(())
    }

    /// Remove an alias from the project .aliases file, saving to disk.
    pub fn save_project_aliases_remove(&mut self, name: &str) -> crate::Result<()> {
        let path = self.project_path_or_create();
        let mut project = self.project_aliases().cloned().unwrap_or_default();
        let key = crate::AliasName::from(name);
        project.aliases.remove(&key);
        project.save(&path)?;
        let hash = crate::trust::compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        Ok(())
    }

    /// Add a subcommand alias to the project .aliases file, saving to disk.
    pub fn save_project_subcommand_add(
        &mut self,
        key: &str,
        long_subcommands: &[String],
    ) -> crate::Result<()> {
        let path = self.project_path_or_create();
        let mut project = self.project_aliases().cloned().unwrap_or_default();
        project.add_subcommand(key.to_string(), long_subcommands.to_vec());
        project.save(&path)?;
        let hash = crate::trust::compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        Ok(())
    }

    /// Remove a subcommand alias from the project .aliases file, saving to disk.
    pub fn save_project_subcommand_remove(&mut self, key: &str) -> crate::Result<()> {
        let path = self.project_path_or_create();
        let mut project = self.project_aliases().cloned().unwrap_or_default();
        project.remove_subcommand(key)?;
        project.save(&path)?;
        let hash = crate::trust::compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        Ok(())
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
            AliasTarget::ActiveProfile if model.config.active_profiles.is_empty() => {
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
            AliasTarget::ActiveProfile if model.config.active_profiles.is_empty() => {
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
        Message::ListProfiles => {
            let resolved_subs = model
                .profile_config()
                .resolve_active_subcommands(&model.config.active_profiles);
            let mut all_subs = model.config.subcommands.clone();
            for (k, v) in resolved_subs {
                all_subs.insert(k, v);
            }
            let output = render_listing(
                &model.config.aliases,
                model.profile_config(),
                &model.session.active_profiles,
                model.project_trust(),
                &all_subs,
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
        Message::InitShell(shell) => {
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
            let output = generate_init(&shell, &model.config.aliases, &resolved, &all_subs);
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
            let prev = std::env::var("_AM_ALIASES").ok();
            let output = generate_reload(
                &shell,
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
            let prev = std::env::var("_AM_PROJECT_ALIASES").ok();
            let (output, security_changed) = crate::hook::generate_hook_with_security(
                &shell,
                &model.cwd,
                prev.as_deref(),
                &mut model.security_config,
                quiet,
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
                let alias_count = model
                    .profile_config()
                    .get_profile_by_name(&name)
                    .map(|p| p.aliases.iter().count())
                    .unwrap_or(0);
                model.session.toggle_profile(name.clone());
                let (action, verb) = if was_active {
                    ("deactivated", "unloaded")
                } else {
                    ("activated", "loaded")
                };
                effects.push(Effect::Print(format!(
                    "{name} {action}, {alias_count} aliases {verb}"
                )));
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
                let alias_count = model
                    .profile_config()
                    .get_profile_by_name(&name)
                    .map(|p| p.aliases.iter().count())
                    .unwrap_or(0);
                model.session.use_profile_at(name.clone(), priority + i);
                effects.push(Effect::Print(format!(
                    "{name} activated at position {}, {alias_count} aliases loaded",
                    priority + i
                )));
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
            for profile in payload.profiles {
                model.profile_config_mut().merge_profile(profile);
                if !effects.contains(&Effect::SaveProfiles) {
                    effects.push(Effect::SaveProfiles);
                }
            }
            // local_aliases are saved by the CLI layer (needs file path)
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::effects::Effect;

    #[test]
    fn update_result_done_has_no_message_or_effects() {
        let r = UpdateResult::done();
        assert!(r.next.is_none());
        assert!(r.effects.is_empty());
    }

    #[test]
    fn update_result_message_has_message_and_no_effects() {
        let r = UpdateResult::message(Message::ListProfiles);
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
            Message::ListProfiles,
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
            Message::AddSubcommandAlias(
                "jj:ab".into(),
                vec!["abandon".into()],
                AliasTarget::Local,
            ),
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
