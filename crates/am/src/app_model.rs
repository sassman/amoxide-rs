use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::project::{ProjectAliases, ALIASES_FILE};
use crate::security::{SecurityConfig, TrustStatus};
use crate::trust::{compute_file_hash, ProjectTrust};
use crate::{AliasName, AliasSet, Profile, ProfileConfig, Session};

pub struct AppModel {
    pub config: Config,
    pub session: Session,
    pub(crate) cwd: std::path::PathBuf,
    config_dir: PathBuf,
    profile_config: ProfileConfig,
    security_config: SecurityConfig,
    pub(crate) project_trust: Option<ProjectTrust>,
}

fn resolve_project_trust(cwd: &Path, security_config: &mut SecurityConfig) -> Option<ProjectTrust> {
    let project_path = ProjectAliases::find_path(cwd).ok().flatten()?;
    let hash = compute_file_hash(&project_path).ok()?;
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
        let mut session = Session::load_from(&config_dir).unwrap_or_default();
        let profile_config = ProfileConfig::load_from(&config_dir).unwrap_or_default();
        // Silently drop any active profile that no longer exists.
        session
            .active_profiles
            .retain(|name| profile_config.get_profile_by_name(name).is_some());
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

    /// Get project aliases' AliasSet, or empty default.
    pub fn project_alias_set(&self) -> AliasSet {
        self.project_aliases()
            .map(|p| p.aliases.clone())
            .unwrap_or_default()
    }

    /// Get project aliases and subcommands together; both empty if project is absent/untrusted.
    pub fn project_alias_set_and_subcommands(
        &self,
    ) -> (AliasSet, crate::subcommand::SubcommandSet) {
        match self.project_aliases() {
            Some(p) => (p.aliases.clone(), p.subcommands.clone()),
            None => (AliasSet::default(), crate::subcommand::SubcommandSet::new()),
        }
    }

    /// Get or create the project path (for saving new .aliases files).
    /// If no .aliases exists, returns cwd/.aliases
    pub fn project_path_or_create(&self) -> PathBuf {
        self.project_path()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.cwd.join(ALIASES_FILE))
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
        let hash = compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        self.save_security()?;
        Ok(())
    }

    /// Remove an alias from the project .aliases file, saving to disk.
    pub fn save_project_aliases_remove(&mut self, name: &str) -> crate::Result<()> {
        let path = self.project_path_or_create();
        let mut project = self.project_aliases().cloned().unwrap_or_default();
        let key = AliasName::from(name);
        project.aliases.remove(&key);
        project.save(&path)?;
        let hash = compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        self.save_security()?;
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
        let hash = compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        self.save_security()?;
        Ok(())
    }

    /// Merge a full SubcommandSet into the project .aliases file, saving to disk.
    pub fn save_project_subcommands(
        &mut self,
        subcommands: crate::subcommand::SubcommandSet,
    ) -> crate::Result<()> {
        let path = self.project_path_or_create();
        let mut project = self.project_aliases().cloned().unwrap_or_default();
        for (key, longs) in subcommands {
            project.subcommands.as_mut().insert(key, longs);
        }
        project.save(&path)?;
        let hash = compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        self.save_security()?;
        Ok(())
    }

    /// Remove a subcommand alias from the project .aliases file, saving to disk.
    pub fn save_project_subcommand_remove(&mut self, key: &str) -> crate::Result<()> {
        let path = self.project_path_or_create();
        let mut project = self.project_aliases().cloned().unwrap_or_default();
        project.remove_subcommand(key)?;
        project.save(&path)?;
        let hash = compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        self.save_security()?;
        Ok(())
    }

    pub fn save_project_vars_set(&mut self, name: &str, value: &str) -> crate::Result<()> {
        let path = self.project_path_or_create();
        let mut project = self.project_aliases().cloned().unwrap_or_default();
        let parsed = crate::vars::VarName::parse(name).map_err(|e| anyhow::anyhow!("{e}"))?;
        project.vars.insert(parsed, value.to_string());
        project.save(&path)?;
        let hash = compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        self.save_security()?;
        Ok(())
    }

    pub fn save_project_vars_unset(&mut self, name: &str) -> crate::Result<()> {
        let path = self.project_path_or_create();
        let mut project = self.project_aliases().cloned().unwrap_or_default();
        let parsed = crate::vars::VarName::parse(name).map_err(|e| anyhow::anyhow!("{e}"))?;
        project
            .vars
            .remove(&parsed)
            .ok_or_else(|| anyhow::anyhow!("variable '{name}' not found in .aliases"))?;
        project.save(&path)?;
        let hash = compute_file_hash(&path)?;
        self.security_config_mut().trust(&path, &hash);
        self.project_trust = Some(ProjectTrust::Trusted(project, path));
        self.save_security()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProfileConfig, Session};

    #[test]
    fn load_prunes_stale_active_profiles() {
        let dir = tempfile::tempdir().unwrap();
        let config_dir = dir.path().to_path_buf();

        // Write a session with "misc" and "rust" active, but only "rust" will exist as a profile.
        let session = Session {
            active_profiles: vec!["misc".to_string(), "rust".to_string()],
        };
        session.save_to(&config_dir).unwrap();

        let profile_config: ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"rust\"\n").unwrap();
        profile_config.save_to(&config_dir).unwrap();

        let model = AppModel::load_from_internal(config_dir);
        assert_eq!(
            model.session.active_profiles,
            vec!["rust".to_string()],
            "stale profile 'misc' should be pruned silently on load"
        );
    }
}
