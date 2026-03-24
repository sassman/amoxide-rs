use std::collections::BTreeMap;
use std::path::Path;

use crate::dirs::relative_path;
use crate::project::{ProjectAliases, ALIASES_FILE};
use crate::{AliasSet, Profile, ProfileConfig};

/// Render profiles + project aliases as a complete listing.
pub fn render_listing(
    global_aliases: &AliasSet,
    config: &ProfileConfig,
    active_name: &str,
    cwd: &Path,
) -> String {
    let mut output = String::new();

    // Global aliases
    if !global_aliases.is_empty() {
        output.push_str("global");
        for (alias_name, alias_value) in global_aliases.iter() {
            let name = alias_name.as_ref();
            let cmd = alias_value.command();
            output.push_str(&format!("\n  {name} → {cmd}"));
        }
        output.push_str("\n\n");
    }

    output.push_str(&render_profile_tree(config, active_name));

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

/// Render all profiles as a tree with unicode box-drawing characters.
///
/// Profiles that inherit from another are shown nested under their parent.
/// The active profile is marked with `●`, others with `○`.
pub fn render_profile_tree(config: &ProfileConfig, active_name: &str) -> String {
    let profiles: Vec<&Profile> = config.iter().collect();

    // Build parent -> children mapping
    let mut children_of: BTreeMap<&str, Vec<&Profile>> = BTreeMap::new();
    for profile in &profiles {
        if let Some(ref parent) = profile.inherits {
            // Only treat as child if parent actually exists
            if profiles.iter().any(|p| p.name == *parent) {
                children_of
                    .entry(parent.as_str())
                    .or_default()
                    .push(profile);
            }
        }
    }

    // Find roots: profiles with no inherits, or whose inherits target doesn't exist
    let roots: Vec<&Profile> = profiles
        .iter()
        .filter(|p| {
            p.inherits.is_none()
                || !profiles
                    .iter()
                    .any(|other| Some(&other.name) == p.inherits.as_ref())
        })
        .copied()
        .collect();

    let mut lines: Vec<String> = Vec::new();

    for (i, root) in roots.iter().enumerate() {
        render_node(root, &children_of, active_name, "", true, &mut lines);
        // Blank line between root-level trees (but not after the last)
        if i < roots.len() - 1 {
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

fn render_node(
    profile: &Profile,
    children_of: &BTreeMap<&str, Vec<&Profile>>,
    active_name: &str,
    prefix: &str,
    is_root: bool,
    lines: &mut Vec<String>,
) {
    let is_active = profile.name == active_name;
    let marker = if is_active { "●" } else { "○" };
    let active_tag = if is_active { " (active)" } else { "" };

    let kids = children_of
        .get(profile.name.as_str())
        .cloned()
        .unwrap_or_default();
    let has_children = !kids.is_empty();

    // Print profile header
    lines.push(format!("{prefix}{marker} {}{active_tag}", profile.name));

    // Determine prefix for alias lines
    let alias_prefix = if is_root {
        if has_children {
            format!("{prefix}│ ")
        } else {
            format!("{prefix}  ")
        }
    } else if has_children {
        format!("{prefix}│ ")
    } else {
        format!("{prefix}  ")
    };

    // Print aliases
    if profile.aliases.is_empty() {
        lines.push(format!("{alias_prefix}(no aliases)"));
    } else {
        for (alias_name, alias_value) in profile.aliases.iter() {
            let name = alias_name.as_ref();
            let cmd = alias_value.command();
            lines.push(format!("{alias_prefix}{name} → {cmd}"));
        }
    }

    // Print children
    for (i, child) in kids.iter().enumerate() {
        let is_last = i == kids.len() - 1;
        let connector = if is_last { "╰─" } else { "├─" };
        let child_prefix = if is_last {
            format!("{prefix}  ")
        } else {
            format!("{prefix}│ ")
        };
        // Print the connector line, then render child with its own prefix
        render_child(
            child,
            children_of,
            active_name,
            prefix,
            connector,
            &child_prefix,
            lines,
        );
    }
}

fn render_child(
    profile: &Profile,
    children_of: &BTreeMap<&str, Vec<&Profile>>,
    active_name: &str,
    parent_prefix: &str,
    connector: &str,
    content_prefix: &str,
    lines: &mut Vec<String>,
) {
    let is_active = profile.name == active_name;
    let marker = if is_active { "●" } else { "○" };
    let active_tag = if is_active { " (active)" } else { "" };

    let kids = children_of
        .get(profile.name.as_str())
        .cloned()
        .unwrap_or_default();
    let has_children = !kids.is_empty();

    // Print profile header with connector
    lines.push(format!(
        "{parent_prefix}{connector}{marker} {}{active_tag}",
        profile.name
    ));

    // Determine prefix for alias lines
    let alias_prefix = if has_children {
        format!("{content_prefix}│ ")
    } else {
        format!("{content_prefix}  ")
    };

    // Print aliases
    if profile.aliases.is_empty() {
        lines.push(format!("{alias_prefix}(no aliases)"));
    } else {
        for (alias_name, alias_value) in profile.aliases.iter() {
            let name = alias_name.as_ref();
            let cmd = alias_value.command();
            lines.push(format!("{alias_prefix}{name} → {cmd}"));
        }
    }

    // Recurse into children
    for (i, child) in kids.iter().enumerate() {
        let is_last = i == kids.len() - 1;
        let connector = if is_last { "╰─" } else { "├─" };
        let child_prefix = if is_last {
            format!("{content_prefix}  ")
        } else {
            format!("{content_prefix}│ ")
        };
        render_child(
            child,
            children_of,
            active_name,
            content_prefix,
            connector,
            &child_prefix,
            lines,
        );
    }
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

        let output = render_profile_tree(&config, "default");
        assert!(output.contains("● default (active)"));
        assert!(output.contains("  ll → ls -lha"));
    }

    #[test]
    fn test_inheritance_tree() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "rust"
            inherits = "git"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_profile_tree(&config, "rust");
        let lines: Vec<&str> = output.lines().collect();

        // git is root
        assert_eq!(lines[0], "○ git");
        // git aliases have │ prefix (has children)
        assert_eq!(lines[1], "│ gs → git status");
        // rust is child with connector
        assert_eq!(lines[2], "╰─● rust (active)");
        // rust aliases indented
        assert_eq!(lines[3], "    ct → cargo test");
    }

    #[test]
    fn test_multiple_children() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "node"
            inherits = "git"
            [profiles.aliases]
            nr = "npm run"

            [[profiles]]
            name = "rust"
            inherits = "git"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_profile_tree(&config, "rust");
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines[0], "○ git");
        assert_eq!(lines[1], "│ gs → git status");
        // node is first child (not last)
        assert_eq!(lines[2], "├─○ node");
        assert_eq!(lines[3], "│   nr → npm run");
        // rust is last child
        assert_eq!(lines[4], "╰─● rust (active)");
        assert_eq!(lines[5], "    ct → cargo test");
    }

    #[test]
    fn test_empty_profile() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "empty"
        "#});

        let output = render_profile_tree(&config, "empty");
        assert!(output.contains("● empty (active)"));
        assert!(output.contains("(no aliases)"));
    }

    #[test]
    fn test_separate_root_trees() {
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

        let output = render_profile_tree(&config, "default");
        // Should have blank line between root trees
        assert!(output.contains("● default (active)"));
        assert!(output.contains("○ git"));
        assert!(output.contains("\n\n"));
    }

    #[test]
    fn test_deep_chain() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "base"

            [[profiles]]
            name = "git"
            inherits = "base"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "rust"
            inherits = "git"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_profile_tree(&config, "rust");
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines[0], "○ base");
        assert_eq!(lines[1], "│ (no aliases)");
        assert_eq!(lines[2], "╰─○ git");
        assert_eq!(lines[3], "  │ gs → git status");
        assert_eq!(lines[4], "  ╰─● rust (active)");
        assert_eq!(lines[5], "      ct → cargo test");
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

        let output = render_listing(&AliasSet::default(), &config, "default", dir.path());
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
        let output = render_listing(&AliasSet::default(), &config, "default", dir.path());
        assert!(output.contains("● default (active)"));
        assert!(!output.contains("📁"));
    }
}
