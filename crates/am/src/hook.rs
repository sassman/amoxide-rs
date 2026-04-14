use std::path::Path;

use crate::project::ProjectAliases;
use crate::security::{SecurityConfig, TrustStatus};
use crate::shell::ShellContext;
use crate::trust::{compute_file_hash, render_load_message, render_unload_message};

/// Generate shell code for the cd hook.
///
/// `ctx.cwd` — the current working directory to search for `.aliases`.
/// `previous_aliases` — comma-separated alias names from `_AM_PROJECT_ALIASES` env var.
pub fn generate_hook(ctx: &ShellContext, previous_aliases: Option<&str>) -> crate::Result<String> {
    let mut security = SecurityConfig::load().unwrap_or_default();
    let prev_project_path = std::env::var("_AM_PROJECT_PATH").ok();
    let (output, _changed) = generate_hook_with_security(
        ctx,
        previous_aliases,
        prev_project_path.as_deref(),
        &mut security,
        false,
    )?;
    Ok(output)
}

/// Generate shell code for the cd hook with explicit security config.
///
/// Returns `(shell_code, security_changed)` — `security_changed` is true
/// when a tamper was detected and `security_config` was mutated in memory.
///
/// When `quiet` is true, info and warning echo messages are suppressed
/// (alias loading/unloading still happens).
///
/// `prev_project_path` — the value of `_AM_PROJECT_PATH` from the shell
/// environment, used to suppress duplicate warnings. Pass `None` to treat
/// the env var as unset (e.g. in tests).
pub fn generate_hook_with_security(
    ctx: &ShellContext,
    previous_aliases: Option<&str>,
    prev_project_path: Option<&str>,
    security_config: &mut SecurityConfig,
    quiet: bool,
) -> crate::Result<(String, bool)> {
    let shell_impl = ctx.shell.clone().as_shell(ctx.cfg);
    let cwd = ctx.cwd;
    let mut lines: Vec<String> = Vec::new();
    let mut security_changed = false;

    let prev: Vec<&str> = previous_aliases
        .filter(|s| !s.is_empty())
        .map(|s| s.split(',').collect())
        .unwrap_or_default();

    // `prev_project_path` tracks which .aliases file was last seen, to avoid
    // repeating warnings. It is passed in explicitly rather than read from the
    // environment so that callers (e.g. tests) can control it independently.

    // Helper: unalias only shell-level names (no `:` — subcommand keys like `c:l`
    // are tracked for change detection but are not themselves shell functions).
    let unload_prev = |lines: &mut Vec<String>| {
        for name in &prev {
            if !name.contains(':') {
                lines.push(shell_impl.unalias(name));
            }
        }
    };

    let project_path = ProjectAliases::find_path(cwd)?;

    match project_path {
        Some(path) => {
            let hash = compute_file_hash(&path)?;
            let status = security_config.check(&path, &hash);

            // Only show info/warning messages when:
            // - not in quiet mode
            // - the .aliases file is directly in cwd (not inherited from parent)
            // - we haven't already shown a message for this exact file
            let is_direct = path.parent().is_some_and(|p| p == cwd);
            let already_seen = prev_project_path.is_some_and(|p| Path::new(p) == path);
            let show_messages = !quiet && is_direct && !already_seen;

            match status {
                TrustStatus::Trusted => {
                    let project = ProjectAliases::load(&path)?;
                    if !project.aliases.is_empty() || !project.subcommands.is_empty() {
                        let names: Vec<String> = project
                            .aliases
                            .iter()
                            .map(|(n, _)| n.as_ref().to_string())
                            .collect();

                        let subcmd_groups =
                            crate::subcommand::group_by_program(&project.subcommands);

                        // all_names tracks both the shell-level wrapper names (e.g. `c`) and
                        // the individual subcommand keys (e.g. `c:l`, `c:t`). Wrapper names
                        // are used to unload old functions; subcommand keys make change
                        // detection precise — adding c:t when c:l already exists would
                        // otherwise appear identical (both produce program name `c`).
                        let subcmd_program_names: Vec<String> =
                            subcmd_groups.keys().cloned().collect();
                        let subcmd_keys: Vec<String> =
                            project.subcommands.keys().cloned().collect();

                        let mut all_names: Vec<String> = names.clone();
                        all_names.extend(subcmd_program_names.clone());
                        all_names.extend(subcmd_keys);
                        all_names.sort();
                        all_names.dedup();

                        // If the exact same set of aliases and subcommand keys is already
                        // loaded, skip entirely — nothing changed.
                        if all_names.len() == prev.len()
                            && all_names.iter().zip(&prev).all(|(a, b)| a == b)
                        {
                            return Ok((String::new(), false));
                        }

                        unload_prev(&mut lines);

                        if show_messages {
                            for line in
                                render_load_message(&project.aliases, &project.subcommands).lines()
                            {
                                lines.push(shell_impl.echo(line));
                            }
                        }

                        // Emit regular aliases (skip those with subcommand wrappers)
                        let programs_set: std::collections::BTreeSet<&str> =
                            subcmd_groups.keys().map(|s| s.as_str()).collect();
                        for (alias_name, alias_value) in project.aliases.iter() {
                            let name = alias_name.as_ref();
                            if !programs_set.contains(name) {
                                lines.push(shell_impl.alias(&alias_value.as_entry(name)));
                            }
                        }

                        // Emit subcommand wrappers
                        for (program, entries) in &subcmd_groups {
                            let base_cmd = project
                                .aliases
                                .iter()
                                .find(|(n, _)| n.as_ref() == program.as_str())
                                .map(|(_, v)| v.command().to_string())
                                .unwrap_or_else(|| format!("command {program}"));
                            lines.push(shell_impl.subcommand_wrapper(program, &base_cmd, entries));
                        }

                        lines.push(shell_impl.set_env("_AM_PROJECT_ALIASES", &all_names.join(",")));
                    }
                }
                TrustStatus::Unknown => {
                    unload_prev(&mut lines);
                    if show_messages {
                        lines.push(shell_impl.echo(
                            "am: .aliases found but not trusted. Run 'am trust' to review and allow.",
                        ));
                    }
                }
                TrustStatus::Untrusted => {
                    unload_prev(&mut lines);
                }
                TrustStatus::Tampered => {
                    unload_prev(&mut lines);
                    security_changed = true;
                    if show_messages {
                        lines.push(shell_impl.echo(
                            "am: .aliases was modified since last trusted. Run 'am trust' to review and allow.",
                        ));
                    }
                }
            }

            // For non-trusted states: track the path to avoid repeating warnings,
            // and clear the alias tracking env var.
            if !matches!(status, TrustStatus::Trusted) {
                lines.push(shell_impl.set_env("_AM_PROJECT_PATH", &path.display().to_string()));
                if !prev.is_empty() {
                    lines.push(shell_impl.unset_env("_AM_PROJECT_ALIASES"));
                }
            } else if prev_project_path.is_some() {
                lines.push(shell_impl.unset_env("_AM_PROJECT_PATH"));
            }
        }
        None => {
            if !prev.is_empty() {
                unload_prev(&mut lines);
                if !quiet {
                    lines.push(shell_impl.echo(&render_unload_message(&prev)));
                }
                lines.push(shell_impl.unset_env("_AM_PROJECT_ALIASES"));
            }
            if prev_project_path.is_some() {
                lines.push(shell_impl.unset_env("_AM_PROJECT_PATH"));
            }
        }
    }

    Ok((lines.join("\n"), security_changed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::Shells;
    use std::path::{Path, PathBuf};

    /// Builder for hook test fixtures.
    struct TestBed {
        dir: tempfile::TempDir,
        aliases_content: Option<String>,
        subdirs: Vec<PathBuf>,
        security: SecurityConfig,
    }

    impl TestBed {
        fn new() -> Self {
            Self {
                dir: tempfile::tempdir().unwrap(),
                aliases_content: None,
                subdirs: Vec::new(),
                security: SecurityConfig::default(),
            }
        }

        fn with_aliases(mut self, content: &str) -> Self {
            self.aliases_content = Some(content.to_string());
            self
        }

        fn with_subdir(mut self, rel_path: &str) -> Self {
            self.subdirs.push(PathBuf::from(rel_path));
            self
        }

        fn with_security_trusted(mut self) -> Self {
            let path = self.aliases_path();
            if let Some(content) = &self.aliases_content {
                std::fs::write(&path, content).unwrap();
            }
            let hash = compute_file_hash(&path).unwrap();
            self.security.trust(&path, &hash);
            self
        }

        fn with_security_untrusted(mut self) -> Self {
            self.security.untrust(&self.aliases_path());
            self
        }

        fn with_security_tampered(mut self) -> Self {
            self.security.trust(&self.aliases_path(), "wrong_hash");
            self
        }

        fn setup(self) -> SetupTestBed {
            let aliases_path = self.dir.path().join(".aliases");
            if let Some(content) = &self.aliases_content {
                std::fs::write(&aliases_path, content).unwrap();
            }
            for sub in &self.subdirs {
                std::fs::create_dir_all(self.dir.path().join(sub)).unwrap();
            }
            SetupTestBed {
                dir: self.dir,
                security: self.security,
            }
        }

        fn aliases_path(&self) -> PathBuf {
            self.dir.path().join(".aliases")
        }
    }

    struct SetupTestBed {
        dir: tempfile::TempDir,
        security: SecurityConfig,
    }

    impl SetupTestBed {
        fn root(&self) -> PathBuf {
            self.dir.path().to_path_buf()
        }

        fn subdir(&self, rel_path: &str) -> PathBuf {
            self.dir.path().join(rel_path)
        }

        fn run(&mut self, shell: &Shells, cwd: &Path, prev: Option<&str>) -> (String, bool) {
            use crate::config::ShellsTomlConfig;
            let cfg = ShellsTomlConfig::default();
            let ctx = ShellContext {
                shell,
                cfg: &cfg,
                cwd,
            };
            generate_hook_with_security(&ctx, prev, None, &mut self.security, false).unwrap()
        }

        /// Update the .aliases content and re-trust.
        fn update_aliases(&mut self, content: &str) {
            let path = self.dir.path().join(".aliases");
            std::fs::write(&path, content).unwrap();
            let hash = compute_file_hash(&path).unwrap();
            self.security.trust(&path, &hash);
        }
    }

    // ─── Basic hook behavior ────────────────────────────────────────

    #[test]
    fn test_hook_with_aliases_file() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\nt = \"make test\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shells::Fish, &cwd, None);
        assert!(output.contains("alias b \"make build\""));
        assert!(output.contains("alias t \"make test\""));
        assert!(output.contains("_AM_PROJECT_ALIASES"));
    }

    #[test]
    fn test_hook_unloads_previous_aliases() {
        let mut t = TestBed::new().setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shells::Fish, &cwd, Some("old1,old2"));
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("set -e _AM_PROJECT_ALIASES"));
    }

    #[test]
    fn test_hook_no_aliases_no_previous() {
        let mut t = TestBed::new().setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shells::Fish, &cwd, None);
        assert!(output.is_empty());
    }

    #[test]
    fn test_hook_transitions_between_projects() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nnew1 = \"echo new\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shells::Fish, &cwd, Some("old1,old2"));
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("alias new1 \"echo new\""));
        assert!(output.contains("\"new1\""));
    }

    #[test]
    fn test_hook_zsh_output() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shells::Zsh, &cwd, Some("old"));
        assert!(output.contains("unset -f old"));
        assert!(output.contains("b() { make build \"$@\"; }"));
        assert!(output.contains("export _AM_PROJECT_ALIASES="));
    }

    #[test]
    fn test_hook_picks_up_added_alias() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shells::Fish, &cwd, None);
        assert!(output.contains("alias b \"make build\""));
        assert!(!output.contains("alias t"));

        t.update_aliases("[aliases]\nb = \"make build\"\nt = \"make test\"\n");

        let (output, _) = t.run(&Shells::Fish, &cwd, Some("b"));
        assert!(output.contains("functions -e b"));
        assert!(output.contains("alias b \"make build\""));
        assert!(output.contains("alias t \"make test\""));
        assert!(output.contains("\"b,t\""));
    }

    #[test]
    fn test_hook_picks_up_removed_alias() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\nt = \"make test\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shells::Fish, &cwd, None);
        assert!(output.contains("alias b"));
        assert!(output.contains("alias t"));

        t.update_aliases("[aliases]\nb = \"make build\"\n");

        let (output, _) = t.run(&Shells::Fish, &cwd, Some("b,t"));
        assert!(output.contains("functions -e b"));
        assert!(output.contains("functions -e t"));
        assert!(output.contains("alias b \"make build\""));
        assert!(!output.contains("alias t \"make test\""));
        assert!(output.contains("\"b\""));
    }

    #[test]
    fn test_hook_bash_output() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shells::Bash, &cwd, Some("old"));
        assert!(output.contains("unset -f old"));
        assert!(output.contains("b() { make build \"$@\"; }"));
        assert!(output.contains("export _AM_PROJECT_ALIASES="));
    }

    #[test]
    fn test_hook_loads_aliases_from_parent_directory() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\nt = \"make test\"\n")
            .with_subdir("src/deep")
            .with_security_trusted()
            .setup();

        let sub = t.subdir("src/deep");
        let (output, _) = t.run(&Shells::Fish, &sub, None);
        assert!(
            output.contains("alias b \"make build\""),
            "should load aliases from parent .aliases, got: {output}"
        );
        assert!(output.contains("alias t \"make test\""));
        assert!(output.contains("_AM_PROJECT_ALIASES"));
    }

    // ─── Trust-gated hook tests ─────────────────────────────────────

    #[test]
    fn test_hook_trusted_shows_load_message() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, changed) = t.run(&Shells::Fish, &cwd, None);
        assert!(!changed);
        assert!(output.contains("alias b \"make build\""));
        assert!(output.contains("am: loaded .aliases"));
    }

    #[test]
    fn test_hook_unknown_shows_warning() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .setup();

        let cwd = t.root();
        let (output, changed) = t.run(&Shells::Fish, &cwd, None);
        assert!(!changed);
        assert!(!output.contains("alias b"));
        assert!(output.contains("am: .aliases found but not trusted"));
        assert!(output.contains("am trust"));
    }

    #[test]
    fn test_hook_untrusted_silent() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_untrusted()
            .setup();

        let cwd = t.root();
        let (output, changed) = t.run(&Shells::Fish, &cwd, None);
        assert!(!changed);
        assert!(!output.contains("alias b"));
        assert!(!output.contains("am:"));
    }

    #[test]
    fn test_hook_tampered_shows_loud_warning() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_tampered()
            .setup();

        let cwd = t.root();
        let (output, changed) = t.run(&Shells::Fish, &cwd, None);
        assert!(changed);
        assert!(!output.contains("alias b"));
        assert!(output.contains("modified since last trusted"));
    }

    #[test]
    fn test_hook_unload_shows_message() {
        let mut t = TestBed::new().setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shells::Fish, &cwd, Some("old1,old2"));
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("am: unloaded .aliases"));
    }

    // ─── Subdirectory behavior ──────────────────────────────────────

    #[test]
    fn test_hook_subdirectory_no_warning_for_parent_aliases() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_subdir("src")
            .setup();

        let sub = t.subdir("src");
        let (output, _) = t.run(&Shells::Fish, &sub, None);
        assert!(
            !output.contains("am:"),
            "should not show warning for parent .aliases, got: {output}"
        );
    }

    #[test]
    fn test_hook_subdirectory_trusted_loads_silently() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_subdir("src")
            .with_security_trusted()
            .setup();

        let sub = t.subdir("src");
        let (output, _) = t.run(&Shells::Fish, &sub, None);
        assert!(output.contains("alias b \"make build\""));
        assert!(
            !output.contains("am: loaded"),
            "should not show load message for parent .aliases, got: {output}"
        );
    }

    #[test]
    fn test_hook_picks_up_new_subcommand_added_to_existing_program() {
        // Regression: when a second subcommand is added under the same program (e.g. c:t after
        // c:l), the hook was incorrectly skipping the reload because the set of *program names*
        // hadn't changed ("c" was already in _AM_PROJECT_ALIASES). The wrapper function must
        // be regenerated whenever the file content changes.
        let mut t = TestBed::new()
            .with_aliases("[subcommands]\n\"c:l\" = [\"clippy\"]\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();

        // First run: load c:l, c wrapper is emitted
        let (output, _) = t.run(&Shells::Fish, &cwd, None);
        assert!(
            output.contains("function c"),
            "first run should emit c wrapper"
        );
        assert!(output.contains("clippy"));

        // Add c:t — the .aliases file changes, but program name `c` stays the same
        t.update_aliases("[subcommands]\n\"c:l\" = [\"clippy\"]\n\"c:t\" = [\"test\"]\n");

        // Second run: prev="c" (already loaded), but file has new content
        let (output, _) = t.run(&Shells::Fish, &cwd, Some("c"));
        assert!(
            output.contains("function c"),
            "hook must re-emit c wrapper after new subcommand added, got: {output}"
        );
        assert!(output.contains("test"), "updated wrapper must include c:t");
        assert!(
            output.contains("clippy"),
            "updated wrapper must still include c:l"
        );
    }

    #[test]
    fn test_hook_with_project_subcommands() {
        let mut t = TestBed::new()
            .with_aliases(
                "[aliases]\nb = \"make build\"\n\n[subcommands]\n\"jj:ab\" = [\"abandon\"]\n",
            )
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shells::Bash, &cwd, None);
        assert!(output.contains("b() { make build \"$@\"; }"));
        assert!(output.contains("jj() {"));
        assert!(output.contains("ab) shift; command jj abandon"));
    }
}
