use std::path::{Path, PathBuf};

use crate::effects::Echo;
use crate::shell::ShellAdapter;
use crate::{project::ProjectAliases, LogVerbosity};

/// Trust state for a discovered project `.aliases` file.
#[derive(Debug)]
pub enum ProjectTrust {
    Trusted(ProjectAliases, PathBuf),
    Untrusted(PathBuf),
    Tampered(PathBuf),
    Unknown(PathBuf),
}

impl ProjectTrust {
    /// Get the path regardless of trust state.
    pub fn path(&self) -> &Path {
        match self {
            ProjectTrust::Trusted(_, p)
            | ProjectTrust::Untrusted(p)
            | ProjectTrust::Tampered(p)
            | ProjectTrust::Unknown(p) => p,
        }
    }

    /// Get the aliases if trusted, None otherwise.
    pub fn aliases(&self) -> Option<&ProjectAliases> {
        match self {
            ProjectTrust::Trusted(aliases, _) => Some(aliases),
            _ => None,
        }
    }

    /// Returns true if trusted.
    pub fn is_trusted(&self) -> bool {
        matches!(self, ProjectTrust::Trusted(..))
    }
}

/// Compute BLAKE3 hash of file contents.
pub fn compute_file_hash(path: &Path) -> crate::Result<String> {
    let contents = std::fs::read(path)?;
    Ok(compute_hash(&contents))
}

/// Compute BLAKE3 hash of byte content.
pub fn compute_hash(content: &[u8]) -> String {
    blake3::hash(content).to_hex().to_string()
}

/// Compute a 7-character short BLAKE3 hash of byte content.
///
/// Returns the first 7 hex characters of the BLAKE3 hash — analogous to
/// git's short SHA. 28 bits gives collision probability <0.2 % for 1 000 items.
pub fn compute_short_hash(content: &[u8]) -> String {
    blake3::hash(content).to_hex()[..7].to_string()
}

/// Render the "loaded" info as individual shell echo statements.
///
/// Returns one `Echo` per visual line, each wrapped in the shell's echo command.
/// This ensures correct cross-platform behavior (PowerShell `Write-Host` vs Unix `printf`).
pub fn render_load_lines(
    aliases: &crate::AliasSet,
    subcommands: &crate::subcommand::SubcommandSet,
    verbosity: &LogVerbosity,
    shell: &dyn ShellAdapter,
) -> Vec<Echo> {
    match verbosity {
        LogVerbosity::Off => vec![Echo::Silent],
        LogVerbosity::Short => {
            let names: Vec<&str> = aliases.iter().map(|(n, _)| n.as_ref()).collect();
            vec![Echo::Line(
                shell.echo(&format!("am: loaded .aliases: {}", names.join(", "))),
            )]
        }
        LogVerbosity::Verbose => {
            let mut lines = vec![Echo::Line(shell.echo("am: loaded .aliases"))];
            let max_len = aliases
                .iter()
                .map(|(name, _)| name.as_ref().len())
                .max()
                .unwrap_or(0);
            for (alias_name, alias_value) in aliases.iter() {
                let name = alias_name.as_ref();
                let cmd = alias_value.command();
                let padded = format!("{:width$}", name, width = max_len);
                lines.push(Echo::Line(
                    shell.echo(&format!("  {padded} \u{2192} {cmd}")),
                ));
            }
            let subcmd_groups = subcommands.group_by_program();
            for (program, entries) in &subcmd_groups {
                lines.push(Echo::Line(
                    shell.echo(&format!("  {program} (subcommands):")),
                ));
                for entry in entries {
                    let shorts = entry.short_subcommands.join(" ");
                    let longs = entry.long_subcommands.join(" ");
                    lines.push(Echo::Line(
                        shell.echo(&format!("    {shorts} \u{2192} {longs}")),
                    ));
                }
            }
            lines
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LogVerbosity;
    use crate::config::ShellsTomlConfig;
    use crate::shell::Shell;
    use crate::{AliasName, AliasSet, TomlAlias};

    fn test_aliases() -> AliasSet {
        let mut set = AliasSet::default();
        set.insert(
            AliasName::from("b"),
            TomlAlias::Command("make build".to_string()),
        );
        set.insert(
            AliasName::from("t"),
            TomlAlias::Command("cargo test".to_string()),
        );
        set.insert(
            AliasName::from("cb"),
            TomlAlias::Command("cargo build".to_string()),
        );
        set
    }

    #[test]
    fn compute_hash_deterministic() {
        let hash1 = compute_hash(b"[aliases]\nb = \"make build\"\n");
        let hash2 = compute_hash(b"[aliases]\nb = \"make build\"\n");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn compute_short_hash_returns_7_chars() {
        let hash = compute_short_hash(b"make build");
        assert_eq!(hash.len(), 7);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn compute_short_hash_deterministic() {
        let h1 = compute_short_hash(b"make build");
        let h2 = compute_short_hash(b"make build");
        assert_eq!(h1, h2);
    }

    #[test]
    fn compute_short_hash_different_content() {
        let h1 = compute_short_hash(b"make build");
        let h2 = compute_short_hash(b"cargo build");
        assert_ne!(h1, h2);
    }

    #[test]
    fn compute_hash_different_content_different_hash() {
        let hash1 = compute_hash(b"[aliases]\nb = \"make build\"\n");
        let hash2 = compute_hash(b"[aliases]\nb = \"make test\"\n");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn compute_file_hash_reads_and_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".aliases");
        std::fs::write(&path, "[aliases]\nb = \"make build\"\n").unwrap();
        let hash = compute_file_hash(&path).unwrap();
        let expected = compute_hash(b"[aliases]\nb = \"make build\"\n");
        assert_eq!(hash, expected);
    }

    fn test_shell() -> Box<dyn crate::shell::ShellAdapter> {
        Shell::Fish.as_shell(&ShellsTomlConfig::default(), Default::default(), Default::default())
    }

    #[test]
    fn render_load_lines_off_returns_silent() {
        let aliases = test_aliases();
        let lines = render_load_lines(
            &aliases,
            &Default::default(),
            &LogVerbosity::Off,
            test_shell().as_ref(),
        );
        assert!(lines.iter().all(|l| matches!(l, crate::Echo::Silent)));
    }

    #[test]
    fn render_load_lines_short_single_line() {
        let aliases = test_aliases();
        let lines = render_load_lines(
            &aliases,
            &Default::default(),
            &LogVerbosity::Short,
            test_shell().as_ref(),
        );
        assert_eq!(lines.len(), 1);
        match &lines[0] {
            crate::Echo::Line(s) => {
                assert!(s.contains("am: loaded .aliases"), "got: {s}");
                assert!(s.contains("b"), "got: {s}");
                assert!(s.contains("t"), "got: {s}");
            }
            _ => panic!("expected Echo::Line"),
        }
    }

    #[test]
    fn render_load_lines_verbose_multi_line() {
        let aliases = test_aliases();
        let lines = render_load_lines(
            &aliases,
            &Default::default(),
            &LogVerbosity::Verbose,
            test_shell().as_ref(),
        );
        // Header + one line per alias (3 aliases in test_aliases)
        assert!(lines.len() >= 4, "expected at least 4 lines, got {}", lines.len());
        let line_strs: Vec<&str> = lines
            .iter()
            .filter_map(|l| match l {
                crate::Echo::Line(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert!(line_strs[0].contains("am: loaded .aliases"));
        assert!(line_strs.iter().any(|s| s.contains("make build")));
        assert!(line_strs.iter().any(|s| s.contains("cargo test")));
    }

    #[test]
    fn project_trust_path_returns_path_for_all_variants() {
        let path = PathBuf::from("/project/.aliases");
        let trusted = ProjectTrust::Trusted(ProjectAliases::default(), path.clone());
        let untrusted = ProjectTrust::Untrusted(path.clone());
        let tampered = ProjectTrust::Tampered(path.clone());
        let unknown = ProjectTrust::Unknown(path.clone());
        assert_eq!(trusted.path(), Path::new("/project/.aliases"));
        assert_eq!(untrusted.path(), Path::new("/project/.aliases"));
        assert_eq!(tampered.path(), Path::new("/project/.aliases"));
        assert_eq!(unknown.path(), Path::new("/project/.aliases"));
    }

    #[test]
    fn project_trust_aliases_only_for_trusted() {
        let path = PathBuf::from("/project/.aliases");
        let trusted = ProjectTrust::Trusted(ProjectAliases::default(), path.clone());
        let untrusted = ProjectTrust::Untrusted(path.clone());
        assert!(trusted.aliases().is_some());
        assert!(untrusted.aliases().is_none());
    }
}
