use amoxide::update::AppModel;
use amoxide::{AliasSet, ProfileConfig, ProjectAliases};
use crate::model::{AliasId, NodeKind, TreeNode};

// ---------------------------------------------------------------------------
// Public facades
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
// Shared helpers
// ---------------------------------------------------------------------------

/// Compute the connector prefix and content prefix for a child node.
fn child_prefixes(parent_cp: &str, is_last: bool) -> (String, String) {
    let connector = if is_last {
        format!("{parent_cp}╰─")
    } else {
        format!("{parent_cp}├─")
    };
    let content = if is_last {
        format!("{parent_cp}  ")
    } else {
        format!("{parent_cp}│ ")
    };
    (connector, content)
}

/// Build the profile hierarchy info from a ProfileConfig.
fn build_profile_hierarchy(profiles: &ProfileConfig) -> (Vec<&str>, std::collections::BTreeMap<&str, Vec<&str>>) {
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
    (roots, children_of)
}

// ---------------------------------------------------------------------------
// build_tree_from_parts
// ---------------------------------------------------------------------------

/// Build the full tree. Structure matches `am l` output:
/// 1. Global header (standalone root) with its aliases indented
/// 2. Profile tree (root profiles as top-level siblings with ├─/╰─)
/// 3. Project header (standalone) with its aliases indented
pub fn build_tree_from_parts(
    global_aliases: &AliasSet,
    profiles: &ProfileConfig,
    active_profile: Option<&str>,
    project: Option<&ProjectAliases>,
) -> Vec<TreeNode> {
    let mut nodes: Vec<TreeNode> = Vec::new();
    let (roots, children_of) = build_profile_hierarchy(profiles);

    // --- 1. Global header (always visible, standalone root) ---
    nodes.push(TreeNode {
        kind: NodeKind::GlobalHeader,
        alias_id: None,
        alias_command: None,
        is_active: false,
        label: "global".to_string(),
        prefix: String::new(),
        content_prefix: "  ".to_string(),
    });
    for (name, alias) in global_aliases.iter() {
        nodes.push(TreeNode {
            kind: NodeKind::AliasItem,
            alias_id: Some(AliasId::Global { alias_name: name.to_string() }),
            alias_command: Some(alias.command().to_string()),
            is_active: false,
            label: name.to_string(),
            prefix: String::new(),
            content_prefix: "  ".to_string(),
        });
    }

    // --- 2. Profile tree (root profiles are top-level siblings) ---
    for (i, root) in roots.iter().enumerate() {
        let is_last_root = i == roots.len() - 1;
        let connector = if is_last_root { "╰─" } else { "├─" };
        let content_prefix = if is_last_root { "  " } else { "│ " };

        emit_profile_node(
            root,
            &children_of,
            active_profile,
            profiles,
            connector,
            content_prefix,
            &mut nodes,
        );
    }

    // --- 3. Project (standalone section at the end) ---
    if let Some(proj) = project {
        if !proj.aliases.is_empty() {
            nodes.push(TreeNode {
                kind: NodeKind::ProjectHeader,
                alias_id: None,
                alias_command: None,
                is_active: false,
                label: "project (.aliases)".to_string(),
                prefix: String::new(),
                content_prefix: "  ".to_string(),
            });
            for (name, alias) in proj.aliases.iter() {
                nodes.push(TreeNode {
                    kind: NodeKind::AliasItem,
                    alias_id: Some(AliasId::Project { alias_name: name.to_string() }),
                    alias_command: Some(alias.command().to_string()),
                    is_active: false,
                    label: name.to_string(),
                    prefix: String::new(),
                    content_prefix: "  ".to_string(),
                });
            }
        }
    }

    nodes
}

/// Emit a profile header + its aliases + recurse into children.
fn emit_profile_node(
    profile_name: &str,
    children_of: &std::collections::BTreeMap<&str, Vec<&str>>,
    active_profile: Option<&str>,
    profiles: &ProfileConfig,
    header_connector: &str,
    content_prefix: &str,
    nodes: &mut Vec<TreeNode>,
) {
    let is_active = active_profile == Some(profile_name);
    let kids = children_of.get(profile_name).cloned().unwrap_or_default();
    let has_children = !kids.is_empty();

    // If this profile has child profiles, its own content needs "│ " to
    // connect to the children below.
    let own_content_prefix = if has_children {
        format!("{content_prefix}│ ")
    } else {
        format!("{content_prefix}  ")
    };

    nodes.push(TreeNode {
        kind: NodeKind::ProfileHeader,
        alias_id: None,
        alias_command: None,
        is_active,
        label: profile_name.to_string(),
        prefix: header_connector.to_string(),
        content_prefix: own_content_prefix.clone(),
    });

    if let Some(profile) = profiles.get_profile_by_name(profile_name) {
        for (name, alias) in profile.aliases.iter() {
            nodes.push(TreeNode {
                kind: NodeKind::AliasItem,
                alias_id: Some(AliasId::Profile {
                    profile_name: profile_name.to_string(),
                    alias_name: name.to_string(),
                }),
                alias_command: Some(alias.command().to_string()),
                is_active: false,
                label: name.to_string(),
                prefix: String::new(),
                content_prefix: own_content_prefix.clone(),
            });
        }
    }

    for (i, child) in kids.iter().enumerate() {
        let (connector, child_cp) = child_prefixes(content_prefix, i == kids.len() - 1);
        emit_profile_node(
            child, children_of, active_profile, profiles,
            &connector, &child_cp, nodes,
        );
    }
}

// ---------------------------------------------------------------------------
// build_dest_tree_from_parts — headers only, same structure
// ---------------------------------------------------------------------------

pub fn build_dest_tree_from_parts(
    _global_aliases: &AliasSet,
    profiles: &ProfileConfig,
    active_profile: Option<&str>,
    has_project: bool,
) -> Vec<TreeNode> {
    let mut nodes: Vec<TreeNode> = Vec::new();
    let (roots, children_of) = build_profile_hierarchy(profiles);

    // Global header (always present)
    nodes.push(TreeNode {
        kind: NodeKind::GlobalHeader,
        alias_id: None, alias_command: None, is_active: false,
        label: "global".to_string(),
        prefix: String::new(),
        content_prefix: "  ".to_string(),
    });

    // Profile headers (same structure, no aliases)
    for (i, root) in roots.iter().enumerate() {
        let is_last = i == roots.len() - 1;
        let connector = if is_last { "╰─" } else { "├─" };
        let cp = if is_last { "  " } else { "│ " };
        emit_dest_profile_node(
            root, &children_of, active_profile,
            connector, cp, &mut nodes,
        );
    }

    // Project header (if applicable)
    if has_project {
        nodes.push(TreeNode {
            kind: NodeKind::ProjectHeader,
            alias_id: None, alias_command: None, is_active: false,
            label: "project (.aliases)".to_string(),
            prefix: String::new(),
            content_prefix: "  ".to_string(),
        });
    }

    nodes
}

fn emit_dest_profile_node(
    profile_name: &str,
    children_of: &std::collections::BTreeMap<&str, Vec<&str>>,
    active_profile: Option<&str>,
    header_connector: &str,
    content_prefix: &str,
    nodes: &mut Vec<TreeNode>,
) {
    let is_active = active_profile == Some(profile_name);
    let kids = children_of.get(profile_name).cloned().unwrap_or_default();

    nodes.push(TreeNode {
        kind: NodeKind::ProfileHeader,
        alias_id: None, alias_command: None, is_active,
        label: profile_name.to_string(),
        prefix: header_connector.to_string(),
        content_prefix: content_prefix.to_string(),
    });

    for (i, child) in kids.iter().enumerate() {
        let (connector, child_cp) = child_prefixes(content_prefix, i == kids.len() - 1);
        emit_dest_profile_node(
            child, children_of, active_profile,
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
    use amoxide::{Config, ProfileConfig, ProjectAliases};

    struct TestConfigBuilder {
        config: Config,
        profiles_toml: String,
        has_aliases_header: bool,
        project: Option<ProjectAliases>,
    }

    impl TestConfigBuilder {
        fn new() -> Self {
            Self {
                config: Config::default(),
                profiles_toml: String::new(),
                has_aliases_header: false,
                project: None,
            }
        }

        fn global_alias(mut self, name: &str, cmd: &str) -> Self {
            self.config.add_alias(name.into(), cmd.into(), false);
            self
        }

        fn profile(mut self, name: &str) -> Self {
            self.has_aliases_header = false;
            self.profiles_toml.push_str(&format!("\n[[profiles]]\nname = \"{name}\"\n"));
            self
        }

        fn profile_with_parent(mut self, name: &str, parent: &str) -> Self {
            self.has_aliases_header = false;
            self.profiles_toml.push_str(&format!(
                "\n[[profiles]]\nname = \"{name}\"\ninherits = \"{parent}\"\n"
            ));
            self
        }

        fn alias(mut self, name: &str, cmd: &str) -> Self {
            if !self.has_aliases_header {
                self.profiles_toml.push_str("[profiles.aliases]\n");
                self.has_aliases_header = true;
            }
            self.profiles_toml.push_str(&format!("{name} = \"{cmd}\"\n"));
            self
        }

        fn project_alias(mut self, name: &str, cmd: &str) -> Self {
            self.project
                .get_or_insert_with(ProjectAliases::default)
                .add_alias(name.into(), cmd.into(), false);
            self
        }

        fn build_tree(&self) -> Vec<TreeNode> {
            let profiles: ProfileConfig = if self.profiles_toml.is_empty() {
                ProfileConfig::default()
            } else {
                toml::from_str(&self.profiles_toml).unwrap()
            };
            build_tree_from_parts(
                &self.config.aliases, &profiles, None, self.project.as_ref(),
            )
        }

        fn build_dest_tree(&self) -> Vec<TreeNode> {
            let profiles: ProfileConfig = if self.profiles_toml.is_empty() {
                ProfileConfig::default()
            } else {
                toml::from_str(&self.profiles_toml).unwrap()
            };
            build_dest_tree_from_parts(
                &self.config.aliases, &profiles, None, self.project.is_some(),
            )
        }
    }

    #[test]
    fn test_build_tree_empty() {
        let tree = TestConfigBuilder::new().build_tree();
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].kind, NodeKind::GlobalHeader);
        assert_eq!(tree[0].prefix, "");
    }

    #[test]
    fn test_build_tree_global_only() {
        let tree = TestConfigBuilder::new()
            .global_alias("ll", "ls -lha")
            .build_tree();
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].kind, NodeKind::GlobalHeader);
        assert_eq!(tree[1].kind, NodeKind::AliasItem);
        assert_eq!(tree[1].label, "ll");
        assert_eq!(tree[1].alias_id, Some(AliasId::Global { alias_name: "ll".into() }));
    }

    #[test]
    fn test_build_tree_profiles_with_inheritance() {
        let tree = TestConfigBuilder::new()
            .profile("git").alias("gs", "git status")
            .profile_with_parent("rust", "git").alias("ct", "cargo test")
            .build_tree();
        let headers: Vec<_> = tree.iter().filter(|n| n.kind == NodeKind::ProfileHeader).collect();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].label, "git");
        assert_eq!(headers[1].label, "rust");
        // rust is a child of git — its prefix is longer (nested connector)
        assert!(headers[1].prefix.len() > headers[0].prefix.len());
    }

    #[test]
    fn test_build_tree_with_project() {
        let tree = TestConfigBuilder::new()
            .project_alias("t", "./x.py test")
            .build_tree();
        let headers: Vec<_> = tree.iter().filter(|n| n.kind == NodeKind::ProjectHeader).collect();
        assert_eq!(headers.len(), 1);
        let aliases: Vec<_> = tree.iter().filter(|n| n.kind == NodeKind::AliasItem).collect();
        assert_eq!(aliases.len(), 1);
        assert_eq!(aliases[0].alias_id, Some(AliasId::Project { alias_name: "t".into() }));
    }

    #[test]
    fn test_build_tree_ordering_global_profiles_project() {
        let tree = TestConfigBuilder::new()
            .global_alias("ll", "ls -lha")
            .project_alias("t", "cargo test")
            .profile("rust")
            .build_tree();
        let header_kinds: Vec<_> = tree.iter()
            .filter(|n| matches!(n.kind, NodeKind::GlobalHeader | NodeKind::ProjectHeader | NodeKind::ProfileHeader))
            .map(|n| n.kind.clone())
            .collect();
        // Order: global, then profiles, then project (matches am l)
        assert_eq!(header_kinds, vec![NodeKind::GlobalHeader, NodeKind::ProfileHeader, NodeKind::ProjectHeader]);
    }

    #[test]
    fn test_build_tree_profiles_are_siblings_not_children_of_global() {
        let tree = TestConfigBuilder::new()
            .global_alias("ll", "ls -lha")
            .profile("rust")
            .build_tree();

        // Global is standalone root — no connector prefix
        let global = &tree[0];
        assert_eq!(global.prefix, "");

        // Rust is a root profile — gets ╰─ (last and only root profile)
        let rust = tree.iter().find(|n| n.kind == NodeKind::ProfileHeader).unwrap();
        assert_eq!(rust.prefix, "╰─");

        // Global aliases have "  " content_prefix (standalone indentation)
        let ll = tree.iter().find(|n| n.label == "ll").unwrap();
        assert_eq!(ll.content_prefix, "  ");
    }

    #[test]
    fn test_build_tree_multiple_roots_with_children() {
        let tree = TestConfigBuilder::new()
            .global_alias("helo", "echo hello")
            .project_alias("t", "cargo test")
            .profile("foo").alias("sayt", "echo say it")
            .profile("git").alias("gst", "git status")
            .profile_with_parent("node", "git").alias("b", "npm run build")
            .profile_with_parent("rust", "git").alias("f", "cargo fmt")
            .build_tree();

        let foo = tree.iter().find(|n| n.kind == NodeKind::ProfileHeader && n.label == "foo").unwrap();
        let git = tree.iter().find(|n| n.kind == NodeKind::ProfileHeader && n.label == "git").unwrap();
        let node = tree.iter().find(|n| n.kind == NodeKind::ProfileHeader && n.label == "node").unwrap();
        let rust = tree.iter().find(|n| n.kind == NodeKind::ProfileHeader && n.label == "rust").unwrap();

        // foo is NOT last (git follows) — must be ├─
        assert_eq!(foo.prefix, "├─", "foo should not be last root profile");
        // git IS last root — must be ╰─
        assert_eq!(git.prefix, "╰─", "git should be last root profile");
        // git has children, so its content_prefix must contain │
        assert!(git.content_prefix.contains('│'), "git needs │ in content_prefix for children");

        // node is ├─ (not last child of git), rust is ╰─ (last child of git)
        assert!(node.prefix.ends_with("├─"), "node should be ├─ under git");
        assert!(rust.prefix.ends_with("╰─"), "rust should be ╰─ under git");
    }

    #[test]
    fn test_tree_prefix_continuity() {
        let tree = TestConfigBuilder::new()
            .global_alias("helo", "echo hello")
            .project_alias("i", "cargo install")
            .project_alias("l", "cargo clippy")
            .project_alias("t", "cargo test")
            .profile("foo").alias("sayt", "echo say it")
            .profile("git").alias("gcm", "commit -S").alias("gst", "git status")
            .profile_with_parent("node", "git").alias("b", "npm run build").alias("t", "npm run test")
            .profile_with_parent("rust", "git").alias("f", "cargo fmt").alias("l", "cargo clippy").alias("t", "cargo test")
            .build_tree();

        // Profile/Project headers must have non-empty prefix (except global which is root)
        for node in &tree {
            if node.kind == NodeKind::ProfileHeader {
                assert!(!node.prefix.is_empty(), "profile '{}' must have a prefix", node.label);
            }
        }

        // foo must be ├─ (not last, git follows)
        let foo = tree.iter().find(|n| n.label == "foo").unwrap();
        assert!(foo.prefix.ends_with("├─"), "foo should be ├─ but got {:?}", foo.prefix);

        // git must be ╰─ (last root)
        let git = tree.iter().find(|n| n.label == "git" && n.kind == NodeKind::ProfileHeader).unwrap();
        assert!(git.prefix.ends_with("╰─"), "git should be ╰─ but got {:?}", git.prefix);

        // Project is standalone (no connector prefix)
        let proj = tree.iter().find(|n| n.kind == NodeKind::ProjectHeader).unwrap();
        assert_eq!(proj.prefix, "", "project should be standalone with no connector");
    }

    #[test]
    fn test_build_dest_tree_headers_only() {
        let dest = TestConfigBuilder::new()
            .project_alias("t", "cargo test")
            .profile("git").alias("gs", "git status")
            .profile_with_parent("rust", "git")
            .build_dest_tree();
        assert!(dest.iter().all(|n| n.kind != NodeKind::AliasItem));
        let headers: Vec<_> = dest.iter().filter(|n| n.kind == NodeKind::ProfileHeader).collect();
        assert_eq!(headers.len(), 2);
    }
}
