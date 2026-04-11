use std::path::Path;

use crate::trust::ProjectTrust;
use crate::{AliasSet, Profile, ProfileConfig};

/// Display a path as `~/…` when it falls under the user's home directory.
fn display_path(path: &Path) -> String {
    if let Some(home) = crate::dirs::home_dir() {
        if let Ok(rel) = path.strip_prefix(&home) {
            return format!("~/{}", rel.display());
        }
    }
    path.display().to_string()
}

/// Render profiles + project aliases as a complete two-zone listing.
///
/// **Active zone** (connected by tree trunk):
///   global → active profiles (by activation order) → project
///
/// **Inactive zone** (flat, alphabetical):
///   remaining profiles
pub fn render_listing(
    global_aliases: &AliasSet,
    global_subcommands: &crate::subcommand::SubcommandSet,
    config: &ProfileConfig,
    active_profiles: &[String],
    project: Option<&ProjectTrust>,
) -> String {
    let mut output = String::new();

    // Collect active profiles in activation order
    let active_ordered: Vec<&Profile> = active_profiles
        .iter()
        .filter_map(|name| config.get_profile_by_name(name))
        .collect();

    // Collect inactive profiles (alphabetical, already sorted in ProfileConfig)
    let inactive: Vec<&Profile> = config
        .iter()
        .filter(|p| !active_profiles.contains(&p.name))
        .collect();

    // Determine if there are items after global in the active zone
    let has_active_items = !active_ordered.is_empty() || project.is_some();

    // ── Active zone ──────────────────────────────────────────────

    // Global header (always present)
    output.push_str("\u{1f310} global");
    for (alias_name, alias_value) in global_aliases.iter() {
        let name = alias_name.as_ref();
        let cmd = alias_value.command();
        if has_active_items {
            output.push_str(&format!("\n\u{2502} {name} \u{2192} {cmd}"));
        } else {
            output.push_str(&format!("\n  {name} \u{2192} {cmd}"));
        }
    }
    // Global subcommand aliases
    let subcmd_groups = crate::subcommand::group_by_program(global_subcommands);
    for (program, entries) in &subcmd_groups {
        if has_active_items {
            output.push_str(&format!("\n\u{2502} {program} (subcommands):"));
        } else {
            output.push_str(&format!("\n  {program} (subcommands):"));
        }
        for entry in entries {
            let shorts = entry.short_subcommands.join(" ");
            let longs = entry.long_subcommands.join(" ");
            if has_active_items {
                output.push_str(&format!("\n\u{2502}   {shorts} \u{2192} {longs}"));
            } else {
                output.push_str(&format!("\n    {shorts} \u{2192} {longs}"));
            }
        }
    }

    // Blank line under global aliases (trunk continues if more items)
    if has_active_items {
        output.push_str("\n\u{2502}");
    }

    // Active profiles
    for (i, profile) in active_ordered.iter().enumerate() {
        let order = active_profiles
            .iter()
            .position(|n| n == &profile.name)
            .map(|idx| idx + 1)
            .unwrap_or(0);

        let is_last_active_item = i == active_ordered.len() - 1 && project.is_none();

        let connector = if is_last_active_item {
            "\u{2570}\u{2500}"
        } else {
            "\u{251c}\u{2500}"
        };
        let trunk = if is_last_active_item { " " } else { "\u{2502}" };

        output.push_str(&format!(
            "\n{connector}\u{25cf} {} (active: {order})",
            profile.name
        ));

        for (alias_name, alias_value) in profile.aliases.iter() {
            let name = alias_name.as_ref();
            let cmd = alias_value.command();
            output.push_str(&format!("\n{trunk} {name} \u{2192} {cmd}"));
        }

        // Profile subcommand aliases
        let profile_subcmd_groups = crate::subcommand::group_by_program(&profile.subcommands);
        for (program, entries) in &profile_subcmd_groups {
            output.push_str(&format!("\n{trunk} {program} (subcommands):"));
            for entry in entries {
                let shorts = entry.short_subcommands.join(" ");
                let longs = entry.long_subcommands.join(" ");
                output.push_str(&format!("\n{trunk}   {shorts} \u{2192} {longs}"));
            }
        }

        // Blank line after profile (trunk continues if not last)
        if !is_last_active_item {
            output.push_str(&format!("\n{trunk}"));
        }
    }

    // Project aliases (last in active zone)
    if let Some(trust) = &project {
        let path = trust.path();
        match trust {
            ProjectTrust::Trusted(proj, _) => {
                output.push_str(&format!(
                    "\n\u{2570}\u{2500}\u{1f4c1} project ({})",
                    display_path(path)
                ));
                for (alias_name, alias_value) in proj.aliases.iter() {
                    let name = alias_name.as_ref();
                    let cmd = alias_value.command();
                    output.push_str(&format!("\n  {name} \u{2192} {cmd}"));
                }
                // Project subcommand aliases
                let proj_subcmd_groups = crate::subcommand::group_by_program(&proj.subcommands);
                for (program, entries) in &proj_subcmd_groups {
                    output.push_str(&format!("\n  {program} (subcommands):"));
                    for entry in entries {
                        let shorts = entry.short_subcommands.join(" ");
                        let longs = entry.long_subcommands.join(" ");
                        output.push_str(&format!("\n    {shorts} \u{2192} {longs}"));
                    }
                }
            }
            ProjectTrust::Unknown(_) => {
                output.push_str(&format!(
                    "\n\u{2570}\u{2500}\u{1f4c1} project ({})",
                    display_path(path)
                ));
                output.push_str(
                    "\n       \u{26A0} untrusted \u{2014} run 'am trust' to review and allow",
                );
            }
            ProjectTrust::Tampered(_) => {
                output.push_str(&format!(
                    "\n\u{2570}\u{2500}\u{1f4c1} project ({})",
                    display_path(path)
                ));
                output.push_str(
                    "\n       \u{26A0} modified since last trust \u{2014} run 'am trust' to review and allow",
                );
            }
            ProjectTrust::Untrusted(_) => {
                output.push_str(&format!(
                    "\n\u{2570}\u{2500}\u{1f4c1} project ({})",
                    display_path(path)
                ));
                output.push_str(
                    "\n       \u{26A0} blocked \u{2014} run 'am untrust --forget' to reset",
                );
            }
        }
    }

    // ── Inactive zone ────────────────────────────────────────────

    if !inactive.is_empty() {
        output.push('\n');
        for profile in &inactive {
            output.push_str(&format!("\n\u{25cb} {}", profile.name));
            for (alias_name, alias_value) in profile.aliases.iter() {
                let name = alias_name.as_ref();
                let cmd = alias_value.command();
                output.push_str(&format!("\n  {name} \u{2192} {cmd}"));
            }
            // Inactive profile subcommand aliases
            let inactive_subcmd_groups = crate::subcommand::group_by_program(&profile.subcommands);
            for (program, entries) in &inactive_subcmd_groups {
                output.push_str(&format!("\n  {program} (subcommands):"));
                for entry in entries {
                    let shorts = entry.short_subcommands.join(" ");
                    let longs = entry.long_subcommands.join(" ");
                    output.push_str(&format!("\n    {shorts} \u{2192} {longs}"));
                }
            }
            output.push('\n');
        }
    }

    output
}

/// Render profiles as a two-zone display (active zone + inactive zone).
///
/// Active profiles show `● name (active: N)` with activation order.
/// Inactive profiles show `○ name`.
pub fn render_profiles(config: &ProfileConfig, active_profiles: &[String]) -> String {
    // Collect active profiles in activation order
    let active_ordered: Vec<&Profile> = active_profiles
        .iter()
        .filter_map(|name| config.get_profile_by_name(name))
        .collect();

    // Collect inactive profiles (alphabetical, already sorted in ProfileConfig)
    let inactive: Vec<&Profile> = config
        .iter()
        .filter(|p| !active_profiles.contains(&p.name))
        .collect();

    let mut lines: Vec<String> = Vec::new();

    // Active profiles
    for profile in &active_ordered {
        let order = active_profiles
            .iter()
            .position(|n| n == &profile.name)
            .map(|idx| idx + 1)
            .unwrap_or(0);

        lines.push(format!("\u{25cf} {} (active: {order})", profile.name));

        if profile.aliases.is_empty() {
            lines.push("  (no aliases)".to_string());
        } else {
            for (alias_name, alias_value) in profile.aliases.iter() {
                let name = alias_name.as_ref();
                let cmd = alias_value.command();
                lines.push(format!("  {name} \u{2192} {cmd}"));
            }
        }

        lines.push(String::new());
    }

    // Inactive profiles
    for (i, profile) in inactive.iter().enumerate() {
        lines.push(format!("\u{25cb} {}", profile.name));

        if profile.aliases.is_empty() {
            lines.push("  (no aliases)".to_string());
        } else {
            for (alias_name, alias_value) in profile.aliases.iter() {
                let name = alias_name.as_ref();
                let cmd = alias_value.command();
                lines.push(format!("  {name} \u{2192} {cmd}"));
            }
        }

        // Blank line between profiles (but not after the last)
        if i < inactive.len() - 1 {
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::ProjectAliases;
    use crate::ProfileConfig;
    use indoc::indoc;

    fn make_config(toml_str: &str) -> ProfileConfig {
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_active_profile_shows_order() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#});

        let output = render_profiles(&config, &["git".to_string()]);
        assert!(output.contains("● git (active: 1)"));
        assert!(output.contains("  gs → git status"));
    }

    #[test]
    fn test_multiple_active_profiles_ordered() {
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

        let output = render_profiles(&config, &["git".to_string(), "rust".to_string()]);
        assert!(output.contains("● git (active: 1)"));
        assert!(output.contains("● rust (active: 2)"));
    }

    #[test]
    fn test_inactive_profiles_shown_after_active() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "node"
            [profiles.aliases]
            b = "npm run build"

            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_profiles(&config, &["rust".to_string()]);
        // Active first
        assert!(output.contains("● rust (active: 1)"));
        // Inactive alphabetical
        assert!(output.contains("○ git"));
        assert!(output.contains("○ node"));
        // Active should appear before inactive
        let active_pos = output.find("● rust").unwrap();
        let inactive_git_pos = output.find("○ git").unwrap();
        assert!(active_pos < inactive_git_pos);
    }

    #[test]
    fn test_empty_profile_shows_no_aliases() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "empty"
        "#});

        let output = render_profiles(&config, &["empty".to_string()]);
        assert!(output.contains("● empty (active: 1)"));
        assert!(output.contains("(no aliases)"));
    }

    #[test]
    fn test_listing_global_with_trunk() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let mut globals = AliasSet::default();
        globals.insert(
            "ll".into(),
            crate::TomlAlias::Command("ls -lha".to_string()),
        );

        let output = render_listing(
            &globals,
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["rust".to_string()],
            None,
        );
        // Global with trunk
        assert!(output.contains("🌐 global"));
        assert!(output.contains("│ ll → ls -lha"));
        // Active profile with connector
        assert!(output.contains("╰─● rust (active: 1)"));
    }

    #[test]
    fn test_listing_active_profiles_with_project() {
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

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".aliases");
        std::fs::write(&path, "[aliases]\nt = \"cargo test\"\n").unwrap();
        let project = ProjectAliases::load(&path).unwrap();
        let trust = ProjectTrust::Trusted(project, std::path::PathBuf::from(".aliases"));

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["git".to_string(), "rust".to_string()],
            Some(&trust),
        );
        assert!(output.contains("├─● git (active: 1)"));
        assert!(output.contains("├─● rust (active: 2)"));
        assert!(output.contains("╰─📁 project"));
        assert!(output.contains("t → cargo test"));
    }

    #[test]
    fn test_listing_last_active_gets_corner_when_no_project() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["rust".to_string()],
            None,
        );
        assert!(output.contains("╰─● rust (active: 1)"));
    }

    #[test]
    fn test_listing_inactive_profiles_below() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "foo"
            [profiles.aliases]
            sayt = "echo say it"

            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["rust".to_string()],
            None,
        );
        assert!(output.contains("╰─● rust (active: 1)"));
        assert!(output.contains("○ foo"));
        assert!(output.contains("  sayt → echo say it"));
    }

    #[test]
    fn test_listing_global_alone_no_trunk() {
        let config: ProfileConfig = ProfileConfig::default();

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &[],
            None,
        );
        assert!(output.contains("🌐 global"));
        // No trunk when global stands alone
        assert!(!output.contains("│"));
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
        let path = dir.path().join(".aliases");
        std::fs::write(&path, "[aliases]\nt = \"cargo test\"\n").unwrap();
        let project = ProjectAliases::load(&path).unwrap();
        let trust = ProjectTrust::Trusted(project, std::path::PathBuf::from(".aliases"));

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["default".to_string()],
            Some(&trust),
        );
        assert!(output.contains("● default (active: 1)"));
        assert!(output.contains("📁 project"));
        assert!(output.contains("t → cargo test"));
    }

    #[test]
    fn test_listing_without_project_aliases() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "default"
        "#});

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["default".to_string()],
            None,
        );
        assert!(output.contains("● default (active: 1)"));
        assert!(!output.contains("📁"));
    }

    #[test]
    fn test_listing_global_subcommands() {
        use crate::subcommand::SubcommandSet;

        let config: ProfileConfig = ProfileConfig::default();
        let mut subs = SubcommandSet::new();
        subs.insert("jj:ab".into(), vec!["abandon".into()]);
        subs.insert("jj:b:l".into(), vec!["branch".into(), "list".into()]);

        let output = render_listing(&AliasSet::default(), &subs, &config, &[], None);
        assert!(output.contains("jj (subcommands)"));
        assert!(output.contains("ab → abandon"));
        assert!(output.contains("b l → branch list"));
    }
}
