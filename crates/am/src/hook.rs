use std::path::Path;

use crate::project::ProjectAliases;
use crate::shell::Shells;
use crate::TomlAlias;

/// Generate shell code for the cd hook.
///
/// `cwd` — the current working directory to search for `.aliases`.
/// `previous_aliases` — comma-separated alias names from `_AM_PROJECT_ALIASES` env var.
pub fn generate_hook(
    shell: &Shells,
    cwd: &Path,
    previous_aliases: Option<&str>,
) -> crate::Result<String> {
    let shell_impl = shell.clone().as_shell();
    let mut lines: Vec<String> = Vec::new();

    // Unload previous project aliases
    let prev: Vec<&str> = previous_aliases
        .filter(|s| !s.is_empty())
        .map(|s| s.split(',').collect())
        .unwrap_or_default();

    for alias_name in &prev {
        lines.push(shell_impl.unalias(alias_name));
    }

    // Load new project aliases
    let project = ProjectAliases::find(cwd)?;

    match project {
        Some(project) if !project.aliases.is_empty() => {
            let mut names: Vec<String> = Vec::new();
            for (alias_name, alias_value) in project.aliases.iter() {
                let name = alias_name.as_ref();
                let cmd = match alias_value {
                    TomlAlias::Command(cmd) => cmd.as_str(),
                    TomlAlias::Detailed(detail) => detail.command.as_str(),
                };
                lines.push(shell_impl.alias(name, cmd));
                names.push(name.to_string());
            }
            let names_csv = names.join(",");
            lines.push(shell_impl.set_env("_AM_PROJECT_ALIASES", &names_csv));
        }
        _ => {
            // No project aliases — clear the tracking env var if it was set
            if !prev.is_empty() {
                lines.push(shell_impl.unset_env("_AM_PROJECT_ALIASES"));
            }
        }
    }

    Ok(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_hook_with_aliases_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nb = \"make build\"\nt = \"make test\"\n",
        )
        .unwrap();

        let output = generate_hook(&Shells::Fish, dir.path(), None).unwrap();
        assert!(output.contains("alias b \"make build\""));
        assert!(output.contains("alias t \"make test\""));
        assert!(output.contains("_AM_PROJECT_ALIASES"));
    }

    #[test]
    fn test_hook_unloads_previous_aliases() {
        let dir = tempfile::tempdir().unwrap();
        // no .aliases file, but previous aliases exist
        let output = generate_hook(&Shells::Fish, dir.path(), Some("old1,old2")).unwrap();
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("set -e _AM_PROJECT_ALIASES"));
    }

    #[test]
    fn test_hook_no_aliases_no_previous() {
        let dir = tempfile::tempdir().unwrap();
        let output = generate_hook(&Shells::Fish, dir.path(), None).unwrap();
        assert!(output.is_empty());
    }

    #[test]
    fn test_hook_transitions_between_projects() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nnew1 = \"echo new\"\n",
        )
        .unwrap();

        let output = generate_hook(&Shells::Fish, dir.path(), Some("old1,old2")).unwrap();
        // should unload old
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        // should load new
        assert!(output.contains("alias new1 \"echo new\""));
        assert!(output.contains("\"new1\""));
    }

    #[test]
    fn test_hook_zsh_output() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nb = \"make build\"\n",
        )
        .unwrap();

        let output = generate_hook(&Shells::Zsh, dir.path(), Some("old")).unwrap();
        assert!(output.contains("unalias old"));
        assert!(output.contains("alias b=\"make build\""));
        assert!(output.contains("export _AM_PROJECT_ALIASES="));
    }
}
