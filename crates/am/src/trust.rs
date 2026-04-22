use std::path::{Path, PathBuf};

use crate::project::ProjectAliases;

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

/// Render the "loaded" info message shown on cd into a trusted directory.
///
/// Alias names are right-padded so `->` and commands align in columns.
/// Subcommand wrapper programs are listed with their short→long expansions.
pub fn render_load_message(
    aliases: &crate::AliasSet,
    subcommands: &crate::subcommand::SubcommandSet,
) -> String {
    let mut lines = vec!["am: loaded .aliases".to_string()];

    // Find max alias name length for column alignment
    let max_name_len = aliases
        .iter()
        .map(|(name, _)| name.as_ref().len())
        .max()
        .unwrap_or(0);

    for (alias_name, alias_value) in aliases.iter() {
        let name = alias_name.as_ref();
        let cmd = alias_value.command();
        let padded = format!("{:width$}", name, width = max_name_len);
        lines.push(format!("  {padded} \u{2192} {cmd}"));
    }

    let subcmd_groups = crate::subcommand::group_by_program(subcommands);
    for (program, entries) in &subcmd_groups {
        lines.push(format!("  {program} (subcommands):"));
        for entry in entries {
            let shorts = entry.short_subcommands.join(" ");
            let longs = entry.long_subcommands.join(" ");
            lines.push(format!("    {shorts} \u{2192} {longs}"));
        }
    }

    lines.join("\n")
}

/// Render the "unloaded" info message shown when leaving a trusted directory.
pub fn render_unload_message(alias_names: &[&str]) -> String {
    format!("am: unloaded .aliases: {}", alias_names.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn render_load_message_columnar_alignment() {
        let aliases = test_aliases();
        let msg = render_load_message(&aliases, &Default::default());
        assert!(msg.starts_with("am: loaded .aliases\n"));
        // All arrows should be at the same column
        let arrow_positions: Vec<usize> = msg
            .lines()
            .skip(1)
            .filter_map(|line| line.find('\u{2192}'))
            .collect();
        assert!(!arrow_positions.is_empty());
        let first = arrow_positions[0];
        assert!(
            arrow_positions.iter().all(|&p| p == first),
            "Arrows not aligned: {arrow_positions:?}"
        );
    }

    #[test]
    fn render_load_message_contains_all_aliases() {
        let aliases = test_aliases();
        let msg = render_load_message(&aliases, &Default::default());
        assert!(msg.contains("make build"));
        assert!(msg.contains("cargo test"));
        assert!(msg.contains("cargo build"));
    }

    #[test]
    fn render_unload_message_comma_separated() {
        let msg = render_unload_message(&["b", "t", "cb"]);
        assert_eq!(msg, "am: unloaded .aliases: b, t, cb");
    }

    #[test]
    fn render_unload_message_single() {
        let msg = render_unload_message(&["b"]);
        assert_eq!(msg, "am: unloaded .aliases: b");
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
