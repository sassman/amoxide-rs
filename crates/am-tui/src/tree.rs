use crate::model::{AliasId, NodeKind, TreeNode, TREE_BRANCH, TREE_LAST, TREE_SPACE, TREE_TRUNK};
use amoxide::update::AppModel;
use amoxide::{AliasSet, ProfileConfig, ProjectAliases};

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectTrustState {
    Trusted,
    Unknown,
    Untrusted,
    Tampered,
}

// ---------------------------------------------------------------------------
// Public facades
// ---------------------------------------------------------------------------

pub fn build_tree(app_model: &AppModel) -> Vec<TreeNode> {
    let project_trust_state = app_model.project_trust().map(|t| match t {
        amoxide::trust::ProjectTrust::Trusted(..) => ProjectTrustState::Trusted,
        amoxide::trust::ProjectTrust::Unknown(..) => ProjectTrustState::Unknown,
        amoxide::trust::ProjectTrust::Untrusted(..) => ProjectTrustState::Untrusted,
        amoxide::trust::ProjectTrust::Tampered(..) => ProjectTrustState::Tampered,
    });
    build_tree_from_parts(
        &app_model.config.aliases,
        app_model.profile_config(),
        &app_model.config.active_profiles,
        app_model.project_aliases(),
        project_trust_state,
    )
}

pub fn build_dest_tree(app_model: &AppModel) -> Vec<TreeNode> {
    let project_is_trusted = app_model.project_trust().is_some_and(|t| t.is_trusted());
    build_dest_tree_from_parts(
        &app_model.config.aliases,
        app_model.profile_config(),
        &app_model.config.active_profiles,
        project_is_trusted,
    )
}

// ---------------------------------------------------------------------------
// build_tree_from_parts — two-zone flat layout
// ---------------------------------------------------------------------------

/// Build the full tree with a two-zone layout:
///
/// **Active zone** (connected by trunk):
///   global → active profiles (in activation order) → project
///
/// **Inactive zone** (no connectors):
///   remaining profiles, alphabetical
pub fn build_tree_from_parts(
    global_aliases: &AliasSet,
    profiles: &ProfileConfig,
    active_profiles: &[String],
    project: Option<&ProjectAliases>,
    project_trust: Option<ProjectTrustState>,
) -> Vec<TreeNode> {
    let mut nodes: Vec<TreeNode> = Vec::new();
    // Show the project header when aliases are available (trusted) OR when a
    // project file was discovered but could not be trusted yet (Unknown,
    // Untrusted, Tampered). This lets the user see and act on the trust state.
    let has_project = project.is_some_and(|p| !p.aliases.is_empty())
        || matches!(
            &project_trust,
            Some(ProjectTrustState::Unknown)
                | Some(ProjectTrustState::Untrusted)
                | Some(ProjectTrustState::Tampered)
        );

    // Partition profiles into active (in activation order) and inactive (alphabetical).
    let active_names: Vec<&str> = active_profiles
        .iter()
        .filter(|name| profiles.get_profile_by_name(name).is_some())
        .map(String::as_str)
        .collect();
    let inactive_names: Vec<&str> = profiles
        .iter()
        .map(|p| p.name.as_str())
        .filter(|name| !active_profiles.contains(&name.to_string()))
        .collect();

    // Active zone: global + global aliases + active profiles + project
    // All connected by trunk lines.
    let active_zone_children =
        global_aliases.iter().count() + active_names.len() + usize::from(has_project);

    // --- Global root ---
    nodes.push(TreeNode {
        kind: NodeKind::GlobalHeader,
        alias_id: None,
        alias_command: None,
        is_active: false,
        label: "global".to_string(),
        prefix: String::new(),
        content_prefix: String::new(),
        project_trust: None,
    });

    let mut child_idx = 0;

    // --- Global aliases ---
    for (name, alias) in global_aliases.iter() {
        let is_last = child_idx == active_zone_children - 1;
        let cp = if is_last { TREE_SPACE } else { TREE_TRUNK };
        nodes.push(TreeNode {
            kind: NodeKind::AliasItem,
            alias_id: Some(AliasId::Global {
                alias_name: name.to_string(),
            }),
            alias_command: Some(alias.command().to_string()),
            is_active: false,
            label: name.to_string(),
            prefix: String::new(),
            content_prefix: cp.to_string(),
            project_trust: None,
        });
        child_idx += 1;
    }

    // --- Active profiles (in activation order) ---
    for (i, profile_name) in active_names.iter().enumerate() {
        let remaining_active = active_names.len() - i - 1;
        let is_last = remaining_active == 0 && !has_project;
        let connector = if is_last { TREE_LAST } else { TREE_BRANCH };
        let cp = if is_last { TREE_SPACE } else { TREE_TRUNK };

        nodes.push(TreeNode {
            kind: NodeKind::ProfileHeader,
            alias_id: None,
            alias_command: None,
            is_active: true,
            label: profile_name.to_string(),
            prefix: connector.to_string(),
            content_prefix: format!("{cp}  "),
            project_trust: None,
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
                    content_prefix: format!("{cp}  "),
                    project_trust: None,
                });
            }
        }
        child_idx += 1;
    }

    // --- Project (last in active zone) ---
    if has_project {
        let connector = TREE_LAST;
        let cp = "    ";

        let project_label = match &project_trust {
            Some(ProjectTrustState::Trusted) | None => "project (.aliases)".to_string(),
            Some(ProjectTrustState::Unknown) => {
                "project (.aliases) \u{26a0} untrusted \u{2014} press 't' to trust".to_string()
            }
            Some(ProjectTrustState::Untrusted) => {
                "project (.aliases) \u{26a0} blocked \u{2014} press 't' to trust".to_string()
            }
            Some(ProjectTrustState::Tampered) => {
                "project (.aliases) \u{26a0} modified since last trust \u{2014} press 't' to re-trust"
                    .to_string()
            }
        };

        nodes.push(TreeNode {
            kind: NodeKind::ProjectHeader,
            alias_id: None,
            alias_command: None,
            is_active: false,
            label: project_label,
            prefix: connector.to_string(),
            content_prefix: cp.to_string(),
            project_trust: project_trust.clone(),
        });
        // Only list aliases when the project is trusted and aliases exist.
        if let Some(proj) = project {
            for (name, alias) in proj.aliases.iter() {
                nodes.push(TreeNode {
                    kind: NodeKind::AliasItem,
                    alias_id: Some(AliasId::Project {
                        alias_name: name.to_string(),
                    }),
                    alias_command: Some(alias.command().to_string()),
                    is_active: false,
                    label: name.to_string(),
                    prefix: String::new(),
                    content_prefix: cp.to_string(),
                    project_trust: None,
                });
            }
        }
    }

    // --- Inactive zone: remaining profiles, no connectors ---
    for profile_name in &inactive_names {
        nodes.push(TreeNode {
            kind: NodeKind::ProfileHeader,
            alias_id: None,
            alias_command: None,
            is_active: false,
            label: profile_name.to_string(),
            prefix: String::new(),
            content_prefix: TREE_SPACE.to_string(),
            project_trust: None,
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
                    content_prefix: TREE_SPACE.to_string(),
                    project_trust: None,
                });
            }
        }
    }

    nodes
}

// ---------------------------------------------------------------------------
// build_dest_tree_from_parts — headers only, same two-zone layout
// ---------------------------------------------------------------------------

pub fn build_dest_tree_from_parts(
    _global_aliases: &AliasSet,
    profiles: &ProfileConfig,
    active_profiles: &[String],
    has_project: bool,
) -> Vec<TreeNode> {
    let mut nodes: Vec<TreeNode> = Vec::new();

    let active_names: Vec<&str> = active_profiles
        .iter()
        .filter(|name| profiles.get_profile_by_name(name).is_some())
        .map(String::as_str)
        .collect();
    let inactive_names: Vec<&str> = profiles
        .iter()
        .map(|p| p.name.as_str())
        .filter(|name| !active_profiles.contains(&name.to_string()))
        .collect();

    let active_zone_children = active_names.len() + usize::from(has_project);

    // Global header (always present, root)
    nodes.push(TreeNode {
        kind: NodeKind::GlobalHeader,
        alias_id: None,
        alias_command: None,
        is_active: false,
        label: "global".to_string(),
        prefix: String::new(),
        content_prefix: String::new(),
        project_trust: None,
    });

    // Active profiles
    for (i, profile_name) in active_names.iter().enumerate() {
        let remaining = active_names.len() - i - 1;
        let is_last = remaining == 0 && !has_project;
        let connector = if is_last { TREE_LAST } else { TREE_BRANCH };
        let cp = if is_last { TREE_SPACE } else { TREE_TRUNK };

        nodes.push(TreeNode {
            kind: NodeKind::ProfileHeader,
            alias_id: None,
            alias_command: None,
            is_active: true,
            label: profile_name.to_string(),
            prefix: connector.to_string(),
            content_prefix: cp.to_string(),
            project_trust: None,
        });
    }

    // Project header
    if has_project {
        nodes.push(TreeNode {
            kind: NodeKind::ProjectHeader,
            alias_id: None,
            alias_command: None,
            is_active: false,
            label: "project (.aliases)".to_string(),
            prefix: TREE_LAST.to_string(),
            content_prefix: TREE_SPACE.to_string(),
            project_trust: None,
        });
    }

    // Inactive profiles
    for profile_name in &inactive_names {
        nodes.push(TreeNode {
            kind: NodeKind::ProfileHeader,
            alias_id: None,
            alias_command: None,
            is_active: false,
            label: profile_name.to_string(),
            prefix: String::new(),
            content_prefix: String::new(),
            project_trust: None,
        });
    }

    nodes
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
        active_profiles: Vec<String>,
    }

    impl TestConfigBuilder {
        fn new() -> Self {
            Self {
                config: Config::default(),
                profiles_toml: String::new(),
                has_aliases_header: false,
                project: None,
                active_profiles: Vec::new(),
            }
        }

        fn global_alias(mut self, name: &str, cmd: &str) -> Self {
            self.config.add_alias(name.into(), cmd.into(), false);
            self
        }

        fn profile(mut self, name: &str) -> Self {
            self.has_aliases_header = false;
            self.profiles_toml
                .push_str(&format!("\n[[profiles]]\nname = \"{name}\"\n"));
            self
        }

        fn alias(mut self, name: &str, cmd: &str) -> Self {
            if !self.has_aliases_header {
                self.profiles_toml.push_str("[profiles.aliases]\n");
                self.has_aliases_header = true;
            }
            self.profiles_toml
                .push_str(&format!("{name} = \"{cmd}\"\n"));
            self
        }

        fn project_alias(mut self, name: &str, cmd: &str) -> Self {
            self.project
                .get_or_insert_with(ProjectAliases::default)
                .add_alias(name.into(), cmd.into(), false);
            self
        }

        fn active(mut self, names: &[&str]) -> Self {
            self.active_profiles = names.iter().map(|s| s.to_string()).collect();
            self
        }

        fn build_tree(&self) -> Vec<TreeNode> {
            let profiles: ProfileConfig = if self.profiles_toml.is_empty() {
                ProfileConfig::default()
            } else {
                toml::from_str(&self.profiles_toml).unwrap()
            };
            build_tree_from_parts(
                &self.config.aliases,
                &profiles,
                &self.active_profiles,
                self.project.as_ref(),
                None,
            )
        }

        fn build_dest_tree(&self) -> Vec<TreeNode> {
            let profiles: ProfileConfig = if self.profiles_toml.is_empty() {
                ProfileConfig::default()
            } else {
                toml::from_str(&self.profiles_toml).unwrap()
            };
            build_dest_tree_from_parts(
                &self.config.aliases,
                &profiles,
                &self.active_profiles,
                self.project.is_some(),
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
        assert_eq!(
            tree[1].alias_id,
            Some(AliasId::Global {
                alias_name: "ll".into()
            })
        );
    }

    #[test]
    fn test_build_tree_active_profiles_appear_in_active_zone() {
        let tree = TestConfigBuilder::new()
            .profile("git")
            .alias("gs", "git status")
            .profile("rust")
            .alias("ct", "cargo test")
            .active(&["rust", "git"])
            .build_tree();

        let headers: Vec<_> = tree
            .iter()
            .filter(|n| n.kind == NodeKind::ProfileHeader)
            .collect();
        assert_eq!(headers.len(), 2);
        // Active profiles appear in activation order: rust first, then git
        assert_eq!(headers[0].label, "rust");
        assert_eq!(headers[1].label, "git");
        assert!(headers[0].is_active);
        assert!(headers[1].is_active);
    }

    #[test]
    fn test_build_tree_inactive_profiles_have_no_connectors() {
        let tree = TestConfigBuilder::new()
            .profile("git")
            .alias("gs", "git status")
            .profile("rust")
            .alias("ct", "cargo test")
            .build_tree();

        let headers: Vec<_> = tree
            .iter()
            .filter(|n| n.kind == NodeKind::ProfileHeader)
            .collect();
        // Both profiles are inactive (no active_profiles set)
        assert_eq!(headers.len(), 2);
        for h in &headers {
            assert!(!h.is_active, "profile {} should be inactive", h.label);
            assert_eq!(
                h.prefix, "",
                "inactive profile should have no connector prefix"
            );
        }
    }

    #[test]
    fn test_build_tree_with_project() {
        let tree = TestConfigBuilder::new()
            .project_alias("t", "./x.py test")
            .build_tree();
        let headers: Vec<_> = tree
            .iter()
            .filter(|n| n.kind == NodeKind::ProjectHeader)
            .collect();
        assert_eq!(headers.len(), 1);
        let aliases: Vec<_> = tree
            .iter()
            .filter(|n| n.kind == NodeKind::AliasItem)
            .collect();
        assert_eq!(aliases.len(), 1);
        assert_eq!(
            aliases[0].alias_id,
            Some(AliasId::Project {
                alias_name: "t".into()
            })
        );
    }

    #[test]
    fn test_build_tree_ordering_global_active_project_inactive() {
        let tree = TestConfigBuilder::new()
            .global_alias("ll", "ls -lha")
            .project_alias("t", "cargo test")
            .profile("rust")
            .profile("node")
            .active(&["rust"])
            .build_tree();
        let header_labels: Vec<_> = tree
            .iter()
            .filter(|n| {
                matches!(
                    n.kind,
                    NodeKind::GlobalHeader | NodeKind::ProjectHeader | NodeKind::ProfileHeader
                )
            })
            .map(|n| n.label.as_str())
            .collect();
        // Order: global, rust (active), project, node (inactive)
        assert_eq!(
            header_labels,
            vec!["global", "rust", "project (.aliases)", "node"]
        );
    }

    #[test]
    fn test_build_tree_active_profiles_have_trunk_connectors() {
        let tree = TestConfigBuilder::new()
            .global_alias("ll", "ls -lha")
            .profile("rust")
            .active(&["rust"])
            .build_tree();

        let rust = tree
            .iter()
            .find(|n| n.kind == NodeKind::ProfileHeader && n.label == "rust")
            .unwrap();
        // Rust is last active profile with no project — gets ╰─
        assert_eq!(rust.prefix, TREE_LAST);
        assert!(rust.is_active);

        // Global alias has trunk continuation since active profile follows
        let ll = tree.iter().find(|n| n.label == "ll").unwrap();
        assert_eq!(ll.content_prefix, TREE_TRUNK);
    }

    #[test]
    fn test_build_tree_multiple_active_profiles() {
        let tree = TestConfigBuilder::new()
            .profile("git")
            .alias("gs", "git status")
            .profile("rust")
            .alias("ct", "cargo test")
            .profile("node")
            .alias("b", "npm run build")
            .active(&["git", "rust"])
            .build_tree();

        let git = tree
            .iter()
            .find(|n| n.kind == NodeKind::ProfileHeader && n.label == "git")
            .unwrap();
        let rust = tree
            .iter()
            .find(|n| n.kind == NodeKind::ProfileHeader && n.label == "rust")
            .unwrap();
        let node = tree
            .iter()
            .find(|n| n.kind == NodeKind::ProfileHeader && n.label == "node")
            .unwrap();

        // git is active, not last — ├─
        assert_eq!(git.prefix, TREE_BRANCH);
        assert!(git.is_active);
        // rust is active, last active, no project — ╰─
        assert_eq!(rust.prefix, TREE_LAST);
        assert!(rust.is_active);
        // node is inactive — no connector
        assert_eq!(node.prefix, "");
        assert!(!node.is_active);
    }

    #[test]
    fn test_build_tree_active_with_project() {
        let tree = TestConfigBuilder::new()
            .project_alias("t", "cargo test")
            .profile("rust")
            .active(&["rust"])
            .build_tree();

        let rust = tree
            .iter()
            .find(|n| n.kind == NodeKind::ProfileHeader && n.label == "rust")
            .unwrap();
        // Rust is last active but project follows — ├─
        assert_eq!(rust.prefix, TREE_BRANCH);

        let proj = tree
            .iter()
            .find(|n| n.kind == NodeKind::ProjectHeader)
            .unwrap();
        // Project is always last in active zone — ╰─
        assert_eq!(proj.prefix, TREE_LAST);
    }

    #[test]
    fn test_build_dest_tree_headers_only() {
        let dest = TestConfigBuilder::new()
            .project_alias("t", "cargo test")
            .profile("git")
            .alias("gs", "git status")
            .profile("rust")
            .active(&["git"])
            .build_dest_tree();
        assert!(dest.iter().all(|n| n.kind != NodeKind::AliasItem));
        let headers: Vec<_> = dest
            .iter()
            .filter(|n| n.kind == NodeKind::ProfileHeader)
            .collect();
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn test_build_dest_tree_active_before_inactive() {
        let dest = TestConfigBuilder::new()
            .profile("git")
            .profile("rust")
            .profile("node")
            .active(&["rust"])
            .build_dest_tree();
        let profile_labels: Vec<_> = dest
            .iter()
            .filter(|n| n.kind == NodeKind::ProfileHeader)
            .map(|n| n.label.as_str())
            .collect();
        // rust (active) comes first, then git, node (inactive, alphabetical)
        assert_eq!(profile_labels, vec!["rust", "git", "node"]);
    }

    #[test]
    fn project_header_label_reflects_trust_state() {
        let dir = tempfile::tempdir().unwrap();
        let aliases_path = dir.path().join(".aliases");
        std::fs::write(&aliases_path, "[aliases]\nt = \"cargo test\"\n").unwrap();

        let app_model = amoxide::update::AppModel::new_with_security(
            amoxide::Config::default(),
            amoxide::ProfileConfig::default(),
            amoxide::security::SecurityConfig::default(),
        )
        .with_cwd(dir.path().to_path_buf());

        let tree = build_tree(&app_model);
        let project_node = tree
            .iter()
            .find(|n| n.kind == crate::model::NodeKind::ProjectHeader);
        assert!(project_node.is_some());
        let label = &project_node.unwrap().label;
        assert!(
            label.contains("untrusted") || label.contains("trust"),
            "expected trust hint in label, got: {label}"
        );
    }
}
