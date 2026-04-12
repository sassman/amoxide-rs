use std::fmt;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};

use crate::subcommand::SubcommandSet;
use crate::{AliasSet, Profile, ProjectAliases};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExportAll {
    #[serde(default, skip_serializing_if = "AliasSet::is_empty")]
    pub global_aliases: AliasSet,
    #[serde(default, skip_serializing_if = "SubcommandSet::is_empty")]
    pub global_subcommands: SubcommandSet,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<Profile>,
    #[serde(default, skip_serializing_if = "AliasSet::is_empty")]
    pub local_aliases: AliasSet,
    #[serde(default, skip_serializing_if = "SubcommandSet::is_empty")]
    pub local_subcommands: SubcommandSet,
}

impl ExportAll {
    pub fn is_empty(&self) -> bool {
        self.global_aliases.is_empty()
            && self.global_subcommands.is_empty()
            && self.profiles.is_empty()
            && self.local_aliases.is_empty()
            && self.local_subcommands.is_empty()
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

    /// Flatten all subcommands from every scope into one SubcommandSet.
    pub fn flatten_subcommands(&self) -> SubcommandSet {
        let mut result = SubcommandSet::new();
        for (k, v) in &self.global_subcommands {
            result.insert(k.clone(), v.clone());
        }
        for profile in &self.profiles {
            for (k, v) in &profile.subcommands {
                result.insert(k.clone(), v.clone());
            }
        }
        for (k, v) in &self.local_subcommands {
            result.insert(k.clone(), v.clone());
        }
        result
    }
}

#[derive(Debug, Default)]
pub struct ImportPayload {
    pub global_aliases: Option<AliasSet>,
    pub global_subcommands: Option<SubcommandSet>,
    pub profiles: Vec<Profile>,
    pub local_aliases: Option<AliasSet>,
    pub local_subcommands: Option<SubcommandSet>,
}

// ═══════════════════════════════════════════════════════════════════════
// Subcommand merge
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct SubcommandMergeResult {
    pub new_subcommands: SubcommandSet,
    pub conflicts: Vec<SubcommandConflict>,
}

#[derive(Debug)]
pub struct SubcommandConflict {
    pub key: String,
    pub current: Vec<String>,
    pub incoming: Vec<String>,
}

/// Compare `current` subcommands against `incoming`, separating new keys from conflicts.
/// Identical entries (same key, same expansion) are silently skipped.
pub fn subcommand_merge_check(
    current: &SubcommandSet,
    incoming: &SubcommandSet,
) -> SubcommandMergeResult {
    let mut new_subcommands = SubcommandSet::new();
    let mut conflicts = Vec::new();
    for (key, incoming_longs) in incoming {
        match current.get(key) {
            None => {
                new_subcommands.insert(key.clone(), incoming_longs.clone());
            }
            Some(existing_longs) => {
                if existing_longs != incoming_longs {
                    conflicts.push(SubcommandConflict {
                        key: key.clone(),
                        current: existing_longs.clone(),
                        incoming: incoming_longs.clone(),
                    });
                }
            }
        }
    }
    SubcommandMergeResult {
        new_subcommands,
        conflicts,
    }
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

/// Render the import summary for subcommand aliases in a single scope.
pub fn render_import_summary_subcommands(
    scope_name: &str,
    result: &SubcommandMergeResult,
) -> String {
    let total = result.new_subcommands.len() + result.conflicts.len();
    let mut output = format!("Importing subcommands into \"{scope_name}\" ({total} entries)\n");

    if !result.new_subcommands.is_empty() {
        output.push_str("\n  new:\n");
        for (key, longs) in &result.new_subcommands {
            output.push_str(&format!("    {} \u{2192} {}\n", key, longs.join(" ")));
        }
    }

    if !result.conflicts.is_empty() {
        output.push_str(&format!(
            "\n  {} conflict{}:\n",
            result.conflicts.len(),
            if result.conflicts.len() == 1 { "" } else { "s" }
        ));
        for conflict in &result.conflicts {
            output.push_str(&format!("\n    {}:\n", conflict.key));
            output.push_str(&format!("      - {}\n", conflict.current.join(" ")));
            output.push_str(&format!("      + {}\n", conflict.incoming.join(" ")));
        }
    }

    output
}

pub fn base64_encode(input: &str) -> String {
    STANDARD.encode(input.as_bytes())
}

pub fn base64_decode(input: &str) -> anyhow::Result<String> {
    let bytes = STANDARD.decode(input.trim())?;
    Ok(String::from_utf8(bytes)?)
}

// ═══════════════════════════════════════════════════════════════════════
// Security: escape sequence detection
// ═══════════════════════════════════════════════════════════════════════

const NEWLINE: u32 = 0x0A;
const CARRIAGE_RETURN: u32 = 0x0D;
const TAB: u32 = 0x09;
const DEL: u32 = 0x7F;
const C1_RANGE: std::ops::RangeInclusive<u32> = 0x80..=0x9F;

/// Extension trait for detecting suspicious control characters on `char`.
pub trait SuspiciousChar {
    /// Returns true if this character is a control character that could
    /// manipulate terminal output (escape sequences, cursor control, etc.).
    ///
    /// Benign whitespace (`\n`, `\r`, `\t`) is excluded.
    fn is_suspicious(&self) -> bool;
}

impl SuspiciousChar for char {
    fn is_suspicious(&self) -> bool {
        let cp = *self as u32;
        (cp <= 0x1F && cp != NEWLINE && cp != CARRIAGE_RETURN && cp != TAB)
            || cp == DEL
            || C1_RANGE.contains(&cp)
    }
}

/// Returns true if the string contains suspicious control characters.
pub fn has_suspicious_chars(s: &str) -> bool {
    s.chars().any(|c| c.is_suspicious())
}

/// A string that has been sanitized for safe terminal display.
#[derive(Debug, Clone, PartialEq)]
pub struct SanitizedName(String);

impl SanitizedName {
    pub fn new(raw: &str) -> Self {
        Self(sanitize_for_display(raw))
    }
}

impl fmt::Display for SanitizedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A raw value that renders with escape notation for safe display.
#[derive(Debug, Clone)]
pub struct RawValue(String);

impl RawValue {
    pub fn new(raw: &str) -> Self {
        Self(raw.to_string())
    }

    /// Display with control chars rendered as `\u{XXXX}` notation.
    pub fn escaped(&self) -> String {
        escape_for_display(&self.0)
    }

    /// Display with control chars replaced by U+FFFD.
    pub fn sanitized(&self) -> String {
        sanitize_for_display(&self.0)
    }
}

impl fmt::Display for RawValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.escaped())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Scope {
    Global,
    Local,
    Profile(SanitizedName),
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Scope::Global => write!(f, "global"),
            Scope::Local => write!(f, "local"),
            Scope::Profile(name) => write!(f, "profile:{name}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SuspiciousField {
    AliasName,
    AliasCommand,
    ProfileName,
    SubcommandKey,
    SubcommandExpansion,
}

impl fmt::Display for SuspiciousField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SuspiciousField::AliasName => write!(f, "name"),
            SuspiciousField::AliasCommand => write!(f, "command"),
            SuspiciousField::ProfileName => write!(f, "profile_name"),
            SuspiciousField::SubcommandKey => write!(f, "subcommand_key"),
            SuspiciousField::SubcommandExpansion => write!(f, "subcommand_expansion"),
        }
    }
}

/// A suspicious alias finding — records scope, alias name, field, and the raw value.
#[derive(Debug, Clone)]
pub struct SuspiciousAlias {
    pub scope: Scope,
    pub alias_name: SanitizedName,
    pub field: SuspiciousField,
    pub raw_value: RawValue,
}

impl SuspiciousAlias {
    pub fn global_name(name: &str) -> Self {
        Self {
            scope: Scope::Global,
            alias_name: SanitizedName::new(name),
            field: SuspiciousField::AliasName,
            raw_value: RawValue::new(name),
        }
    }

    pub fn global_command(name: &str, command: &str) -> Self {
        Self {
            scope: Scope::Global,
            alias_name: SanitizedName::new(name),
            field: SuspiciousField::AliasCommand,
            raw_value: RawValue::new(command),
        }
    }

    pub fn local_name(name: &str) -> Self {
        Self {
            scope: Scope::Local,
            alias_name: SanitizedName::new(name),
            field: SuspiciousField::AliasName,
            raw_value: RawValue::new(name),
        }
    }

    pub fn local_command(name: &str, command: &str) -> Self {
        Self {
            scope: Scope::Local,
            alias_name: SanitizedName::new(name),
            field: SuspiciousField::AliasCommand,
            raw_value: RawValue::new(command),
        }
    }

    pub fn profile_name(profile_name: &str) -> Self {
        Self {
            scope: Scope::Profile(SanitizedName::new(profile_name)),
            alias_name: SanitizedName::new(""),
            field: SuspiciousField::ProfileName,
            raw_value: RawValue::new(profile_name),
        }
    }

    pub fn profile_alias_name(profile_name: &str, alias_name: &str) -> Self {
        Self {
            scope: Scope::Profile(SanitizedName::new(profile_name)),
            alias_name: SanitizedName::new(alias_name),
            field: SuspiciousField::AliasName,
            raw_value: RawValue::new(alias_name),
        }
    }

    pub fn profile_alias_command(profile_name: &str, alias_name: &str, command: &str) -> Self {
        Self {
            scope: Scope::Profile(SanitizedName::new(profile_name)),
            alias_name: SanitizedName::new(alias_name),
            field: SuspiciousField::AliasCommand,
            raw_value: RawValue::new(command),
        }
    }

    pub fn subcommand_key(scope: Scope, key: &str) -> Self {
        Self {
            scope,
            alias_name: SanitizedName::new(key),
            field: SuspiciousField::SubcommandKey,
            raw_value: RawValue::new(key),
        }
    }

    pub fn subcommand_expansion(scope: Scope, key: &str, expansion: &str) -> Self {
        Self {
            scope,
            alias_name: SanitizedName::new(key),
            field: SuspiciousField::SubcommandExpansion,
            raw_value: RawValue::new(expansion),
        }
    }
}

/// Scan a parsed export for suspicious characters in alias names, commands, profile names,
/// and subcommand keys/expansions.
pub fn scan_suspicious(parsed: &ExportAll) -> Vec<SuspiciousAlias> {
    let mut findings = Vec::new();

    for (name, alias) in parsed.global_aliases.iter() {
        if has_suspicious_chars(name.as_ref()) {
            findings.push(SuspiciousAlias::global_name(name.as_ref()));
        }
        if has_suspicious_chars(alias.command()) {
            findings.push(SuspiciousAlias::global_command(
                name.as_ref(),
                alias.command(),
            ));
        }
    }

    scan_subcommands(&mut findings, Scope::Global, &parsed.global_subcommands);

    for profile in &parsed.profiles {
        if has_suspicious_chars(&profile.name) {
            findings.push(SuspiciousAlias::profile_name(&profile.name));
        }
        for (name, alias) in profile.aliases.iter() {
            if has_suspicious_chars(name.as_ref()) {
                findings.push(SuspiciousAlias::profile_alias_name(
                    &profile.name,
                    name.as_ref(),
                ));
            }
            if has_suspicious_chars(alias.command()) {
                findings.push(SuspiciousAlias::profile_alias_command(
                    &profile.name,
                    name.as_ref(),
                    alias.command(),
                ));
            }
        }
        scan_subcommands(
            &mut findings,
            Scope::Profile(SanitizedName::new(&profile.name)),
            &profile.subcommands,
        );
    }

    for (name, alias) in parsed.local_aliases.iter() {
        if has_suspicious_chars(name.as_ref()) {
            findings.push(SuspiciousAlias::local_name(name.as_ref()));
        }
        if has_suspicious_chars(alias.command()) {
            findings.push(SuspiciousAlias::local_command(
                name.as_ref(),
                alias.command(),
            ));
        }
    }

    scan_subcommands(&mut findings, Scope::Local, &parsed.local_subcommands);

    findings
}

fn scan_subcommands(
    findings: &mut Vec<SuspiciousAlias>,
    scope: Scope,
    subcommands: &SubcommandSet,
) {
    for (key, longs) in subcommands {
        if has_suspicious_chars(key) {
            findings.push(SuspiciousAlias::subcommand_key(scope.clone(), key));
        }
        for expansion in longs {
            if has_suspicious_chars(expansion) {
                findings.push(SuspiciousAlias::subcommand_expansion(
                    scope.clone(),
                    key,
                    expansion,
                ));
            }
        }
    }
}

/// Render control characters as `\u{XXXX}` for safe display.
pub fn escape_for_display(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_suspicious() {
            out.push_str(&format!("\\u{{{:04X}}}", c as u32));
        } else {
            out.push(c);
        }
    }
    out
}

/// Replace suspicious control characters with the Unicode replacement character (U+FFFD).
pub fn sanitize_for_display(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_suspicious() { '\u{FFFD}' } else { c })
        .collect()
}

/// Render a human-readable warning for suspicious alias findings.
pub fn render_suspicious_warning(findings: &[SuspiciousAlias]) -> String {
    let mut out = String::new();
    out.push_str("WARNING: Suspicious characters detected in import\n");
    out.push_str("==================================================\n\n");
    out.push_str("The following entries contain control characters that could be used\n");
    out.push_str("to execute unintended commands or manipulate your terminal:\n\n");

    for finding in findings {
        out.push_str(&format!("  scope:        {}\n", finding.scope));
        if !finding.alias_name.0.is_empty() {
            out.push_str(&format!("  alias:        {}\n", finding.alias_name));
        }
        out.push_str(&format!("  field:        {}\n", finding.field));
        out.push_str(&format!("  original:     {}\n", finding.raw_value));
        out.push_str(&format!(
            "  safe-escaped: {}\n",
            finding.raw_value.sanitized()
        ));
        out.push('\n');
    }

    out.push_str("To import anyway, use: am import --yes --trust\n");
    out
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
            subcommands: Default::default(),
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
            subcommands: Default::default(),
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
            subcommands: Default::default(),
        });
        export
            .local_aliases
            .insert("t".into(), TomlAlias::Command("cargo test".into()));
        let flat = export.flatten();
        assert_eq!(flat.len(), 3);
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = "[global_aliases]\nll = \"ls -lha\"\n";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, original);
    }

    // ─── Security scanning tests ─────────────────────────────────────────

    #[test]
    fn test_has_suspicious_chars_clean() {
        assert!(!has_suspicious_chars("git status"));
        assert!(!has_suspicious_chars("ls -lha"));
        assert!(!has_suspicious_chars("echo hello\nworld"));
        assert!(!has_suspicious_chars("col1\tcol2"));
    }

    #[test]
    fn test_has_suspicious_chars_c0_controls() {
        // NUL
        assert!(has_suspicious_chars("foo\x00bar"));
        // BEL
        assert!(has_suspicious_chars("foo\x07bar"));
        // ESC
        assert!(has_suspicious_chars("foo\x1Bbar"));
    }

    #[test]
    fn test_has_suspicious_chars_cr_allowed() {
        // CR (carriage return) is allowed — comes from Windows line endings
        assert!(!has_suspicious_chars("foo\rbar"));
    }

    #[test]
    fn test_has_suspicious_chars_del_and_c1() {
        // DEL (0x7F)
        assert!(has_suspicious_chars("foo\x7Fbar"));
        // C1 control (0x80)
        assert!(has_suspicious_chars("foo\u{0080}bar"));
        // C1 control (0x9F)
        assert!(has_suspicious_chars("foo\u{009F}bar"));
        // Just above C1 range — should be clean
        assert!(!has_suspicious_chars("foo\u{00A0}bar"));
    }

    #[test]
    fn test_scan_suspicious_clean_export() {
        let mut export = ExportAll::default();
        export
            .global_aliases
            .insert("ll".into(), TomlAlias::Command("ls -lha".into()));
        assert!(scan_suspicious(&export).is_empty());
    }

    #[test]
    fn test_scan_suspicious_detects_command_escape() {
        let mut export = ExportAll::default();
        export.global_aliases.insert(
            "evil".into(),
            TomlAlias::Command("echo \x1B[31mhacked\x1B[0m".into()),
        );
        let findings = scan_suspicious(&export);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].scope.to_string(), "global");
        assert_eq!(findings[0].alias_name.to_string(), "evil");
        assert_eq!(findings[0].field.to_string(), "command");
    }

    #[test]
    fn test_scan_suspicious_detects_name_escape() {
        let mut export = ExportAll::default();
        export
            .global_aliases
            .insert("foo\x07bar".into(), TomlAlias::Command("ls".into()));
        let findings = scan_suspicious(&export);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].field.to_string(), "name");
    }

    #[test]
    fn test_scan_suspicious_detects_profile_name() {
        let export = ExportAll {
            profiles: vec![Profile {
                name: "evil\x1Bprofile".into(),
                aliases: AliasSet::default(),
                subcommands: Default::default(),
            }],
            ..Default::default()
        };
        let findings = scan_suspicious(&export);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].field.to_string(), "profile_name");
    }

    #[test]
    fn test_scan_suspicious_profile_aliases() {
        let export = ExportAll {
            profiles: vec![Profile {
                name: "git".into(),
                aliases: {
                    let mut a = AliasSet::default();
                    a.insert(
                        "gs".into(),
                        TomlAlias::Command("git \x1B[1mstatus\x1B[0m".into()),
                    );
                    a
                },
                subcommands: Default::default(),
            }],
            ..Default::default()
        };
        let findings = scan_suspicious(&export);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].scope.to_string(), "profile:git");
        assert_eq!(findings[0].field.to_string(), "command");
    }

    #[test]
    fn test_scan_suspicious_local_aliases() {
        let mut export = ExportAll::default();
        export
            .local_aliases
            .insert("test".into(), TomlAlias::Command("rm -rf / \x07".into()));
        let findings = scan_suspicious(&export);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].scope.to_string(), "local");
    }

    #[test]
    fn test_escape_for_display_clean() {
        assert_eq!(escape_for_display("hello world"), "hello world");
    }

    #[test]
    fn test_escape_for_display_control_chars() {
        assert_eq!(escape_for_display("foo\x1B[31mbar"), "foo\\u{001B}[31mbar");
        assert_eq!(escape_for_display("\x00"), "\\u{0000}");
        assert_eq!(escape_for_display("\x7F"), "\\u{007F}");
    }

    #[test]
    fn test_escape_for_display_preserves_newline_tab() {
        assert_eq!(escape_for_display("a\nb\tc"), "a\nb\tc");
    }

    #[test]
    fn test_sanitize_for_display_replaces_controls() {
        assert_eq!(sanitize_for_display("foo\x1Bbar"), "foo\u{FFFD}bar");
        assert_eq!(sanitize_for_display("\x07test"), "\u{FFFD}test");
    }

    #[test]
    fn test_sanitize_for_display_preserves_clean() {
        assert_eq!(sanitize_for_display("hello\nworld"), "hello\nworld");
    }

    #[test]
    fn test_render_suspicious_warning_output() {
        let findings = vec![SuspiciousAlias::global_command(
            "evil",
            "echo \x1B[31mhacked",
        )];
        let output = render_suspicious_warning(&findings);
        assert!(output.contains("WARNING"));
        assert!(output.contains("global"));
        assert!(output.contains("evil"));
        assert!(output.contains("\\u{001B}"));
        assert!(output.contains("--yes --trust"));
    }

    #[test]
    fn test_render_suspicious_warning_multiple_findings() {
        let findings = vec![
            SuspiciousAlias::global_command("a", "\x07beep"),
            SuspiciousAlias::profile_name("evil\x1Bname"),
        ];
        let output = render_suspicious_warning(&findings);
        // Should contain both findings
        assert!(output.contains("global"));
        assert!(output.contains("profile:evil"));
        assert!(output.contains("profile_name"));
    }
}
