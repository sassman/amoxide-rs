use crate::AliasTarget;

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
}

impl AliasId {
    pub fn name(&self) -> &str {
        match self {
            AliasId::Global { alias_name }
            | AliasId::Profile { alias_name, .. }
            | AliasId::Project { alias_name } => alias_name.as_str(),
        }
    }

    pub fn target(&self) -> AliasTarget {
        match self {
            AliasId::Global { .. } => AliasTarget::Global,
            AliasId::Profile { profile_name, .. } => AliasTarget::Profile(profile_name.clone()),
            AliasId::Project { .. } => AliasTarget::Local,
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
    fn alias_id_ordering_is_stable() {
        let mut ids = vec![
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
