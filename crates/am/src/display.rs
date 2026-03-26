use std::path::Path;

use crate::dirs::relative_path;
use crate::project::{ProjectAliases, ALIASES_FILE};
use crate::{AliasSet, Profile, ProfileConfig};

/// Render profiles + project aliases as a complete listing.
pub fn render_listing(
    global_aliases: &AliasSet,
    config: &ProfileConfig,
    active_profiles: &[String],
    cwd: &Path,
) -> String {
    let mut output = String::new();

    // Global aliases
    if !global_aliases.is_empty() {
        output.push_str("🌐 global");
        for (alias_name, alias_value) in global_aliases.iter() {
            let name = alias_name.as_ref();
            let cmd = alias_value.command();
            output.push_str(&format!("\n  {name} → {cmd}"));
        }
        output.push_str("\n\n");
    }

    output.push_str(&render_profile_tree(config, active_profiles));

    match ProjectAliases::find(cwd) {
        Ok(Some(project)) if !project.aliases.is_empty() => {
            let path = ProjectAliases::find_path(cwd)
                .ok()
                .flatten()
                .map(|p| relative_path(cwd, &p))
                .unwrap_or_else(|| ALIASES_FILE.into());

            output.push_str("\n\n📁 project aliases");
            output.push_str(&format!(" ({})", path.display()));
            for (alias_name, alias_value) in project.aliases.iter() {
                let name = alias_name.as_ref();
                let cmd = alias_value.command();
                output.push_str(&format!("\n  {name} → {cmd}"));
            }
        }
        _ => {}
    }

    output
}

/// Render all profiles as a flat list.
///
/// The active profiles are marked with `●`, others with `○`.
pub fn render_profile_tree(config: &ProfileConfig, active_profiles: &[String]) -> String {
    let profiles: Vec<&Profile> = config.iter().collect();

    let mut lines: Vec<String> = Vec::new();

    for (i, profile) in profiles.iter().enumerate() {
        let is_active = active_profiles.contains(&profile.name);
        let marker = if is_active { "●" } else { "○" };
        let active_tag = if is_active { " (active)" } else { "" };

        lines.push(format!("{marker} {}{active_tag}", profile.name));

        if profile.aliases.is_empty() {
            lines.push("  (no aliases)".to_string());
        } else {
            for (alias_name, alias_value) in profile.aliases.iter() {
                let name = alias_name.as_ref();
                let cmd = alias_value.command();
                lines.push(format!("  {name} → {cmd}"));
            }
        }

        // Blank line between profiles (but not after the last)
        if i < profiles.len() - 1 {
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProfileConfig;
    use indoc::indoc;

    fn make_config(toml_str: &str) -> ProfileConfig {
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_single_profile_active() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "default"
            [profiles.aliases]
            ll = "ls -lha"
        "#});

        let output = render_profile_tree(&config, &["default".to_string()]);
        assert!(output.contains("● default (active)"));
        assert!(output.contains("  ll → ls -lha"));
    }

    #[test]
    fn test_multiple_profiles_some_active() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output =
            render_profile_tree(&config, &["rust".to_string()]);
        assert!(output.contains("○ git"));
        assert!(output.contains("● rust (active)"));
    }

    #[test]
    fn test_empty_profile() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "empty"
        "#});

        let output = render_profile_tree(&config, &["empty".to_string()]);
        assert!(output.contains("● empty (active)"));
        assert!(output.contains("(no aliases)"));
    }

    #[test]
    fn test_separate_profiles() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "default"
            [profiles.aliases]
            ll = "ls -lha"

            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#});

        let output = render_profile_tree(
            &config,
            &["default".to_string()],
        );
        assert!(output.contains("● default (active)"));
        assert!(output.contains("○ git"));
        // Should have blank line between profiles
        assert!(output.contains("\n\n"));
    }

    #[test]
    fn test_multiple_active_profiles() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_profile_tree(
            &config,
            &["git".to_string(), "rust".to_string()],
        );
        assert!(output.contains("● git (active)"));
        assert!(output.contains("● rust (active)"));
    }

    #[test]
    fn test_listing_with_project_aliases() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "default"
            [profiles.aliases]
            ll = "ls -lha"
        "#});

        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".aliases"),
            "[aliases]\nt = \"cargo test\"\n",
        )
        .unwrap();

        let output = render_listing(
            &AliasSet::default(),
            &config,
            &["default".to_string()],
            dir.path(),
        );
        assert!(output.contains("● default (active)"));
        assert!(output.contains("📁 project aliases"));
        assert!(output.contains("t → cargo test"));
    }

    #[test]
    fn test_listing_without_project_aliases() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "default"
        "#});

        let dir = tempfile::tempdir().unwrap();
        let output = render_listing(
            &AliasSet::default(),
            &config,
            &["default".to_string()],
            dir.path(),
        );
        assert!(output.contains("● default (active)"));
        assert!(!output.contains("📁"));
    }
}
