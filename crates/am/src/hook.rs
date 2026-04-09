use std::path::Path;

use crate::project::ProjectAliases;
use crate::security::SecurityConfig;
use crate::shell::Shells;

/// Generate shell code for the cd hook.
///
/// `cwd` — the current working directory to search for `.aliases`.
/// `previous_aliases` — comma-separated alias names from `_AM_PROJECT_ALIASES` env var.
pub fn generate_hook(
    shell: &Shells,
    cwd: &Path,
    previous_aliases: Option<&str>,
) -> crate::Result<String> {
    let mut security = SecurityConfig::load().unwrap_or_default();
    let (output, _changed) = generate_hook_with_security(shell, cwd, previous_aliases, &mut security)?;
    // Note: security changes (tamper detection) are not persisted from this path.
    // The main update() path handles persistence via Effect::SaveSecurity.
    Ok(output)
}

/// Generate shell code for the cd hook with explicit security config.
///
/// Returns `(shell_code, security_changed)` — `security_changed` is true
/// when a tamper was detected and `security_config` was mutated in memory.
pub fn generate_hook_with_security(
    shell: &Shells,
    cwd: &Path,
    previous_aliases: Option<&str>,
    security_config: &mut SecurityConfig,
) -> crate::Result<(String, bool)> {
    let shell_impl = shell.clone().as_shell();
    let mut lines: Vec<String> = Vec::new();
    let mut security_changed = false;

    // Unload previous project aliases
    let prev: Vec<&str> = previous_aliases
        .filter(|s| !s.is_empty())
        .map(|s| s.split(',').collect())
        .unwrap_or_default();

    for alias_name in &prev {
        lines.push(shell_impl.unalias(alias_name));
    }

    // Find project aliases file
    let project_path = ProjectAliases::find_path(cwd)?;

    match project_path {
        Some(path) => {
            let hash = crate::trust::compute_file_hash(&path)?;
            let status = security_config.check(&path, &hash);

            match status {
                crate::security::TrustStatus::Trusted => {
                    let project = ProjectAliases::load(&path)?;
                    if !project.aliases.is_empty() {
                        // Show load message
                        let load_msg = crate::trust::render_load_message(&project.aliases);
                        for line in load_msg.lines() {
                            lines.push(shell_impl.echo(line));
                        }

                        let mut names: Vec<String> = Vec::new();
                        for (alias_name, alias_value) in project.aliases.iter() {
                            let name = alias_name.as_ref();
                            lines.push(shell_impl.alias(&alias_value.as_entry(name)));
                            names.push(name.to_string());
                        }
                        let names_csv = names.join(",");
                        lines.push(shell_impl.set_env("_AM_PROJECT_ALIASES", &names_csv));
                    }
                }
                crate::security::TrustStatus::Unknown => {
                    lines.push(shell_impl.echo(
                        "am: .aliases found but not trusted. Run 'am trust' to review and allow.",
                    ));
                }
                crate::security::TrustStatus::Untrusted => {
                    // Silent — no output, no aliases
                }
                crate::security::TrustStatus::Tampered => {
                    security_changed = true;
                    lines.push(shell_impl.echo(
                        "am: .aliases was modified since last trusted. Run 'am trust' to review and allow.",
                    ));
                }
            }

            // If not trusted, clear tracking env var if it was set
            if !matches!(status, crate::security::TrustStatus::Trusted) && !prev.is_empty() {
                lines.push(shell_impl.unset_env("_AM_PROJECT_ALIASES"));
            }
        }
        None => {
            // No project aliases — show unload message and clear tracking
            if !prev.is_empty() {
                let unload_msg = crate::trust::render_unload_message(&prev);
                lines.push(shell_impl.echo(&unload_msg));
                lines.push(shell_impl.unset_env("_AM_PROJECT_ALIASES"));
            }
        }
    }

    Ok((lines.join("\n"), security_changed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trust::compute_file_hash;
    use std::fs;

    /// Helper: trust the .aliases file at `path` in the given security config.
    fn trust_file(security: &mut SecurityConfig, path: &std::path::Path) {
        let hash = compute_file_hash(path).unwrap();
        security.trust(path, &hash);
    }

    #[test]
    fn test_hook_with_aliases_file() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\nt = \"make test\"\n").unwrap();

        let mut security = SecurityConfig::default();
        trust_file(&mut security, &aliases_path);

        let (output, _) =
            generate_hook_with_security(&Shells::Fish, dir.path(), None, &mut security).unwrap();
        assert!(output.contains("alias b \"make build\""));
        assert!(output.contains("alias t \"make test\""));
        assert!(output.contains("_AM_PROJECT_ALIASES"));
    }

    #[test]
    fn test_hook_unloads_previous_aliases() {
        let dir = tempfile::tempdir().unwrap();
        // no .aliases file, but previous aliases exist
        let mut security = SecurityConfig::default();
        let (output, _) =
            generate_hook_with_security(&Shells::Fish, dir.path(), Some("old1,old2"), &mut security)
                .unwrap();
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("set -e _AM_PROJECT_ALIASES"));
    }

    #[test]
    fn test_hook_no_aliases_no_previous() {
        let dir = tempfile::tempdir().unwrap();
        let mut security = SecurityConfig::default();
        let (output, _) =
            generate_hook_with_security(&Shells::Fish, dir.path(), None, &mut security).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_hook_transitions_between_projects() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        fs::write(&aliases_path, "[aliases]\nnew1 = \"echo new\"\n").unwrap();

        let mut security = SecurityConfig::default();
        trust_file(&mut security, &aliases_path);

        let (output, _) = generate_hook_with_security(
            &Shells::Fish,
            dir.path(),
            Some("old1,old2"),
            &mut security,
        )
        .unwrap();
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("alias new1 \"echo new\""));
        assert!(output.contains("\"new1\""));
    }

    #[test]
    fn test_hook_zsh_output() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();

        let mut security = SecurityConfig::default();
        trust_file(&mut security, &aliases_path);

        let (output, _) = generate_hook_with_security(
            &Shells::Zsh,
            dir.path(),
            Some("old"),
            &mut security,
        )
        .unwrap();
        assert!(output.contains("unset -f old"));
        assert!(output.contains("b() { make build \"$@\"; }"));
        assert!(output.contains("export _AM_PROJECT_ALIASES="));
    }

    #[test]
    fn test_hook_picks_up_added_alias() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();

        let mut security = SecurityConfig::default();
        trust_file(&mut security, &aliases_path);

        // First hook call — loads b
        let (output, _) =
            generate_hook_with_security(&Shells::Fish, dir.path(), None, &mut security).unwrap();
        assert!(output.contains("alias b \"make build\""));
        assert!(!output.contains("alias t"));

        // User runs `am add -l t "make test"` — file is updated
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\nt = \"make test\"\n").unwrap();
        // Update hash for the new content
        trust_file(&mut security, &aliases_path);

        // Second hook call — should unload old and load both
        let (output, _) =
            generate_hook_with_security(&Shells::Fish, dir.path(), Some("b"), &mut security)
                .unwrap();
        assert!(output.contains("functions -e b"));
        assert!(output.contains("alias b \"make build\""));
        assert!(output.contains("alias t \"make test\""));
        assert!(output.contains("\"b,t\""));
    }

    #[test]
    fn test_hook_picks_up_removed_alias() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\nt = \"make test\"\n").unwrap();

        let mut security = SecurityConfig::default();
        trust_file(&mut security, &aliases_path);

        // First hook — loads both
        let (output, _) =
            generate_hook_with_security(&Shells::Fish, dir.path(), None, &mut security).unwrap();
        assert!(output.contains("alias b"));
        assert!(output.contains("alias t"));

        // User removes t from .aliases
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();
        trust_file(&mut security, &aliases_path);

        // Second hook — should unload old (b,t) and only load b
        let (output, _) =
            generate_hook_with_security(&Shells::Fish, dir.path(), Some("b,t"), &mut security)
                .unwrap();
        assert!(output.contains("functions -e b"));
        assert!(output.contains("functions -e t"));
        assert!(output.contains("alias b \"make build\""));
        assert!(!output.contains("alias t \"make test\""));
        assert!(output.contains("\"b\""));
    }

    #[test]
    fn test_hook_bash_output() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();

        let mut security = SecurityConfig::default();
        trust_file(&mut security, &aliases_path);

        let (output, _) = generate_hook_with_security(
            &Shells::Bash,
            dir.path(),
            Some("old"),
            &mut security,
        )
        .unwrap();
        assert!(output.contains("unset -f old"));
        assert!(output.contains("b() { make build \"$@\"; }"));
        assert!(output.contains("export _AM_PROJECT_ALIASES="));
    }

    #[test]
    fn test_hook_loads_aliases_from_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("src").join("deep");
        fs::create_dir_all(&sub).unwrap();
        let aliases_path = dir.path().join(".aliases");
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\nt = \"make test\"\n").unwrap();

        let mut security = SecurityConfig::default();
        trust_file(&mut security, &aliases_path);

        let (output, _) =
            generate_hook_with_security(&Shells::Fish, &sub, None, &mut security).unwrap();
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
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();

        let hash = compute_file_hash(&aliases_path).unwrap();
        let mut security = SecurityConfig::default();
        security.trust(&aliases_path, &hash);

        let (output, changed) =
            generate_hook_with_security(&Shells::Fish, dir.path(), None, &mut security).unwrap();
        assert!(!changed);
        assert!(output.contains("alias b \"make build\""));
        assert!(output.contains("am: loaded .aliases"));
    }

    #[test]
    fn test_hook_unknown_shows_warning() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nb = \"make build\"\n",
        )
        .unwrap();

        let mut security = SecurityConfig::default();
        let (output, changed) =
            generate_hook_with_security(&Shells::Fish, dir.path(), None, &mut security).unwrap();
        assert!(!changed);
        assert!(!output.contains("alias b"));
        assert!(output.contains("am: .aliases found but not trusted"));
        assert!(output.contains("am trust"));
    }

    #[test]
    fn test_hook_untrusted_silent() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();

        let mut security = SecurityConfig::default();
        security.untrust(&aliases_path);

        let (output, changed) =
            generate_hook_with_security(&Shells::Fish, dir.path(), None, &mut security).unwrap();
        assert!(!changed);
        assert!(!output.contains("alias b"));
        assert!(!output.contains("am:"));
    }

    #[test]
    fn test_hook_tampered_shows_loud_warning() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        fs::write(&aliases_path, "[aliases]\nb = \"make build\"\n").unwrap();

        let mut security = SecurityConfig::default();
        security.trust(&aliases_path, "wrong_hash");

        let (output, changed) =
            generate_hook_with_security(&Shells::Fish, dir.path(), None, &mut security).unwrap();
        assert!(changed);
        assert!(!output.contains("alias b"));
        assert!(output.contains("modified since last trusted"));
    }

    #[test]
    fn test_hook_unload_shows_message() {
        let dir = tempfile::tempdir().unwrap();
        let mut security = SecurityConfig::default();
        let (output, _) =
            generate_hook_with_security(&Shells::Fish, dir.path(), Some("old1,old2"), &mut security)
                .unwrap();
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("am: unloaded .aliases"));
    }
}
