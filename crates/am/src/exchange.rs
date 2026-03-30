use serde::{Deserialize, Serialize};

use crate::{AliasSet, Profile, ProjectAliases};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExportAll {
    #[serde(default, skip_serializing_if = "AliasSet::is_empty")]
    pub global_aliases: AliasSet,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<Profile>,
    #[serde(default, skip_serializing_if = "AliasSet::is_empty")]
    pub local_aliases: AliasSet,
}

impl ExportAll {
    pub fn is_empty(&self) -> bool {
        self.global_aliases.is_empty() && self.profiles.is_empty() && self.local_aliases.is_empty()
    }

    pub fn flatten(&self) -> AliasSet {
        let mut result = AliasSet::default();
        for (name, alias) in self.global_aliases.iter() {
            result.insert(name.clone(), alias.clone());
        }
        for profile in &self.profiles {
            for (name, alias) in profile.aliases.iter() {
                result.insert(name.clone(), alias.clone());
            }
        }
        for (name, alias) in self.local_aliases.iter() {
            result.insert(name.clone(), alias.clone());
        }
        result
    }
}

#[derive(Debug, Default)]
pub struct ImportPayload {
    pub global_aliases: Option<AliasSet>,
    pub profiles: Vec<Profile>,
    pub local_aliases: Option<AliasSet>,
}

/// Parse TOML input into ExportAll, with fallback for raw `.aliases` files.
pub fn parse_import(input: &str) -> anyhow::Result<ExportAll> {
    let export_all: ExportAll = toml::from_str(input)?;
    if !export_all.is_empty() {
        return Ok(export_all);
    }

    // Fallback: try raw .aliases format — use if let Ok to avoid propagating TOML errors
    if let Ok(raw) = toml::from_str::<ProjectAliases>(input) {
        if !raw.aliases.is_empty() {
            return Ok(ExportAll {
                global_aliases: raw.aliases,
                ..Default::default()
            });
        }
    }

    anyhow::bail!("no aliases found in input")
}

use crate::alias::MergeResult;

/// Render the import summary for a single scope.
pub fn render_import_summary(scope_name: &str, result: &MergeResult) -> String {
    let total = result.new_aliases.len() + result.conflicts.len();
    let mut output = format!("Importing \"{scope_name}\" ({total} aliases)\n");

    if !result.new_aliases.is_empty() {
        output.push_str("\n  new:\n");
        for (name, alias) in result.new_aliases.iter() {
            output.push_str(&format!(
                "    {} \u{2192} {}\n",
                name.as_ref(),
                alias.command()
            ));
        }
    }

    if !result.conflicts.is_empty() {
        output.push_str(&format!(
            "\n  {} conflict{}:\n",
            result.conflicts.len(),
            if result.conflicts.len() == 1 { "" } else { "s" }
        ));
        for conflict in &result.conflicts {
            output.push_str(&format!("\n    {}:\n", conflict.name.as_ref()));
            output.push_str(&format!("      - {}\n", conflict.current.command()));
            output.push_str(&format!("      + {}\n", conflict.incoming.command()));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TomlAlias;
    use indoc::indoc;

    #[test]
    fn test_export_all_roundtrip() {
        let mut export = ExportAll::default();
        export
            .global_aliases
            .insert("ll".into(), TomlAlias::Command("ls -lha".into()));
        export.profiles.push(Profile {
            name: "git".into(),
            aliases: {
                let mut a = AliasSet::default();
                a.insert("gs".into(), TomlAlias::Command("git status".into()));
                a
            },
        });
        export
            .local_aliases
            .insert("t".into(), TomlAlias::Command("cargo test".into()));

        let toml_str = toml::to_string(&export).unwrap();
        let parsed: ExportAll = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.global_aliases.len(), 1);
        assert_eq!(parsed.profiles.len(), 1);
        assert_eq!(parsed.profiles[0].name, "git");
        assert_eq!(parsed.local_aliases.len(), 1);
    }

    #[test]
    fn test_export_all_empty_fields_omitted() {
        let mut export = ExportAll::default();
        export.profiles.push(Profile {
            name: "git".into(),
            aliases: {
                let mut a = AliasSet::default();
                a.insert("gs".into(), TomlAlias::Command("git status".into()));
                a
            },
        });
        let toml_str = toml::to_string(&export).unwrap();
        assert!(!toml_str.contains("global_aliases"));
        assert!(!toml_str.contains("local_aliases"));
        assert!(toml_str.contains("[[profiles]]"));
    }

    #[test]
    fn test_parse_import_export_all_format() {
        let input = indoc! {r#"
            [global_aliases]
            ll = "ls -lha"

            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#};
        let result = parse_import(input).unwrap();
        assert_eq!(result.global_aliases.len(), 1);
        assert_eq!(result.profiles.len(), 1);
    }

    #[test]
    fn test_parse_import_raw_aliases_file() {
        let input = indoc! {r#"
            [aliases]
            t = "cargo test"
            b = "cargo build"
        "#};
        let result = parse_import(input).unwrap();
        assert_eq!(result.global_aliases.len(), 2);
        assert!(result.profiles.is_empty());
    }

    #[test]
    fn test_parse_import_single_profile() {
        let input = indoc! {r#"
            [[profiles]]
            name = "docker"
            [profiles.aliases]
            dps = "docker ps"
            dcu = "docker compose up -d"
        "#};
        let result = parse_import(input).unwrap();
        assert!(result.global_aliases.is_empty());
        assert_eq!(result.profiles.len(), 1);
        assert_eq!(result.profiles[0].aliases.len(), 2);
    }

    #[test]
    fn test_parse_import_empty_input() {
        let result = parse_import("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_import_no_recognized_sections() {
        let result = parse_import("[something_else]\nfoo = \"bar\"");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no aliases found"));
    }

    #[test]
    fn test_flatten_merges_all_sections() {
        let mut export = ExportAll::default();
        export
            .global_aliases
            .insert("ll".into(), TomlAlias::Command("ls -lha".into()));
        export.profiles.push(Profile {
            name: "git".into(),
            aliases: {
                let mut a = AliasSet::default();
                a.insert("gs".into(), TomlAlias::Command("git status".into()));
                a
            },
        });
        export
            .local_aliases
            .insert("t".into(), TomlAlias::Command("cargo test".into()));
        let flat = export.flatten();
        assert_eq!(flat.len(), 3);
    }
}
