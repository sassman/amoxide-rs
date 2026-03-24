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
pub fn build_tree_from_parts(
    global_aliases: &AliasSet,
    profiles: &ProfileConfig,
    active_profile: Option<&str>,
    project: Option<&ProjectAliases>,
) -> Vec<TreeNode> {
    let mut nodes: Vec<TreeNode> = Vec::new();

    // --- Global section ---
    if !global_aliases.is_empty() {
        nodes.push(TreeNode {
            kind: NodeKind::GlobalHeader,
            depth: 0,
            alias_id: None,
            alias_command: None,
            is_active: false,
            label: "global".to_string(),
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
            });
        }
    }

    // --- Project section ---
    if let Some(proj) = project {
        if !proj.aliases.is_empty() {
            nodes.push(TreeNode {
                kind: NodeKind::ProjectHeader,
                depth: 0,
                alias_id: None,
                alias_command: None,
                is_active: false,
                label: "project".to_string(),
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
                });
            }
        }
    }

    // --- Profile tree ---
    // Build a map: profile_name -> list of child profile names.
    // A "root" profile is one whose `inherits` field is None, or whose
    // parent does not exist in the config.
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
        build_profile_nodes(root, &children_of, active_profile, profiles, 0, &mut nodes);
    }

    nodes
}

/// Recursively emit a `ProfileHeader` node for `profile_name`, then its alias
/// items, then recurse into children — all indented by `depth`.
fn build_profile_nodes(
    profile_name: &str,
    children_of: &std::collections::BTreeMap<&str, Vec<&str>>,
    active_profile: Option<&str>,
    profiles: &ProfileConfig,
    depth: u16,
    nodes: &mut Vec<TreeNode>,
) {
    let is_active = active_profile == Some(profile_name);

    nodes.push(TreeNode {
        kind: NodeKind::ProfileHeader,
        depth,
        alias_id: None,
        alias_command: None,
        is_active,
        label: profile_name.to_string(),
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
            });
        }
    }

    // Recurse into children.
    if let Some(children) = children_of.get(profile_name) {
        for child in children {
            build_profile_nodes(child, children_of, active_profile, profiles, depth + 1, nodes);
        }
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
        build_dest_profile_nodes(root, &children_of, active_profile, 0, &mut nodes);
    }

    nodes
}

/// Recursively emit only `ProfileHeader` nodes (no alias items).
fn build_dest_profile_nodes(
    profile_name: &str,
    children_of: &std::collections::BTreeMap<&str, Vec<&str>>,
    active_profile: Option<&str>,
    depth: u16,
    nodes: &mut Vec<TreeNode>,
) {
    let is_active = active_profile == Some(profile_name);

    nodes.push(TreeNode {
        kind: NodeKind::ProfileHeader,
        depth,
        alias_id: None,
        alias_command: None,
        is_active,
        label: profile_name.to_string(),
    });

    if let Some(children) = children_of.get(profile_name) {
        for child in children {
            build_dest_profile_nodes(child, children_of, active_profile, depth + 1, nodes);
        }
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
        let tree = build_tree_from_parts(&AliasSet::default(), &ProfileConfig::default(), None, None);
        assert!(tree.is_empty());
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
