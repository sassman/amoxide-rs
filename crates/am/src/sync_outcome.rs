use crate::config::{LogVerbosity, LoggingConfig, ShellsTomlConfig};
use crate::effects::Echo;
use crate::env_vars;
use crate::precedence::PrecedenceDiff;
use crate::shell::Shell;
use crate::subcommand::SubcommandSet;
use crate::trust::render_load_lines;
use crate::AliasSet;

pub enum ProjectTransition {
    FreshLoad {
        aliases: AliasSet,
        subcommands: SubcommandSet,
    },
    Unloaded,
    None,
}

pub enum PathUpdate {
    Set(String),
    Unset,
    Unchanged,
}

pub struct SyncOutcome {
    pub shell: Shell,
    pub shell_cfg: ShellsTomlConfig,
    pub quiet: bool,
    pub transition: ProjectTransition,
    pub diff: PrecedenceDiff,
    pub security_warnings: Vec<String>,
    pub path_update: PathUpdate,
}

impl SyncOutcome {
    pub fn render(&self, logging: &LoggingConfig) -> Vec<Echo> {
        let shell_impl = self.shell.clone().as_shell(
            &self.shell_cfg,
            Default::default(),
            Default::default(),
        );
        let mut lines = Vec::new();

        // Security warnings (unless quiet)
        if !self.quiet {
            for warn in &self.security_warnings {
                lines.push(Echo::Line(shell_impl.echo(warn)));
            }
        }

        // Human-readable transition message (unless quiet)
        if !self.quiet {
            match &self.transition {
                ProjectTransition::FreshLoad { aliases, subcommands } => {
                    let verbosity = logging
                        .project_loading
                        .as_ref()
                        .unwrap_or(&LogVerbosity::Verbose);
                    lines.extend(render_load_lines(
                        aliases,
                        subcommands,
                        verbosity,
                        shell_impl.as_ref(),
                    ));
                }
                ProjectTransition::Unloaded => {
                    let verbosity = logging
                        .project_unloading
                        .as_ref()
                        .unwrap_or(&LogVerbosity::Verbose);
                    lines.push(Echo::from_verbosity(
                        verbosity,
                        || shell_impl.echo("am: .aliases unloaded"),
                        || {
                            let msg = self
                                .diff
                                .unload_summary()
                                .unwrap_or_else(|| "am: .aliases unloaded".to_string());
                            shell_impl.echo(&msg)
                        },
                    ));
                }
                ProjectTransition::None => {
                    if let Some(msg) = self.diff.change_summary() {
                        lines.push(Echo::Line(shell_impl.echo(&msg)));
                    }
                }
            }
        }

        // Functional shell commands (always — these ARE the program output)
        let rendered = self.diff.render(shell_impl.as_ref());
        if !rendered.is_empty() {
            for line in rendered.lines() {
                lines.push(Echo::always(line.to_string()));
            }
        }

        // Path tracking (always)
        match &self.path_update {
            PathUpdate::Set(p) => {
                lines.push(Echo::Line(
                    shell_impl.set_env(env_vars::AM_PROJECT_PATH, p),
                ));
            }
            PathUpdate::Unset => {
                lines.push(Echo::Line(
                    shell_impl.unset_env(env_vars::AM_PROJECT_PATH),
                ));
            }
            PathUpdate::Unchanged => {}
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_outcome(
        transition: ProjectTransition,
        quiet: bool,
        path_update: PathUpdate,
    ) -> SyncOutcome {
        SyncOutcome {
            shell: Shell::Fish,
            shell_cfg: ShellsTomlConfig::default(),
            quiet,
            transition,
            diff: PrecedenceDiff::default(),
            security_warnings: vec![],
            path_update,
        }
    }

    #[test]
    fn render_quiet_suppresses_all_messages() {
        let outcome = make_outcome(ProjectTransition::Unloaded, true, PathUpdate::Unchanged);
        let logging = LoggingConfig::default();
        let lines = outcome.render(&logging);
        assert!(lines.iter().all(|l| matches!(l, Echo::Silent)));
    }

    #[test]
    fn render_unloaded_off_produces_silent() {
        let outcome = make_outcome(ProjectTransition::Unloaded, false, PathUpdate::Unchanged);
        let logging = LoggingConfig {
            project_loading: None,
            project_unloading: Some(LogVerbosity::Off),
        };
        let lines = outcome.render(&logging);
        assert!(lines.iter().all(|l| matches!(l, Echo::Silent)));
    }

    #[test]
    fn render_unloaded_short_produces_message() {
        let outcome = make_outcome(ProjectTransition::Unloaded, false, PathUpdate::Unchanged);
        let logging = LoggingConfig {
            project_loading: None,
            project_unloading: Some(LogVerbosity::Short),
        };
        let lines = outcome.render(&logging);
        let text_lines: Vec<&str> = lines
            .iter()
            .filter_map(|l| match l {
                Echo::Line(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert!(text_lines.iter().any(|s| s.contains("am: .aliases unloaded")));
    }

    #[test]
    fn render_path_set_emits_set_env() {
        let outcome = make_outcome(
            ProjectTransition::None,
            true,
            PathUpdate::Set("/project/.aliases".into()),
        );
        let logging = LoggingConfig::default();
        let lines = outcome.render(&logging);
        let text_lines: Vec<&str> = lines
            .iter()
            .filter_map(|l| match l {
                Echo::Line(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert!(text_lines.iter().any(|s| s.contains("_AM_PROJECT_PATH")));
    }

    #[test]
    fn render_path_unset_emits_unset_env() {
        let outcome = make_outcome(
            ProjectTransition::None,
            true,
            PathUpdate::Unset,
        );
        let logging = LoggingConfig::default();
        let lines = outcome.render(&logging);
        let text_lines: Vec<&str> = lines
            .iter()
            .filter_map(|l| match l {
                Echo::Line(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert!(text_lines.iter().any(|s| s.contains("_AM_PROJECT_PATH")));
    }

    #[test]
    fn render_security_warnings_unless_quiet() {
        let mut outcome = make_outcome(ProjectTransition::None, false, PathUpdate::Unchanged);
        outcome.security_warnings = vec!["am: .aliases found but not trusted.".into()];
        let logging = LoggingConfig::default();
        let lines = outcome.render(&logging);
        let text_lines: Vec<&str> = lines
            .iter()
            .filter_map(|l| match l {
                Echo::Line(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert!(text_lines.iter().any(|s| s.contains("not trusted")));
    }
}
