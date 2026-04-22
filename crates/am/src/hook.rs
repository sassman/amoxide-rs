use std::collections::BTreeMap;
use std::path::Path;

use crate::env_vars;
use crate::project::ProjectAliases;
use crate::security::{SecurityConfig, TrustStatus};
use crate::shell::ShellContext;
use crate::trust::{compute_file_hash, compute_short_hash, render_load_message, render_unload_message};

/// Parse `_AM_PROJECT_ALIASES` value: `"name|hash,name|hash,..."` into a map.
/// Falls back to name-only format (no `|`) for backward compat during upgrade.
fn parse_prev_aliases(raw: Option<&str>) -> BTreeMap<String, Option<String>> {
    let mut map = BTreeMap::new();
    let Some(s) = raw.filter(|s| !s.is_empty()) else {
        return map;
    };
    for entry in s.split(',') {
        if let Some((name, hash)) = entry.split_once('|') {
            map.insert(name.to_string(), Some(hash.to_string()));
        } else {
            // Backward compat: name without hash — always triggers reload
            map.insert(entry.to_string(), None);
        }
    }
    map
}

/// Compute a short content hash for a regular alias.
///
/// Hashes the command string which determines the shell-visible behaviour.
fn alias_content_hash(alias: &crate::alias::TomlAlias) -> String {
    compute_short_hash(alias.command().as_bytes())
}

/// Generate shell code for the cd hook.
///
/// `ctx.cwd` — the current working directory to search for `.aliases`.
/// `previous_aliases` — comma-separated alias entries from `_AM_PROJECT_ALIASES` env var.
pub fn generate_hook(ctx: &ShellContext, previous_aliases: Option<&str>) -> crate::Result<String> {
    let mut security = SecurityConfig::load().unwrap_or_default();
    let prev_project_path = std::env::var(env_vars::AM_PROJECT_PATH).ok();
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
    let shell_impl = ctx.shell.clone().as_shell(
        ctx.cfg,
        ctx.external_functions.clone(),
        ctx.external_aliases.clone(),
    );
    let cwd = ctx.cwd;
    let mut lines: Vec<String> = Vec::new();
    let mut security_changed = false;

    let prev = parse_prev_aliases(previous_aliases);

    // `prev_project_path` tracks which .aliases file was last seen, to avoid
    // repeating warnings. It is passed in explicitly rather than read from the
    // environment so that callers (e.g. tests) can control it independently.

    // Helper: unalias only shell-level names (no `:` — subcommand keys like `c:l`
    // are tracked for change detection but are not themselves shell functions).
    let unload_prev_names: Vec<String> = prev
        .keys()
        .filter(|n| !n.contains(':'))
        .cloned()
        .collect();
    let unload_prev = |lines: &mut Vec<String>| {
        for name in &unload_prev_names {
            lines.push(shell_impl.unalias(name));
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
                        let subcmd_groups =
                            crate::subcommand::group_by_program(&project.subcommands);
                        let subcmd_program_names: Vec<String> =
                            subcmd_groups.keys().cloned().collect();

                        // Build current alias map: name -> short content hash
                        let mut current: BTreeMap<String, String> = BTreeMap::new();

                        for (alias_name, alias_value) in project.aliases.iter() {
                            current.insert(
                                alias_name.as_ref().to_string(),
                                alias_content_hash(alias_value),
                            );
                        }

                        // Subcommand program names (shell-level wrapper function)
                        for program in &subcmd_program_names {
                            let entries_str: String = project
                                .subcommands
                                .iter()
                                .filter(|(k, _)| k.starts_with(&format!("{program}:")))
                                .map(|(k, v)| format!("{k}={}", v.join(",")))
                                .collect::<Vec<_>>()
                                .join(";");
                            current.insert(
                                program.clone(),
                                compute_short_hash(entries_str.as_bytes()),
                            );
                        }

                        // Individual subcommand keys for fine-grained tracking
                        for (key, longs) in project.subcommands.iter() {
                            current.insert(
                                key.clone(),
                                compute_short_hash(longs.join(",").as_bytes()),
                            );
                        }

                        // Compute diff against previous state
                        let mut removed: Vec<String> = Vec::new();
                        let mut added: Vec<String> = Vec::new();
                        let mut changed: Vec<String> = Vec::new();

                        for name in prev.keys() {
                            if !current.contains_key(name) {
                                removed.push(name.clone());
                            }
                        }
                        for (name, hash) in &current {
                            match prev.get(name) {
                                None => added.push(name.clone()),
                                Some(prev_hash) => {
                                    // If prev had no hash (backward compat) or hash
                                    // differs -> changed
                                    if prev_hash.as_deref() != Some(hash.as_str()) {
                                        changed.push(name.clone());
                                    }
                                }
                            }
                        }

                        // If nothing changed at all, skip entirely
                        if removed.is_empty() && added.is_empty() && changed.is_empty() {
                            return Ok((String::new(), false));
                        }

                        let is_fresh_load = prev.is_empty();

                        // 1. Unload removed + changed (not unchanged!)
                        for name in removed.iter().chain(changed.iter()) {
                            if !name.contains(':') {
                                lines.push(shell_impl.unalias(name));
                            }
                        }

                        // 2. Show messages
                        if show_messages {
                            if is_fresh_load {
                                // Full load message (same as cd-into-project)
                                for line in render_load_message(
                                    &project.aliases,
                                    &project.subcommands,
                                )
                                .lines()
                                {
                                    lines.push(shell_impl.echo(line));
                                }
                            } else {
                                // Incremental change summary
                                let mut parts = Vec::new();
                                if !added.is_empty() {
                                    parts.push(format!("{} added", added.len()));
                                }
                                if !changed.is_empty() {
                                    parts.push(format!("{} updated", changed.len()));
                                }
                                if !removed.is_empty() {
                                    parts.push(format!("{} removed", removed.len()));
                                }
                                lines.push(shell_impl.echo(&format!(
                                    "am: .aliases changed ({})",
                                    parts.join(", ")
                                )));
                            }
                        }

                        let programs_set: std::collections::BTreeSet<&str> =
                            subcmd_groups.keys().map(|s| s.as_str()).collect();

                        if is_fresh_load {
                            // 3a. Fresh load: emit all aliases
                            for (alias_name, alias_value) in project.aliases.iter() {
                                let name = alias_name.as_ref();
                                if !programs_set.contains(name) {
                                    lines.push(shell_impl.alias(&alias_value.as_entry(name)));
                                }
                            }

                            // All subcommand wrappers
                            for (program, entries) in &subcmd_groups {
                                let base_cmd = project
                                    .aliases
                                    .iter()
                                    .find(|(n, _)| n.as_ref() == program.as_str())
                                    .map(|(_, v)| v.command().to_string())
                                    .unwrap_or_else(|| format!("command {program}"));
                                lines.push(
                                    shell_impl.subcommand_wrapper(program, &base_cmd, entries),
                                );
                            }
                        } else {
                            // 3b. Incremental: only load added + changed aliases
                            for (alias_name, alias_value) in project.aliases.iter() {
                                let name = alias_name.as_ref();
                                if !programs_set.contains(name)
                                    && (added.contains(&name.to_string())
                                        || changed.contains(&name.to_string()))
                                {
                                    lines.push(shell_impl.alias(&alias_value.as_entry(name)));
                                }
                            }

                            // Reload subcommand wrappers if any subcmd was
                            // added/changed/removed
                            let subcmd_changed = added
                                .iter()
                                .chain(changed.iter())
                                .chain(removed.iter())
                                .any(|n| {
                                    n.contains(':') || subcmd_program_names.contains(n)
                                });
                            if subcmd_changed {
                                for (program, entries) in &subcmd_groups {
                                    let base_cmd = project
                                        .aliases
                                        .iter()
                                        .find(|(n, _)| n.as_ref() == program.as_str())
                                        .map(|(_, v)| v.command().to_string())
                                        .unwrap_or_else(|| format!("command {program}"));
                                    lines.push(
                                        shell_impl
                                            .subcommand_wrapper(program, &base_cmd, entries),
                                    );
                                }
                            }
                        }

                        // 4. Update tracking env var with name|hash format
                        let tracking: Vec<String> = current
                            .iter()
                            .map(|(name, hash)| format!("{name}|{hash}"))
                            .collect();
                        lines.push(
                            shell_impl
                                .set_env(env_vars::AM_PROJECT_ALIASES, &tracking.join(",")),
                        );
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
                lines.push(
                    shell_impl.set_env(env_vars::AM_PROJECT_PATH, &path.display().to_string()),
                );
                if !prev.is_empty() {
                    lines.push(shell_impl.unset_env(env_vars::AM_PROJECT_ALIASES));
                }
            } else if prev_project_path.is_some() {
                lines.push(shell_impl.unset_env(env_vars::AM_PROJECT_PATH));
            }
        }
        None => {
            if !prev.is_empty() {
                unload_prev(&mut lines);
                if !quiet {
                    let prev_names: Vec<&str> = unload_prev_names.iter().map(|s| s.as_str()).collect();
                    lines.push(shell_impl.echo(&render_unload_message(&prev_names)));
                }
                lines.push(shell_impl.unset_env(env_vars::AM_PROJECT_ALIASES));
            }
            if prev_project_path.is_some() {
                lines.push(shell_impl.unset_env(env_vars::AM_PROJECT_PATH));
            }
        }
    }

    Ok((lines.join("\n"), security_changed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::Shell;
    use crate::trust::compute_short_hash;
    use std::path::{Path, PathBuf};

    /// Extract the `_AM_PROJECT_ALIASES` value from generated shell code.
    fn extract_prev_aliases(output: &str, shell: &Shell) -> Option<String> {
        let prefix = match shell {
            Shell::Fish => "set -gx _AM_PROJECT_ALIASES \"",
            _ => "export _AM_PROJECT_ALIASES=\"",
        };
        output.lines()
            .find(|l| l.contains("_AM_PROJECT_ALIASES"))
            .and_then(|l| {
                let start = l.find(prefix).map(|i| i + prefix.len())?;
                let end = l[start..].find('"').map(|i| start + i)?;
                Some(l[start..end].to_string())
            })
    }

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

        fn run(&mut self, shell: &Shell, cwd: &Path, prev: Option<&str>) -> (String, bool) {
            use crate::config::ShellsTomlConfig;
            let cfg = ShellsTomlConfig::default();
            let ctx = ShellContext {
                shell,
                cfg: &cfg,
                cwd,
                external_functions: Default::default(),
                external_aliases: Default::default(),
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
        let (output, _) = t.run(&Shell::Fish, &cwd, None);
        assert!(output.contains("alias b \"make build\""));
        assert!(output.contains("alias t \"make test\""));
        assert!(output.contains(env_vars::AM_PROJECT_ALIASES));
    }

    #[test]
    fn test_hook_unloads_previous_aliases() {
        let mut t = TestBed::new().setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Fish, &cwd, Some("old1,old2"));
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("set -e _AM_PROJECT_ALIASES"));
    }

    #[test]
    fn test_hook_no_aliases_no_previous() {
        let mut t = TestBed::new().setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Fish, &cwd, None);
        assert!(output.is_empty());
    }

    #[test]
    fn test_hook_transitions_between_projects() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nnew1 = \"echo new\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Fish, &cwd, Some("old1,old2"));
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("alias new1 \"echo new\""));
        let new1_hash = compute_short_hash(b"echo new");
        assert!(
            output.contains(&format!("\"new1|{new1_hash}\"")),
            "expected new1|hash in env var, got: {output}"
        );
    }

    #[test]
    fn test_hook_zsh_output() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Zsh, &cwd, Some("old"));
        assert!(output.contains("unset -f old"));
        assert!(output.contains("alias b=\"make build\""));
        assert!(output.contains("export _AM_PROJECT_ALIASES="));
    }

    #[test]
    fn test_hook_picks_up_added_alias() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Fish, &cwd, None);
        assert!(output.contains("alias b \"make build\""));
        assert!(!output.contains("alias t"));

        // Extract prev from first run to feed back as realistic input
        let prev = extract_prev_aliases(&output, &Shell::Fish);

        t.update_aliases("[aliases]\nb = \"make build\"\nt = \"make test\"\n");

        let (output, _) = t.run(&Shell::Fish, &cwd, prev.as_deref());
        // b is unchanged so should NOT be unloaded or reloaded
        assert!(
            !output.contains("functions -e b"),
            "unchanged alias b should not be unloaded, got: {output}"
        );
        assert!(
            !output.contains("alias b \"make build\""),
            "unchanged alias b should not be reloaded, got: {output}"
        );
        // t is newly added
        assert!(output.contains("alias t \"make test\""));
        // Env var now contains both with hashes
        let b_hash = compute_short_hash(b"make build");
        let t_hash = compute_short_hash(b"make test");
        assert!(
            output.contains(&format!("b|{b_hash},t|{t_hash}")),
            "expected b|hash,t|hash in env var, got: {output}"
        );
    }

    #[test]
    fn test_hook_picks_up_removed_alias() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\nt = \"make test\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Fish, &cwd, None);
        assert!(output.contains("alias b"));
        assert!(output.contains("alias t"));

        // Extract prev from first run
        let prev = extract_prev_aliases(&output, &Shell::Fish);

        t.update_aliases("[aliases]\nb = \"make build\"\n");

        let (output, _) = t.run(&Shell::Fish, &cwd, prev.as_deref());
        // b is unchanged: should NOT be unloaded or reloaded
        assert!(
            !output.contains("functions -e b"),
            "unchanged alias b should not be unloaded, got: {output}"
        );
        assert!(
            !output.contains("alias b \"make build\""),
            "unchanged alias b should not be reloaded, got: {output}"
        );
        // t is removed: should be unloaded
        assert!(
            output.contains("functions -e t"),
            "removed alias t should be unloaded, got: {output}"
        );
        assert!(!output.contains("alias t \"make test\""));
        // Env var only contains b now
        let b_hash = compute_short_hash(b"make build");
        assert!(
            output.contains(&format!("\"b|{b_hash}\"")),
            "expected b|hash in env var, got: {output}"
        );
    }

    #[test]
    fn test_hook_bash_output() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Bash, &cwd, Some("old"));
        assert!(output.contains("unset -f old"));
        assert!(output.contains("alias b=\"make build\""));
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
        let (output, _) = t.run(&Shell::Fish, &sub, None);
        assert!(
            output.contains("alias b \"make build\""),
            "should load aliases from parent .aliases, got: {output}"
        );
        assert!(output.contains("alias t \"make test\""));
        assert!(output.contains(env_vars::AM_PROJECT_ALIASES));
    }

    // ─── Trust-gated hook tests ─────────────────────────────────────

    #[test]
    fn test_hook_trusted_shows_load_message() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, changed) = t.run(&Shell::Fish, &cwd, None);
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
        let (output, changed) = t.run(&Shell::Fish, &cwd, None);
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
        let (output, changed) = t.run(&Shell::Fish, &cwd, None);
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
        let (output, changed) = t.run(&Shell::Fish, &cwd, None);
        assert!(changed);
        assert!(!output.contains("alias b"));
        assert!(output.contains("modified since last trusted"));
    }

    #[test]
    fn test_hook_unload_shows_message() {
        let mut t = TestBed::new().setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Fish, &cwd, Some("old1,old2"));
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
        let (output, _) = t.run(&Shell::Fish, &sub, None);
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
        let (output, _) = t.run(&Shell::Fish, &sub, None);
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
        let (output, _) = t.run(&Shell::Fish, &cwd, None);
        assert!(
            output.contains("function c"),
            "first run should emit c wrapper"
        );
        assert!(output.contains("clippy"));

        // Extract prev from first run for realistic change detection
        let prev = extract_prev_aliases(&output, &Shell::Fish);

        // Add c:t — the .aliases file changes, but program name `c` stays the same
        t.update_aliases("[subcommands]\n\"c:l\" = [\"clippy\"]\n\"c:t\" = [\"test\"]\n");

        // Second run: prev has c|hash and c:l|hash, but file has new content
        let (output, _) = t.run(&Shell::Fish, &cwd, prev.as_deref());
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
        let (output, _) = t.run(&Shell::Bash, &cwd, None);
        assert!(output.contains("alias b=\"make build\""));
        assert!(output.contains("jj() {"));
        assert!(output.contains("ab) shift; command jj abandon"));
    }

    // ─── Per-alias content hashing ─────────────────────────────────

    #[test]
    fn test_parse_prev_aliases_new_format() {
        let map = parse_prev_aliases(Some("b|abc1234,t|def5678"));
        assert_eq!(map.len(), 2);
        assert_eq!(map["b"], Some("abc1234".to_string()));
        assert_eq!(map["t"], Some("def5678".to_string()));
    }

    #[test]
    fn test_parse_prev_aliases_old_format_backward_compat() {
        let map = parse_prev_aliases(Some("b,t"));
        assert_eq!(map.len(), 2);
        assert_eq!(map["b"], None);
        assert_eq!(map["t"], None);
    }

    #[test]
    fn test_parse_prev_aliases_empty() {
        assert!(parse_prev_aliases(None).is_empty());
        assert!(parse_prev_aliases(Some("")).is_empty());
    }

    #[test]
    fn test_parse_prev_aliases_mixed_format() {
        let map = parse_prev_aliases(Some("b|abc1234,t,gs|fed9876"));
        assert_eq!(map.len(), 3);
        assert_eq!(map["b"], Some("abc1234".to_string()));
        assert_eq!(map["t"], None);
        assert_eq!(map["gs"], Some("fed9876".to_string()));
    }

    #[test]
    fn test_alias_content_hash_deterministic() {
        let alias = crate::TomlAlias::Command("make build".to_string());
        let h1 = alias_content_hash(&alias);
        let h2 = alias_content_hash(&alias);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 7);
    }

    #[test]
    fn test_alias_content_hash_different_commands() {
        let a = crate::TomlAlias::Command("make build".to_string());
        let b = crate::TomlAlias::Command("cargo build".to_string());
        assert_ne!(alias_content_hash(&a), alias_content_hash(&b));
    }

    #[test]
    fn test_hook_reloads_when_alias_value_changes() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Fish, &cwd, None);
        assert!(output.contains("alias b \"make build\""));

        // Extract prev from first run
        let prev = extract_prev_aliases(&output, &Shell::Fish);

        // Update alias value (same name, different command) and re-trust
        t.update_aliases("[aliases]\nb = \"cargo build\"\n");

        // Hook with prev — same name "b" but different command
        let (output, _) = t.run(&Shell::Fish, &cwd, prev.as_deref());
        assert!(
            output.contains("alias b \"cargo build\""),
            "hook must reload when alias value changes, got: {output}"
        );
        // Old value should be unloaded first
        assert!(
            output.contains("functions -e b"),
            "changed alias should be unloaded before reload, got: {output}"
        );
    }

    #[test]
    fn test_hook_skips_unchanged_aliases() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\nt = \"make test\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Fish, &cwd, None);

        // Extract prev from first run
        let prev = extract_prev_aliases(&output, &Shell::Fish);

        // Re-run with same content — should skip entirely
        let (output, _) = t.run(&Shell::Fish, &cwd, prev.as_deref());
        assert!(
            output.is_empty(),
            "unchanged aliases should produce no output, got: {output}"
        );
    }

    #[test]
    fn test_hook_incremental_message_on_change() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\nt = \"make test\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        let (output, _) = t.run(&Shell::Fish, &cwd, None);
        // Fresh load shows full message
        assert!(output.contains("am: loaded .aliases"));

        let prev = extract_prev_aliases(&output, &Shell::Fish);

        // Change t, add x, remove nothing
        t.update_aliases("[aliases]\nb = \"make build\"\nt = \"make test --all\"\nx = \"exit\"\n");

        let (output, _) = t.run(&Shell::Fish, &cwd, prev.as_deref());
        // Incremental message instead of full load
        assert!(
            output.contains("am: .aliases changed"),
            "should show incremental change message, got: {output}"
        );
        assert!(
            !output.contains("am: loaded .aliases"),
            "should not show full load message on incremental change, got: {output}"
        );
    }

    #[test]
    fn test_hook_backward_compat_old_format_triggers_full_reload() {
        let mut t = TestBed::new()
            .with_aliases("[aliases]\nb = \"make build\"\n")
            .with_security_trusted()
            .setup();

        let cwd = t.root();
        // Old format: no hashes
        let (output, _) = t.run(&Shell::Fish, &cwd, Some("b"));
        // Should treat b as "changed" (prev hash is None) and reload
        assert!(
            output.contains("alias b \"make build\""),
            "backward compat: old format should trigger reload, got: {output}"
        );
    }

    #[test]
    fn test_extract_prev_aliases_fish() {
        let output = "set -gx _AM_PROJECT_ALIASES \"b|abc1234,t|def5678\"";
        let prev = extract_prev_aliases(output, &Shell::Fish);
        assert_eq!(prev, Some("b|abc1234,t|def5678".to_string()));
    }

    #[test]
    fn test_extract_prev_aliases_bash() {
        let output = "export _AM_PROJECT_ALIASES=\"b|abc1234,t|def5678\"";
        let prev = extract_prev_aliases(output, &Shell::Bash);
        assert_eq!(prev, Some("b|abc1234,t|def5678".to_string()));
    }
}
