use am::update::AppModel;
use am::{AliasSet, ProfileConfig, ProjectAliases};
use crate::model::{AliasId, NodeKind, TreeNode};

// ---------------------------------------------------------------------------
// Public façades
// ---------------------------------------------------------------------------

pub fn build_tree(app_model: &AppModel, project: Option<&ProjectAliases>) -> Vec<TreeNode> {
    build_tree_from_parts(
        &app_model.config.aliases,
        app_model.profile_config(),
        app_model.config.active_profile.as_deref(),
        project,
    )
}

pub fn build_dest_tree(app_model: &AppModel, has_project: bool) -> Vec<TreeNode> {
    build_dest_tree_from_parts(
        &app_model.config.aliases,
        app_model.profile_config(),
        app_model.config.active_profile.as_deref(),
        has_project,
    )
}

// ---------------------------------------------------------------------------
// Testable core — build_tree_from_parts
// ---------------------------------------------------------------------------

/// Build the full alias tree (headers + alias items).
///
/// Order: Global → Project → Profile tree (roots first, children indented).
///
/// Each node is annotated with `prefix` (for its header line) and
/// `content_prefix` (for alias/connector lines beneath it), mirroring the
/// box-drawing approach from the CLI `display.rs`.
pub fn build_tree_from_parts(
    global_aliases: &AliasSet,
    profiles: &ProfileConfig,
    active_profile: Option<&str>,
    project: Option<&ProjectAliases>,
) -> Vec<TreeNode> {
    let mut nodes: Vec<TreeNode> = Vec::new();

    // Determine which top-level sections exist so we know which is "last".
    let has_project = project.is_some_and(|p| !p.aliases.is_empty());
    let has_profiles = profiles.iter().next().is_some();

    // A top-level section's content_prefix uses "│ " if more sections follow,
    // or "  " if it is the last section.
    // Global is always present (sticky at top).
    let global_is_last = !has_project && !has_profiles;
    let project_is_last = has_project && !has_profiles;

    // --- Global section (always visible) ---
    {
        let content_prefix = if global_is_last { "  " } else { "│ " }.to_string();
        nodes.push(TreeNode {
            kind: NodeKind::GlobalHeader,
            depth: 0,
            alias_id: None,
            alias_command: None,
            is_active: false,
            label: "global".to_string(),
            prefix: String::new(),
            content_prefix: content_prefix.clone(),
        });
        for (name, alias) in global_aliases.iter() {
            nodes.push(TreeNode {
                kind: NodeKind::AliasItem,
                depth: 1,
                alias_id: Some(AliasId::Global {
                    alias_name: name.to_string(),
                }),
                alias_command: Some(alias.command().to_string()),
                is_active: false,
                label: name.to_string(),
                prefix: String::new(),
                content_prefix: content_prefix.clone(),
            });
        }
    }

    // --- Project section ---
    if has_project {
        let proj = project.unwrap();
        let content_prefix = if project_is_last { "  " } else { "│ " }.to_string();
        nodes.push(TreeNode {
            kind: NodeKind::ProjectHeader,
            depth: 0,
            alias_id: None,
            alias_command: None,
            is_active: false,
            label: "project".to_string(),
            prefix: String::new(),
            content_prefix: content_prefix.clone(),
        });
        for (name, alias) in proj.aliases.iter() {
            nodes.push(TreeNode {
                kind: NodeKind::AliasItem,
                depth: 1,
                alias_id: Some(AliasId::Project {
                    alias_name: name.to_string(),
                }),
                alias_command: Some(alias.command().to_string()),
                is_active: false,
                label: name.to_string(),
                prefix: String::new(),
                content_prefix: content_prefix.clone(),
            });
        }
    }

    // --- Profile tree ---
    let profile_names: std::collections::HashSet<&str> =
        profiles.iter().map(|p| p.name.as_str()).collect();

    let mut children_of: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    let mut roots: Vec<&str> = Vec::new();

    for profile in profiles.iter() {
        match profile.inherits.as_deref() {
            Some(parent) if profile_names.contains(parent) => {
                children_of.entry(parent).or_default().push(profile.name.as_str());
            }
            _ => roots.push(profile.name.as_str()),
        }
    }

    // Sort roots and each children list for deterministic ordering.
    roots.sort_unstable();
    for children in children_of.values_mut() {
        children.sort_unstable();
    }

    for root in &roots {
        build_profile_nodes(
            root,
            &children_of,
            active_profile,
            profiles,
            0,
            "",   // prefix (root profiles have no connector)
            "",   // content_prefix (root profiles start at column 0)
            true, // is_root
            &mut nodes,
        );
    }

    nodes
}

/// Recursively emit a `ProfileHeader` node for `profile_name`, then its alias
/// items, then recurse into children.
///
/// `header_prefix` is the prefix for the header line itself (e.g. "├─", "╰─",
/// or "" for roots).
/// `parent_content_prefix` is the prefix inherited from the parent for
/// continuation lines.
fn build_profile_nodes(
    profile_name: &str,
    children_of: &std::collections::BTreeMap<&str, Vec<&str>>,
    active_profile: Option<&str>,
    profiles: &ProfileConfig,
    depth: u16,
    header_prefix: &str,
    parent_content_prefix: &str,
    is_root: bool,
    nodes: &mut Vec<TreeNode>,
) {
    let is_active = active_profile == Some(profile_name);
    let kids = children_of
        .get(profile_name)
        .cloned()
        .unwrap_or_default();
    let has_children = !kids.is_empty();

    // The content_prefix for alias lines under this profile header.
    // If the profile has children, we use "│ " to draw the vertical connector;
    // otherwise "  " for clean indentation.
    let content_prefix = if has_children {
        format!("{parent_content_prefix}│ ")
    } else {
        format!("{parent_content_prefix}  ")
    };

    // For root profiles, prefix is empty (no connector).
    // For child profiles, header_prefix already contains the connector.
    let prefix = if is_root {
        String::new()
    } else {
        header_prefix.to_string()
    };

    nodes.push(TreeNode {
        kind: NodeKind::ProfileHeader,
        depth,
        alias_id: None,
        alias_command: None,
        is_active,
        label: profile_name.to_string(),
        prefix,
        content_prefix: content_prefix.clone(),
    });

    // Emit alias items for this profile (own aliases only, not inherited).
    if let Some(profile) = profiles.get_profile_by_name(profile_name) {
        for (name, alias) in profile.aliases.iter() {
            nodes.push(TreeNode {
                kind: NodeKind::AliasItem,
                depth: depth + 1,
                alias_id: Some(AliasId::Profile {
                    profile_name: profile_name.to_string(),
                    alias_name: name.to_string(),
                }),
                alias_command: Some(alias.command().to_string()),
                is_active: false,
                label: name.to_string(),
                prefix: String::new(),
                content_prefix: content_prefix.clone(),
            });
        }
    }

    // Recurse into children with appropriate connectors.
    for (i, child) in kids.iter().enumerate() {
        let is_last = i == kids.len() - 1;
        let connector = if is_last { "╰─" } else { "├─" };
        let child_header_prefix = format!("{parent_content_prefix}{connector}");
        let child_content_prefix = if is_last {
            format!("{parent_content_prefix}  ")
        } else {
            format!("{parent_content_prefix}│ ")
        };

        build_profile_nodes(
            child,
            children_of,
            active_profile,
            profiles,
            depth + 1,
            &child_header_prefix,
            &child_content_prefix,
            false,
            nodes,
        );
    }
}

// ---------------------------------------------------------------------------
// Testable core — build_dest_tree_from_parts (headers only)
// ---------------------------------------------------------------------------

/// Build the destination tree: same structure as `build_tree_from_parts` but
/// no `AliasItem` nodes — only headers.  Global and Project headers always
/// appear (regardless of whether there are aliases).
pub fn build_dest_tree_from_parts(
    _global_aliases: &AliasSet,
    profiles: &ProfileConfig,
    active_profile: Option<&str>,
    has_project: bool,
) -> Vec<TreeNode> {
    let mut nodes: Vec<TreeNode> = Vec::new();

    // Global header always present in dest tree.
    nodes.push(TreeNode {
        kind: NodeKind::GlobalHeader,
        depth: 0,
        alias_id: None,
        alias_command: None,
        is_active: false,
        label: "global".to_string(),
        prefix: String::new(),
        content_prefix: "│ ".to_string(),
    });

    // Project header always present when a project exists.
    if has_project {
        nodes.push(TreeNode {
            kind: NodeKind::ProjectHeader,
            depth: 0,
            alias_id: None,
            alias_command: None,
            is_active: false,
            label: "project".to_string(),
            prefix: String::new(),
            content_prefix: "│ ".to_string(),
        });
    }

    // Profile tree (headers only).
    let profile_names: std::collections::HashSet<&str> =
        profiles.iter().map(|p| p.name.as_str()).collect();

    let mut children_of: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    let mut roots: Vec<&str> = Vec::new();

    for profile in profiles.iter() {
        match profile.inherits.as_deref() {
            Some(parent) if profile_names.contains(parent) => {
                children_of.entry(parent).or_default().push(profile.name.as_str());
            }
            _ => roots.push(profile.name.as_str()),
        }
    }

    roots.sort_unstable();
    for children in children_of.values_mut() {
        children.sort_unstable();
    }

    for root in &roots {
        build_dest_profile_nodes(
            root,
            &children_of,
            active_profile,
            0,
            "",
            "",
            true,
            &mut nodes,
        );
    }

    nodes
}

/// Recursively emit only `ProfileHeader` nodes (no alias items).
fn build_dest_profile_nodes(
    profile_name: &str,
    children_of: &std::collections::BTreeMap<&str, Vec<&str>>,
    active_profile: Option<&str>,
    depth: u16,
    header_prefix: &str,
    parent_content_prefix: &str,
    is_root: bool,
    nodes: &mut Vec<TreeNode>,
) {
    let is_active = active_profile == Some(profile_name);
    let kids = children_of
        .get(profile_name)
        .cloned()
        .unwrap_or_default();
    let has_children = !kids.is_empty();

    let content_prefix = if has_children {
        format!("{parent_content_prefix}│ ")
    } else {
        format!("{parent_content_prefix}  ")
    };

    let prefix = if is_root {
        String::new()
    } else {
        header_prefix.to_string()
    };

    nodes.push(TreeNode {
        kind: NodeKind::ProfileHeader,
        depth,
        alias_id: None,
        alias_command: None,
        is_active,
        label: profile_name.to_string(),
        prefix,
        content_prefix: content_prefix.clone(),
    });

    for (i, child) in kids.iter().enumerate() {
        let is_last = i == kids.len() - 1;
        let connector = if is_last { "╰─" } else { "├─" };
        let child_header_prefix = format!("{parent_content_prefix}{connector}");
        let child_content_prefix = if is_last {
            format!("{parent_content_prefix}  ")
        } else {
            format!("{parent_content_prefix}│ ")
        };

        build_dest_profile_nodes(
            child,
            children_of,
            active_profile,
            depth + 1,
            &child_header_prefix,
            &child_content_prefix,
            false,
            nodes,
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AliasId, NodeKind};
    use am::{AliasSet, Config, ProfileConfig, ProjectAliases};

    #[test]
    fn test_build_tree_empty() {
        // Global header is always present even with no aliases
        let tree = build_tree_from_parts(&AliasSet::default(), &ProfileConfig::default(), None, None);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].kind, NodeKind::GlobalHeader);
    }

    #[test]
    fn test_build_tree_global_only() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let tree = build_tree_from_parts(&config.aliases, &ProfileConfig::default(), None, None);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].kind, NodeKind::GlobalHeader);
        assert_eq!(tree[0].label, "global");
        assert_eq!(tree[1].kind, NodeKind::AliasItem);
        assert_eq!(tree[1].label, "ll");
        assert_eq!(tree[1].alias_command.as_deref(), Some("ls -lha"));
        assert_eq!(tree[1].alias_id, Some(AliasId::Global { alias_name: "ll".into() }));
    }

    #[test]
    fn test_build_tree_profiles_with_inheritance() {
        let profiles: ProfileConfig = toml::from_str(r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "rust"
            inherits = "git"
            [profiles.aliases]
            ct = "cargo test"
        "#).unwrap();

        let tree = build_tree_from_parts(&AliasSet::default(), &profiles, None, None);
        let headers: Vec<_> = tree.iter().filter(|n| n.kind == NodeKind::ProfileHeader).collect();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].label, "git");
        assert_eq!(headers[1].label, "rust");
        assert!(headers[1].depth > headers[0].depth);
    }

    #[test]
    fn test_build_tree_with_project() {
        let mut project = ProjectAliases::default();
        project.add_alias("t".into(), "./x.py test".into(), false);
        let tree = build_tree_from_parts(&AliasSet::default(), &ProfileConfig::default(), None, Some(&project));
        let headers: Vec<_> = tree.iter().filter(|n| n.kind == NodeKind::ProjectHeader).collect();
        assert_eq!(headers.len(), 1);
        let aliases: Vec<_> = tree.iter().filter(|n| n.kind == NodeKind::AliasItem).collect();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].alias_id, Some(AliasId::Project { alias_name: "t".into() }));
    }

    #[test]
    fn test_build_tree_ordering_global_project_profiles() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let profiles: ProfileConfig = toml::from_str(r#"
            [[profiles]]
            name = "rust"
        "#).unwrap();
        let mut project = ProjectAliases::default();
        project.add_alias("t".into(), "cargo test".into(), false);
        let tree = build_tree_from_parts(&config.aliases, &profiles, None, Some(&project));
        let header_kinds: Vec<_> = tree.iter()
            .filter(|n| matches!(n.kind, NodeKind::GlobalHeader | NodeKind::ProjectHeader | NodeKind::ProfileHeader))
            .map(|n| n.kind.clone())
            .collect();
        assert_eq!(header_kinds, vec![NodeKind::GlobalHeader, NodeKind::ProjectHeader, NodeKind::ProfileHeader]);
    }

    #[test]
    fn test_build_dest_tree_headers_only() {
        let profiles: ProfileConfig = toml::from_str(r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "rust"
            inherits = "git"
        "#).unwrap();
        let dest = build_dest_tree_from_parts(&AliasSet::default(), &profiles, None, true);
        assert!(dest.iter().all(|n| n.kind != NodeKind::AliasItem));
        let headers: Vec<_> = dest.iter().filter(|n| n.kind == NodeKind::ProfileHeader).collect();
        assert_eq!(headers.len(), 2);
    }
}
