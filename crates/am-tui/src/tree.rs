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
// build_tree_from_parts — one unified tree
// ---------------------------------------------------------------------------

/// Build a single connected tree where global, project, and root profiles
/// are all siblings connected by `├─`/`╰─` trunk connectors.
pub fn build_tree_from_parts(
    global_aliases: &AliasSet,
    profiles: &ProfileConfig,
    active_profile: Option<&str>,
    project: Option<&ProjectAliases>,
) -> Vec<TreeNode> {
    let mut nodes: Vec<TreeNode> = Vec::new();

    let has_project = project.is_some_and(|p| !p.aliases.is_empty());

    // Build profile hierarchy info.
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

    // Global is the ROOT of the entire tree. Everything else is a child of global.
    // Children of global: global aliases, project (if present), root profiles.

    // Count children of global root.
    let global_alias_count = global_aliases.iter().count();
    let child_count = global_alias_count + if has_project { 1 } else { 0 } + roots.len();

    // Global root — no prefix (it IS the root).
    nodes.push(TreeNode {
        kind: NodeKind::GlobalHeader,
        depth: 0,
        alias_id: None,
        alias_command: None,
        is_active: false,
        label: "global".to_string(),
        prefix: String::new(),
        content_prefix: String::new(),
    });

    let mut child_idx = 0;

    // Global aliases are direct children of global root.
    for (name, alias) in global_aliases.iter() {
        let is_last = child_idx == child_count - 1;
        let connector = if is_last { "╰─" } else { "├─" };
        let content_prefix = if is_last { "  " } else { "│ " };
        nodes.push(TreeNode {
            kind: NodeKind::AliasItem,
            depth: 1,
            alias_id: Some(AliasId::Global { alias_name: name.to_string() }),
            alias_command: Some(alias.command().to_string()),
            is_active: false,
            label: name.to_string(),
            prefix: String::new(),
            content_prefix: content_prefix.to_string(),
        });
        child_idx += 1;
    }

    // Project is a child of global root.
    if has_project {
        let proj = project.unwrap();
        let is_last = child_idx == child_count - 1;
        let connector = if is_last { "╰─" } else { "├─" };
        let content_prefix = if is_last { "  " } else { "│ " };

        nodes.push(TreeNode {
            kind: NodeKind::ProjectHeader,
            depth: 1,
            alias_id: None,
            alias_command: None,
            is_active: false,
            label: "project (.aliases)".to_string(),
            prefix: connector.to_string(),
            content_prefix: content_prefix.to_string(),
        });
        for (name, alias) in proj.aliases.iter() {
            nodes.push(TreeNode {
                kind: NodeKind::AliasItem,
                depth: 2,
                alias_id: Some(AliasId::Project { alias_name: name.to_string() }),
                alias_command: Some(alias.command().to_string()),
                is_active: false,
                label: name.to_string(),
                prefix: String::new(),
                content_prefix: content_prefix.to_string(),
            });
        }
        child_idx += 1;
    }

    // Root profiles are children of global root.
    for (i, root) in roots.iter().enumerate() {
        let is_last = child_idx + i == child_count - 1;
        let connector = if is_last { "╰─" } else { "├─" };
        let child_cp = if is_last { "  " } else { "│ " };

        emit_profile_node(
            root,
            &children_of,
            active_profile,
            profiles,
            1,
            connector,
            child_cp,
            &mut nodes,
        );
    }

    nodes
}

/// Emit a profile header + its aliases + recurse into children.
/// `header_connector` is the `├─` or `╰─` for this node's header line.
/// `content_prefix` is the prefix for lines underneath (aliases, child connectors).
fn emit_profile_node(
    profile_name: &str,
    children_of: &std::collections::BTreeMap<&str, Vec<&str>>,
    active_profile: Option<&str>,
    profiles: &ProfileConfig,
    depth: u16,
    header_connector: &str,
    content_prefix: &str,
    nodes: &mut Vec<TreeNode>,
) {
    let is_active = active_profile == Some(profile_name);
    let kids = children_of.get(profile_name).cloned().unwrap_or_default();

    // The node's content_prefix is the continuation prefix passed from the parent.
    // This is what vertical lines under this node use.
    nodes.push(TreeNode {
        kind: NodeKind::ProfileHeader,
        depth,
        alias_id: None,
        alias_command: None,
        is_active,
        label: profile_name.to_string(),
        prefix: header_connector.to_string(),
        content_prefix: content_prefix.to_string(),
    });

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
                content_prefix: content_prefix.to_string(),
            });
        }
    }

    for (i, child) in kids.iter().enumerate() {
        let is_last = i == kids.len() - 1;
        let connector = if is_last {
            format!("{content_prefix}╰─")
        } else {
            format!("{content_prefix}├─")
        };
        let child_cp = if is_last {
            format!("{content_prefix}  ")
        } else {
            format!("{content_prefix}│ ")
        };

        emit_profile_node(
            child,
            children_of,
            active_profile,
            profiles,
            depth + 1,
            &connector,
            &child_cp,
            nodes,
        );
    }
}

// ---------------------------------------------------------------------------
// build_dest_tree_from_parts — headers only, same unified structure
// ---------------------------------------------------------------------------

pub fn build_dest_tree_from_parts(
    _global_aliases: &AliasSet,
    profiles: &ProfileConfig,
    active_profile: Option<&str>,
    has_project: bool,
) -> Vec<TreeNode> {
    let mut nodes: Vec<TreeNode> = Vec::new();

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

    // Global is the root — same structure as main tree but headers only.
    let child_count = if has_project { 1 } else { 0 } + roots.len();

    nodes.push(TreeNode {
        kind: NodeKind::GlobalHeader,
        depth: 0,
        alias_id: None, alias_command: None, is_active: false,
        label: "global".to_string(),
        prefix: String::new(),
        content_prefix: String::new(),
    });

    let mut child_idx = 0;

    // Project
    if has_project {
        let is_last = child_idx == child_count - 1;
        let connector = if is_last { "╰─" } else { "├─" };
        let cp = if is_last { "  " } else { "│ " };
        nodes.push(TreeNode {
            kind: NodeKind::ProjectHeader,
            depth: 1,
            alias_id: None, alias_command: None, is_active: false,
            label: "project (.aliases)".to_string(),
            prefix: connector.to_string(),
            content_prefix: cp.to_string(),
        });
        child_idx += 1;
    }

    // Root profiles
    for (i, root) in roots.iter().enumerate() {
        let is_last = child_idx + i == child_count - 1;
        let connector = if is_last { "╰─" } else { "├─" };
        let child_cp = if is_last { "  " } else { "│ " };

        emit_dest_profile_node(
            root, &children_of, active_profile, 1,
            connector, child_cp, &mut nodes,
        );
    }

    nodes
}

fn emit_dest_profile_node(
    profile_name: &str,
    children_of: &std::collections::BTreeMap<&str, Vec<&str>>,
    active_profile: Option<&str>,
    depth: u16,
    header_connector: &str,
    content_prefix: &str,
    nodes: &mut Vec<TreeNode>,
) {
    let is_active = active_profile == Some(profile_name);
    let kids = children_of.get(profile_name).cloned().unwrap_or_default();

    nodes.push(TreeNode {
        kind: NodeKind::ProfileHeader,
        depth,
        alias_id: None, alias_command: None, is_active,
        label: profile_name.to_string(),
        prefix: header_connector.to_string(),
        content_prefix: content_prefix.to_string(),
    });

    for (i, child) in kids.iter().enumerate() {
        let is_last = i == kids.len() - 1;
        let connector = if is_last {
            format!("{content_prefix}╰─")
        } else {
            format!("{content_prefix}├─")
        };
        let child_cp = if is_last {
            format!("{content_prefix}  ")
        } else {
            format!("{content_prefix}│ ")
        };
        emit_dest_profile_node(
            child, children_of, active_profile, depth + 1,
            &connector, &child_cp, nodes,
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
        // Global is always the root with no prefix
        let tree = build_tree_from_parts(&AliasSet::default(), &ProfileConfig::default(), None, None);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].kind, NodeKind::GlobalHeader);
        assert_eq!(tree[0].prefix, "");
    }

    #[test]
    fn test_build_tree_global_only() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let tree = build_tree_from_parts(&config.aliases, &ProfileConfig::default(), None, None);
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].kind, NodeKind::GlobalHeader);
        assert_eq!(tree[0].prefix, ""); // root, no connector
        assert_eq!(tree[1].kind, NodeKind::AliasItem);
        assert_eq!(tree[1].label, "ll");
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
    fn test_build_tree_all_connected() {
        let mut config = Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let profiles: ProfileConfig = toml::from_str(r#"
            [[profiles]]
            name = "rust"
        "#).unwrap();
        let mut project = ProjectAliases::default();
        project.add_alias("t".into(), "cargo test".into(), false);
        let tree = build_tree_from_parts(&config.aliases, &profiles, None, Some(&project));

        // Global = root (no prefix)
        assert_eq!(tree[0].prefix, "");
        assert_eq!(tree[0].kind, NodeKind::GlobalHeader);

        // ll alias = child of global, ├─ (project + rust follow)
        let ll = tree.iter().find(|n| n.label == "ll").unwrap();
        assert_eq!(ll.kind, NodeKind::AliasItem);

        // Project = child of global, ├─ (rust follows)
        let proj = tree.iter().find(|n| n.kind == NodeKind::ProjectHeader).unwrap();
        assert_eq!(proj.prefix, "├─");

        // Rust = child of global, ╰─ (last child)
        let rust = tree.iter().find(|n| n.kind == NodeKind::ProfileHeader).unwrap();
        assert_eq!(rust.prefix, "╰─");
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
