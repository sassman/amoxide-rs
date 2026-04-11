use crate::AliasTarget;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum SubcommandScope {
    Global,
    Profile(String),
    Project,
}

/// Canonical reference to a single alias across any scope.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum AliasId {
    Global {
        alias_name: String,
    },
    Profile {
        profile_name: String,
        alias_name: String,
    },
    Project {
        alias_name: String,
    },
    Subcommand {
        scope: SubcommandScope,
        key: String,
    },
}

impl AliasId {
    pub fn name(&self) -> &str {
        match self {
            AliasId::Global { alias_name }
            | AliasId::Profile { alias_name, .. }
            | AliasId::Project { alias_name } => alias_name.as_str(),
            AliasId::Subcommand { key, .. } => key.as_str(),
        }
    }

    pub fn target(&self) -> AliasTarget {
        match self {
            AliasId::Global { .. } => AliasTarget::Global,
            AliasId::Profile { profile_name, .. } => AliasTarget::Profile(profile_name.clone()),
            AliasId::Project { .. } => AliasTarget::Local,
            AliasId::Subcommand { scope, .. } => match scope {
                SubcommandScope::Global => AliasTarget::Global,
                SubcommandScope::Profile(name) => AliasTarget::Profile(name.clone()),
                SubcommandScope::Project => AliasTarget::Local,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_id_name_returns_alias_name() {
        assert_eq!(
            AliasId::Global {
                alias_name: "ll".into()
            }
            .name(),
            "ll"
        );
        assert_eq!(
            AliasId::Profile {
                profile_name: "git".into(),
                alias_name: "gs".into()
            }
            .name(),
            "gs"
        );
        assert_eq!(
            AliasId::Project {
                alias_name: "t".into()
            }
            .name(),
            "t"
        );
    }

    #[test]
    fn alias_id_target_returns_correct_scope() {
        assert_eq!(
            AliasId::Global {
                alias_name: "x".into()
            }
            .target(),
            AliasTarget::Global
        );
        assert_eq!(
            AliasId::Profile {
                profile_name: "git".into(),
                alias_name: "gs".into()
            }
            .target(),
            AliasTarget::Profile("git".into())
        );
        assert_eq!(
            AliasId::Project {
                alias_name: "t".into()
            }
            .target(),
            AliasTarget::Local
        );
    }

    #[test]
    fn subcommand_scope_to_alias_target() {
        assert_eq!(
            AliasId::Subcommand {
                scope: SubcommandScope::Global,
                key: "jj:ab".into(),
            }
            .target(),
            AliasTarget::Global
        );
        assert_eq!(
            AliasId::Subcommand {
                scope: SubcommandScope::Profile("rust".into()),
                key: "cargo:t".into(),
            }
            .target(),
            AliasTarget::Profile("rust".into())
        );
        assert_eq!(
            AliasId::Subcommand {
                scope: SubcommandScope::Project,
                key: "k:gp".into(),
            }
            .target(),
            AliasTarget::Local
        );
    }

    #[test]
    fn subcommand_alias_id_name_returns_last_segment() {
        let id = AliasId::Subcommand {
            scope: SubcommandScope::Global,
            key: "jj:b:l".into(),
        };
        assert_eq!(id.name(), "jj:b:l");
    }

    #[test]
    fn alias_id_ordering_is_stable() {
        let mut ids = [
            AliasId::Profile {
                profile_name: "z".into(),
                alias_name: "b".into(),
            },
            AliasId::Global {
                alias_name: "a".into(),
            },
            AliasId::Project {
                alias_name: "c".into(),
            },
        ];
        ids.sort();
        assert!(matches!(ids[0], AliasId::Global { .. }));
        assert!(matches!(ids[1], AliasId::Profile { .. }));
        assert!(matches!(ids[2], AliasId::Project { .. }));
    }
}
