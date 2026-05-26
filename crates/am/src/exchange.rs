use std::fmt;

use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};

use crate::subcommand::{SubcommandSet, TomlSubcommand};
use crate::vars::VarSet;
use crate::{AliasSet, Described, Profile, ProjectAliases};

/// Current export-file format version. Written into `[meta]` by every export.
pub const EXPORT_FORMAT_VERSION: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub version: u32,
}

impl Default for Meta {
    fn default() -> Self {
        Self {
            version: EXPORT_FORMAT_VERSION,
        }
    }
}

/// One scope's full content: aliases, subcommand aliases, and variables.
/// Used at both `[global]` and `[local]` in the export TOML, and as a
/// granular payload inside [`ImportPayload`].
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ScopeBundle {
    #[serde(default, skip_serializing_if = "AliasSet::is_empty")]
    pub aliases: AliasSet,
    #[serde(default, skip_serializing_if = "SubcommandSet::is_empty")]
    pub subcommands: SubcommandSet,
    #[serde(default, skip_serializing_if = "VarSet::is_empty")]
    pub vars: VarSet,
}

impl ScopeBundle {
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty() && self.subcommands.is_empty() && self.vars.is_empty()
    }
}

/// Canonical export bundle (v2 format).
///
/// Serialized as TOML with a `[meta]` block carrying the format version,
/// followed by `[global.*]`, `[[profiles]]`, and `[local.*]` sections.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExportAll {
    #[serde(default)]
    pub meta: Meta,
    #[serde(default, skip_serializing_if = "ScopeBundle::is_empty")]
    pub global: ScopeBundle,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<Profile>,
    #[serde(default, skip_serializing_if = "ScopeBundle::is_empty")]
    pub local: ScopeBundle,
}

impl ExportAll {
    pub fn is_empty(&self) -> bool {
        self.global.is_empty() && self.profiles.is_empty() && self.local.is_empty()
    }

    /// Flatten all aliases from every scope into one AliasSet.
    /// Precedence: local > profiles (last profile wins) > global.
    /// Duplicate keys are silently overwritten by higher-priority scopes.
    pub fn flatten(&self) -> AliasSet {
        let mut result = AliasSet::default();
        for (name, alias) in self.global.aliases.iter() {
            result.insert(name.clone(), alias.clone());
        }
        for profile in &self.profiles {
            for (name, alias) in profile.aliases.iter() {
                result.insert(name.clone(), alias.clone());
            }
        }
        for (name, alias) in self.local.aliases.iter() {
            result.insert(name.clone(), alias.clone());
        }
        result
    }

    /// Flatten all subcommands from every scope into one SubcommandSet.
    /// Precedence: local > profiles (last profile wins) > global.
    /// Duplicate keys are silently overwritten by higher-priority scopes.
    pub fn flatten_subcommands(&self) -> SubcommandSet {
        let mut result = SubcommandSet::new();
        for (k, v) in &self.global.subcommands {
            result.as_mut().insert(k.clone(), v.clone());
        }
        for profile in &self.profiles {
            for (k, v) in &profile.subcommands {
                result.as_mut().insert(k.clone(), v.clone());
            }
        }
        for (k, v) in &self.local.subcommands {
            result.as_mut().insert(k.clone(), v.clone());
        }
        result
    }

    /// Flatten all variables from every scope into one VarSet.
    /// Precedence: local > profiles (last profile wins) > global.
    pub fn flatten_vars(&self) -> VarSet {
        let mut result = VarSet::default();
        for (name, value) in self.global.vars.iter() {
            result.insert(name.clone(), value.clone());
        }
        for profile in &self.profiles {
            for (name, value) in profile.vars.iter() {
                result.insert(name.clone(), value.clone());
            }
        }
        for (name, value) in self.local.vars.iter() {
            result.insert(name.clone(), value.clone());
        }
        result
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Legacy v1 format (amoxide < 0.9.0)
// ═══════════════════════════════════════════════════════════════════════

/// Legacy export bundle, used by amoxide releases before 0.9.0.
/// Deserialize only — never written. Variables are not part of this
/// format at any scope (profile vars were stripped on import in older
/// releases, so legacy exports cannot carry them either).
#[derive(Debug, Default, Deserialize)]
pub(crate) struct ExportAllV1 {
    #[serde(default)]
    pub global_aliases: AliasSet,
    #[serde(default)]
    pub global_subcommands: SubcommandSet,
    #[serde(default)]
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub local_aliases: AliasSet,
    #[serde(default)]
    pub local_subcommands: SubcommandSet,
}

impl ExportAllV1 {
    fn is_empty(&self) -> bool {
        self.global_aliases.is_empty()
            && self.global_subcommands.is_empty()
            && self.profiles.is_empty()
            && self.local_aliases.is_empty()
            && self.local_subcommands.is_empty()
    }

    /// Lift the legacy shape into the canonical v2 representation.
    ///
    /// The v1 format had no `vars` table at global or local scope — those are
    /// emitted as empty. Profile vars are preserved if the input TOML happens
    /// to carry them (the `Profile` struct itself has always supported the
    /// field — pre-0.9.0 the import handler just stripped them on the way in).
    fn into_v2(self) -> ExportAll {
        ExportAll {
            meta: Meta::default(),
            global: ScopeBundle {
                aliases: self.global_aliases,
                subcommands: self.global_subcommands,
                vars: VarSet::default(),
            },
            profiles: self.profiles,
            local: ScopeBundle {
                aliases: self.local_aliases,
                subcommands: self.local_subcommands,
                vars: VarSet::default(),
            },
        }
    }
}

/// Probe for the presence of `[meta]` to disambiguate v2 from legacy on import.
#[derive(Default, Deserialize)]
struct MetaPresenceProbe {
    #[serde(default)]
    meta: Option<MetaRaw>,
}

/// Empty placeholder — only used to detect the `[meta]` table's presence.
/// The strict `ExportAll` parse validates `meta.version` after dispatch.
#[derive(Default, Deserialize)]
struct MetaRaw {}

/// Granular per-scope payload — `None` on any field means "do not write".
/// Mirrors the merge-decline UX where the user accepts aliases but skips
/// subcommands (or vice versa) within one scope.
#[derive(Debug, Default)]
pub struct ScopeBundlePayload {
    pub aliases: Option<AliasSet>,
    pub subcommands: Option<SubcommandSet>,
    pub vars: Option<VarSet>,
}

#[derive(Debug, Default)]
pub struct ImportPayload {
    pub global: ScopeBundlePayload,
    pub profiles: Vec<Profile>,
    pub local: ScopeBundlePayload,
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
    pub current: TomlSubcommand,
    pub incoming: TomlSubcommand,
}

// ═══════════════════════════════════════════════════════════════════════
// Var merge
// ═══════════════════════════════════════════════════════════════════════

#[derive(Debug)]
pub struct VarMergeResult {
    pub new_vars: VarSet,
    pub conflicts: Vec<VarConflict>,
}

#[derive(Debug)]
pub struct VarConflict {
    pub name: crate::vars::VarName,
    pub current: String,
    pub incoming: String,
}

/// Compare `current` vars against `incoming`, separating new names from conflicts.
/// Identical entries (same name, same value) are silently skipped.
pub fn var_merge_check(current: &VarSet, incoming: &VarSet) -> VarMergeResult {
    let mut new_vars = VarSet::default();
    let mut conflicts = Vec::new();
    for (name, incoming_value) in incoming.iter() {
        match current.get(name) {
            None => {
                new_vars.insert(name.clone(), incoming_value.clone());
            }
            Some(existing_value) => {
                if existing_value != incoming_value {
                    conflicts.push(VarConflict {
                        name: name.clone(),
                        current: existing_value.clone(),
                        incoming: incoming_value.clone(),
                    });
                }
            }
        }
    }
    VarMergeResult {
        new_vars,
        conflicts,
    }
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
        match current.as_ref().get(key) {
            None => {
                new_subcommands
                    .as_mut()
                    .insert(key.clone(), incoming_longs.clone());
            }
            Some(existing) => {
                let same = existing.expansions() == incoming_longs.expansions()
                    && Described::description(existing) == Described::description(incoming_longs);
                if !same {
                    conflicts.push(SubcommandConflict {
                        key: key.clone(),
                        current: existing.clone(),
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

/// Outcome of parsing — distinguishes the canonical format from legacy
/// fallbacks so callers can render a one-line note when older exports
/// flow through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseSource {
    /// Current v2 format (with `[meta] version = 2`).
    V2,
    /// Legacy bundle from amoxide < 0.9.0 (no `[meta]` block).
    LegacyV1,
    /// Raw `.aliases` file dropped in as the import source.
    RawAliasesFile,
}

#[derive(Debug)]
pub struct ParsedImport {
    pub export: ExportAll,
    pub source: ParseSource,
}

/// Parse TOML input into [`ExportAll`].
///
/// Dispatch order:
/// 1. If `[meta]` is present, require `version = 2` and parse strictly as v2.
/// 2. Else try the legacy v1 layout (flat `global_aliases`/`local_aliases`
///    keys). Variables are absent at every scope after the lift.
/// 3. Else try a raw `.aliases` file (single project-local section).
pub fn parse_import(input: &str) -> anyhow::Result<ParsedImport> {
    let probe: MetaPresenceProbe = toml::from_str(input).unwrap_or_default();

    if probe.meta.is_some() {
        let export: ExportAll =
            toml::from_str(input).map_err(|e| anyhow::anyhow!("invalid v2 export: {e}"))?;
        if export.meta.version != EXPORT_FORMAT_VERSION {
            anyhow::bail!(
                "unsupported export format version {} (this amoxide supports version {})",
                export.meta.version,
                EXPORT_FORMAT_VERSION
            );
        }
        return Ok(ParsedImport {
            export,
            source: ParseSource::V2,
        });
    }

    if let Ok(v1) = toml::from_str::<ExportAllV1>(input) {
        if !v1.is_empty() {
            return Ok(ParsedImport {
                export: v1.into_v2(),
                source: ParseSource::LegacyV1,
            });
        }
    }

    // Fallback: raw .aliases — use if let Ok to avoid propagating TOML errors
    if let Ok(raw) = toml::from_str::<ProjectAliases>(input) {
        if !raw.aliases.is_empty() || !raw.vars.is_empty() || !raw.subcommands.is_empty() {
            return Ok(ParsedImport {
                export: ExportAll {
                    meta: Meta::default(),
                    local: ScopeBundle {
                        aliases: raw.aliases,
                        subcommands: raw.subcommands,
                        vars: raw.vars,
                    },
                    ..Default::default()
                },
                source: ParseSource::RawAliasesFile,
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
            if conflict.current.command() != conflict.incoming.command() {
                output.push_str(&format!("      - {}\n", conflict.current.command()));
                output.push_str(&format!("      + {}\n", conflict.incoming.command()));
            }
            let cur_desc = conflict.current.description();
            let inc_desc = conflict.incoming.description();
            if cur_desc != inc_desc {
                output.push_str(&format!("      - # {}\n", cur_desc.unwrap_or("(none)")));
                output.push_str(&format!("      + # {}\n", inc_desc.unwrap_or("(none)")));
            }
        }
    }

    output
}

/// Render the import summary for variables in a single scope.
pub fn render_import_summary_vars(scope_name: &str, result: &VarMergeResult) -> String {
    let total = result.new_vars.iter().count() + result.conflicts.len();
    let mut output = format!("Importing variables into \"{scope_name}\" ({total} entries)\n");

    if result.new_vars.iter().count() > 0 {
        output.push_str("\n  new:\n");
        for (name, value) in result.new_vars.iter() {
            output.push_str(&format!("    {} \u{2192} {}\n", name.as_str(), value));
        }
    }

    if !result.conflicts.is_empty() {
        output.push_str(&format!(
            "\n  {} conflict{}:\n",
            result.conflicts.len(),
            if result.conflicts.len() == 1 { "" } else { "s" }
        ));
        for conflict in &result.conflicts {
            output.push_str(&format!("\n    {}:\n", conflict.name.as_str()));
            output.push_str(&format!("      - {}\n", conflict.current));
            output.push_str(&format!("      + {}\n", conflict.incoming));
        }
    }

    output
}

/// Render the import summary for subcommand aliases in a single scope.
pub fn render_import_summary_subcommands(
    scope_name: &str,
    result: &SubcommandMergeResult,
) -> String {
    let total = result.new_subcommands.as_ref().len() + result.conflicts.len();
    let mut output = format!("Importing subcommands into \"{scope_name}\" ({total} entries)\n");

    if !result.new_subcommands.is_empty() {
        output.push_str("\n  new:\n");
        for (key, longs) in &result.new_subcommands {
            output.push_str(&format!(
                "    {} \u{2192} {}\n",
                key,
                longs.expansions().join(" ")
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
            output.push_str(&format!("\n    {}:\n", conflict.key));
            if conflict.current.expansions() != conflict.incoming.expansions() {
                output.push_str(&format!(
                    "      - {}\n",
                    conflict.current.expansions().join(" ")
                ));
                output.push_str(&format!(
                    "      + {}\n",
                    conflict.incoming.expansions().join(" ")
                ));
            }
            let cur_desc = Described::description(&conflict.current);
            let inc_desc = Described::description(&conflict.incoming);
            if cur_desc != inc_desc {
                output.push_str(&format!("      - # {}\n", cur_desc.unwrap_or("(none)")));
                output.push_str(&format!("      + # {}\n", inc_desc.unwrap_or("(none)")));
            }
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
    VarName,
    VarValue,
}

impl fmt::Display for SuspiciousField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SuspiciousField::AliasName => write!(f, "name"),
            SuspiciousField::AliasCommand => write!(f, "command"),
            SuspiciousField::ProfileName => write!(f, "profile_name"),
            SuspiciousField::SubcommandKey => write!(f, "subcommand_key"),
            SuspiciousField::SubcommandExpansion => write!(f, "subcommand_expansion"),
            SuspiciousField::VarName => write!(f, "var_name"),
            SuspiciousField::VarValue => write!(f, "var_value"),
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

    pub fn var_name(scope: Scope, name: &str) -> Self {
        Self {
            scope,
            alias_name: SanitizedName::new(name),
            field: SuspiciousField::VarName,
            raw_value: RawValue::new(name),
        }
    }

    pub fn var_value(scope: Scope, name: &str, value: &str) -> Self {
        Self {
            scope,
            alias_name: SanitizedName::new(name),
            field: SuspiciousField::VarValue,
            raw_value: RawValue::new(value),
        }
    }
}

/// Scan a parsed export for suspicious characters in alias names/commands,
/// profile names, subcommand keys/expansions, and variable names/values.
pub fn scan_suspicious(parsed: &ExportAll) -> Vec<SuspiciousAlias> {
    let mut findings = Vec::new();

    scan_bundle(&mut findings, Scope::Global, &parsed.global);

    for profile in &parsed.profiles {
        if has_suspicious_chars(&profile.name) {
            findings.push(SuspiciousAlias::profile_name(&profile.name));
        }
        let scope = Scope::Profile(SanitizedName::new(&profile.name));
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
        scan_subcommands(&mut findings, scope.clone(), &profile.subcommands);
        scan_vars(&mut findings, scope, &profile.vars);
    }

    scan_bundle(&mut findings, Scope::Local, &parsed.local);

    findings
}

fn scan_bundle(findings: &mut Vec<SuspiciousAlias>, scope: Scope, bundle: &ScopeBundle) {
    let alias_finding_factory = match scope {
        Scope::Global => (
            SuspiciousAlias::global_name as fn(&str) -> SuspiciousAlias,
            SuspiciousAlias::global_command as fn(&str, &str) -> SuspiciousAlias,
        ),
        Scope::Local => (
            SuspiciousAlias::local_name as fn(&str) -> SuspiciousAlias,
            SuspiciousAlias::local_command as fn(&str, &str) -> SuspiciousAlias,
        ),
        Scope::Profile(_) => unreachable!("profile bundles scanned via the profile loop"),
    };
    let (mk_name, mk_command) = alias_finding_factory;
    for (name, alias) in bundle.aliases.iter() {
        if has_suspicious_chars(name.as_ref()) {
            findings.push(mk_name(name.as_ref()));
        }
        if has_suspicious_chars(alias.command()) {
            findings.push(mk_command(name.as_ref(), alias.command()));
        }
    }
    scan_subcommands(findings, scope.clone(), &bundle.subcommands);
    scan_vars(findings, scope, &bundle.vars);
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
        for expansion in longs.expansions() {
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

fn scan_vars(findings: &mut Vec<SuspiciousAlias>, scope: Scope, vars: &VarSet) {
    for (name, value) in vars.iter() {
        let name_str = name.as_str();
        if has_suspicious_chars(name_str) {
            findings.push(SuspiciousAlias::var_name(scope.clone(), name_str));
        }
        if has_suspicious_chars(value) {
            findings.push(SuspiciousAlias::var_value(scope.clone(), name_str, value));
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
            .global
            .aliases
            .insert("ll".into(), TomlAlias::Command("ls -lha".into()));
        export.profiles.push(Profile {
            name: "git".into(),
            aliases: {
                let mut a = AliasSet::default();
                a.insert("gs".into(), TomlAlias::Command("git status".into()));
                a
            },
            subcommands: Default::default(),
            vars: Default::default(),
        });
        export
            .local
            .aliases
            .insert("t".into(), TomlAlias::Command("cargo test".into()));

        let toml_str = toml::to_string(&export).unwrap();
        let parsed: ExportAll = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.global.aliases.len(), 1);
        assert_eq!(parsed.profiles.len(), 1);
        assert_eq!(parsed.profiles[0].name, "git");
        assert_eq!(parsed.local.aliases.len(), 1);
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
            vars: Default::default(),
        });
        let toml_str = toml::to_string(&export).unwrap();
        assert!(!toml_str.contains("[global"));
        assert!(!toml_str.contains("[local"));
        assert!(toml_str.contains("[meta]"));
        assert!(toml_str.contains("[[profiles]]"));
    }

    #[test]
    fn test_parse_import_legacy_v1_format() {
        // Legacy amoxide < 0.9.0 export: flat global_aliases/local_aliases
        // without a [meta] block. Must still import.
        let input = indoc! {r#"
            [global_aliases]
            ll = "ls -lha"

            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#};
        let result = parse_import(input).unwrap();
        assert_eq!(result.source, ParseSource::LegacyV1);
        assert_eq!(result.export.global.aliases.len(), 1);
        assert_eq!(result.export.profiles.len(), 1);
        // Vars are not part of the v1 format — empty at every scope.
        assert!(result.export.global.vars.is_empty());
        assert!(result.export.local.vars.is_empty());
        assert!(result.export.profiles[0].vars.is_empty());
    }

    #[test]
    fn test_parse_import_raw_aliases_file() {
        let input = indoc! {r#"
            [aliases]
            t = "cargo test"
            b = "cargo build"
        "#};
        let result = parse_import(input).unwrap();
        assert_eq!(result.source, ParseSource::RawAliasesFile);
        // Raw .aliases is project-local content — routed to the local scope.
        assert_eq!(result.export.local.aliases.len(), 2);
        assert!(result.export.profiles.is_empty());
    }

    #[test]
    fn test_parse_import_raw_aliases_file_with_vars() {
        let input = indoc! {r#"
            [vars]
            path = "/v1"

            [aliases]
            t = "cargo test"
        "#};
        let result = parse_import(input).unwrap();
        assert_eq!(result.source, ParseSource::RawAliasesFile);
        assert_eq!(result.export.local.aliases.len(), 1);
        assert_eq!(result.export.local.vars.iter().count(), 1);
    }

    #[test]
    fn test_parse_import_single_profile() {
        // No [meta] block → legacy fallback handles `[[profiles]]`-only input.
        let input = indoc! {r#"
            [[profiles]]
            name = "docker"
            [profiles.aliases]
            dps = "docker ps"
            dcu = "docker compose up -d"
        "#};
        let result = parse_import(input).unwrap();
        assert!(result.export.global.aliases.is_empty());
        assert_eq!(result.export.profiles.len(), 1);
        assert_eq!(result.export.profiles[0].aliases.len(), 2);
    }

    #[test]
    fn test_parse_import_v2_with_meta() {
        let input = indoc! {r#"
            [meta]
            version = 2

            [global.aliases]
            ll = "ls -lha"

            [global.vars]
            editor = "hx"

            [[profiles]]
            name = "k8s"

            [profiles.aliases]
            klogs = "kubectl -n {{ns}} logs -f {{1}}"

            [profiles.vars]
            ns = "default"

            [local.aliases]
            t = "cargo test"

            [local.vars]
            target = "x86_64-unknown-linux-musl"
        "#};
        let result = parse_import(input).unwrap();
        assert_eq!(result.source, ParseSource::V2);
        assert_eq!(result.export.meta.version, 2);
        assert_eq!(result.export.global.aliases.len(), 1);
        assert_eq!(result.export.global.vars.iter().count(), 1);
        assert_eq!(result.export.profiles.len(), 1);
        assert_eq!(result.export.profiles[0].vars.iter().count(), 1);
        assert_eq!(result.export.local.aliases.len(), 1);
        assert_eq!(result.export.local.vars.iter().count(), 1);
    }

    #[test]
    fn test_parse_import_v2_unsupported_version() {
        let input = indoc! {r#"
            [meta]
            version = 99

            [global.aliases]
            ll = "ls -lha"
        "#};
        let err = parse_import(input).unwrap_err().to_string();
        assert!(err.contains("unsupported export format version"));
        assert!(err.contains("99"));
    }

    #[test]
    fn test_parse_import_v2_malformed_meta_is_error() {
        // [meta] present but malformed should error, not fall through to v1.
        let input = indoc! {r#"
            [meta]
            version = "not a number"
        "#};
        let result = parse_import(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_v2_round_trip_with_vars() {
        let mut export = ExportAll::default();
        export
            .global
            .aliases
            .insert("ll".into(), TomlAlias::Command("ls -lha".into()));
        export
            .global
            .vars
            .insert(crate::vars::VarName::parse("editor").unwrap(), "hx".into());
        export.profiles.push(Profile {
            name: "k8s".into(),
            aliases: AliasSet::default(),
            subcommands: Default::default(),
            vars: {
                let mut v = VarSet::default();
                v.insert(crate::vars::VarName::parse("ns").unwrap(), "default".into());
                v
            },
        });
        export.local.vars.insert(
            crate::vars::VarName::parse("target").unwrap(),
            "x86_64-unknown-linux-musl".into(),
        );

        let toml_str = toml::to_string(&export).unwrap();
        assert!(toml_str.contains("[meta]"));
        assert!(toml_str.contains("version = 2"));

        let parsed = parse_import(&toml_str).unwrap();
        assert_eq!(parsed.source, ParseSource::V2);
        assert_eq!(parsed.export.global.vars.iter().count(), 1);
        assert_eq!(parsed.export.profiles[0].vars.iter().count(), 1);
        assert_eq!(parsed.export.local.vars.iter().count(), 1);
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
            .global
            .aliases
            .insert("ll".into(), TomlAlias::Command("ls -lha".into()));
        export.profiles.push(Profile {
            name: "git".into(),
            aliases: {
                let mut a = AliasSet::default();
                a.insert("gs".into(), TomlAlias::Command("git status".into()));
                a
            },
            subcommands: Default::default(),
            vars: Default::default(),
        });
        export
            .local
            .aliases
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
            .global
            .aliases
            .insert("ll".into(), TomlAlias::Command("ls -lha".into()));
        assert!(scan_suspicious(&export).is_empty());
    }

    #[test]
    fn test_scan_suspicious_detects_command_escape() {
        let mut export = ExportAll::default();
        export.global.aliases.insert(
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
            .global
            .aliases
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
                vars: Default::default(),
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
                vars: Default::default(),
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
            .local
            .aliases
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

    // ─── flatten_subcommands ─────────────────────────────────────────────

    #[test]
    fn test_flatten_subcommands_merges_all_scopes() {
        let mut export = ExportAll::default();
        export.global.subcommands.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(vec!["abandon".into()]),
        );
        export.profiles.push(Profile {
            name: "vcs".into(),
            aliases: AliasSet::default(),
            subcommands: {
                let mut s = SubcommandSet::new();
                s.as_mut().insert(
                    "jj:d".into(),
                    TomlSubcommand::Expansion(vec!["diff".into()]),
                );
                s
            },
            vars: Default::default(),
        });
        export.local.subcommands.as_mut().insert(
            "git:psh".into(),
            TomlSubcommand::Expansion(vec!["push".into()]),
        );

        let flat = export.flatten_subcommands();
        assert_eq!(flat.as_ref().len(), 3);
        assert!(flat.as_ref().contains_key("jj:ab"));
        assert!(flat.as_ref().contains_key("jj:d"));
        assert!(flat.as_ref().contains_key("git:psh"));
    }

    #[test]
    fn test_flatten_subcommands_local_wins_over_global() {
        let mut export = ExportAll::default();
        export.global.subcommands.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(vec!["abandon".into()]),
        );
        export.local.subcommands.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(vec!["abandon", "!"].into_iter().map(String::from).collect()),
        );

        let flat = export.flatten_subcommands();
        assert_eq!(
            flat.as_ref()["jj:ab"],
            TomlSubcommand::Expansion(vec!["abandon".to_string(), "!".to_string()])
        );
    }

    // ─── subcommand_merge_check ──────────────────────────────────────────

    #[test]
    fn test_merge_check_new_entries() {
        let current = SubcommandSet::new();
        let mut incoming = SubcommandSet::new();
        incoming.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(vec!["abandon".into()]),
        );

        let result = subcommand_merge_check(&current, &incoming);
        assert_eq!(result.new_subcommands.as_ref().len(), 1);
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_merge_check_conflict() {
        let mut current = SubcommandSet::new();
        current.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(vec!["abandon".into()]),
        );
        let mut incoming = SubcommandSet::new();
        incoming.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(
                vec!["abandon", "--detach"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            ),
        );

        let result = subcommand_merge_check(&current, &incoming);
        assert!(result.new_subcommands.is_empty());
        assert_eq!(result.conflicts.len(), 1);
        assert_eq!(result.conflicts[0].key, "jj:ab");
    }

    #[test]
    fn test_merge_check_identical_entry_skipped() {
        let mut current = SubcommandSet::new();
        current.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(vec!["abandon".into()]),
        );
        let mut incoming = SubcommandSet::new();
        incoming.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(vec!["abandon".into()]),
        );

        let result = subcommand_merge_check(&current, &incoming);
        assert!(result.new_subcommands.is_empty());
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn subcommand_merge_check_description_change_is_conflict() {
        use crate::subcommand::SubcommandDetail;
        let mut current = SubcommandSet::new();
        current.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Detailed(SubcommandDetail {
                expansions: vec!["abandon".into()],
                description: Some("old".into()),
            }),
        );
        let mut incoming = SubcommandSet::new();
        incoming.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Detailed(SubcommandDetail {
                expansions: vec!["abandon".into()],
                description: Some("new".into()),
            }),
        );
        let result = subcommand_merge_check(&current, &incoming);
        assert!(result.new_subcommands.is_empty());
        assert_eq!(result.conflicts.len(), 1);
    }

    // ─── render_import_summary ───────────────────────────────────────────

    #[test]
    fn test_render_import_summary_new_only() {
        use crate::alias::MergeResult;
        let mut new_aliases = AliasSet::default();
        new_aliases.insert("ll".into(), TomlAlias::Command("ls -lha".into()));
        let result = MergeResult {
            new_aliases,
            conflicts: vec![],
        };
        let output = render_import_summary("global", &result);
        assert!(output.contains("global"));
        assert!(output.contains("1 aliases"));
        assert!(output.contains("ll"));
        assert!(output.contains("ls -lha"));
    }

    #[test]
    fn test_render_import_summary_with_conflicts() {
        use crate::alias::{AliasConflict, MergeResult};
        use crate::AliasName;
        let result = MergeResult {
            new_aliases: AliasSet::default(),
            conflicts: vec![AliasConflict {
                name: AliasName::from("gs"),
                current: TomlAlias::Command("git status".into()),
                incoming: TomlAlias::Command("git status --short".into()),
            }],
        };
        let output = render_import_summary("global", &result);
        assert!(output.contains("1 conflict"));
        assert!(output.contains("gs"));
        assert!(output.contains("git status --short"));
    }

    // ─── render_import_summary_subcommands ───────────────────────────────

    #[test]
    fn test_render_import_summary_subcommands_new_only() {
        let mut new_subcommands = SubcommandSet::new();
        new_subcommands.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(vec!["abandon".into()]),
        );
        let result = SubcommandMergeResult {
            new_subcommands,
            conflicts: vec![],
        };
        let output = render_import_summary_subcommands("global", &result);
        assert!(output.contains("global"));
        assert!(output.contains("1 entries"));
        assert!(output.contains("jj:ab"));
        assert!(output.contains("abandon"));
    }

    #[test]
    fn test_render_import_summary_subcommands_with_conflicts() {
        let result = SubcommandMergeResult {
            new_subcommands: SubcommandSet::new(),
            conflicts: vec![SubcommandConflict {
                key: "jj:ab".into(),
                current: TomlSubcommand::Expansion(vec!["abandon".into()]),
                incoming: TomlSubcommand::Expansion(vec!["abandon".into(), "--detach".into()]),
            }],
        };
        let output = render_import_summary_subcommands("global", &result);
        assert!(output.contains("1 conflict"));
        assert!(output.contains("jj:ab"));
        assert!(output.contains("--detach"));
    }

    // ─── scan_suspicious: subcommand paths ───────────────────────────────

    #[test]
    fn test_scan_suspicious_subcommand_key() {
        let mut export = ExportAll::default();
        export.global.subcommands.as_mut().insert(
            "jj:\x1Bab".into(),
            TomlSubcommand::Expansion(vec!["abandon".into()]),
        );
        let findings = scan_suspicious(&export);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].field.to_string(), "subcommand_key");
        assert_eq!(findings[0].scope.to_string(), "global");
    }

    #[test]
    fn test_scan_suspicious_subcommand_expansion() {
        let mut export = ExportAll::default();
        export.local.subcommands.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(vec!["aban\x07don".into()]),
        );
        let findings = scan_suspicious(&export);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].field.to_string(), "subcommand_expansion");
        assert_eq!(findings[0].scope.to_string(), "local");
    }

    #[test]
    fn test_scan_suspicious_profile_subcommand() {
        let export = ExportAll {
            profiles: vec![Profile {
                name: "vcs".into(),
                aliases: AliasSet::default(),
                subcommands: {
                    let mut s = SubcommandSet::new();
                    s.as_mut().insert(
                        "jj:ab".into(),
                        TomlSubcommand::Expansion(vec!["aban\x1Bdon".into()]),
                    );
                    s
                },
                vars: Default::default(),
            }],
            ..Default::default()
        };
        let findings = scan_suspicious(&export);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].scope.to_string(), "profile:vcs");
        assert_eq!(findings[0].field.to_string(), "subcommand_expansion");
    }

    // ─── ExportAll::is_empty ─────────────────────────────────────────────

    #[test]
    fn test_export_all_is_empty_false_with_global_subcommands() {
        let mut export = ExportAll::default();
        export.global.subcommands.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Expansion(vec!["abandon".into()]),
        );
        assert!(!export.is_empty());
    }

    #[test]
    fn test_export_all_is_empty_false_with_local_subcommands() {
        let mut export = ExportAll::default();
        export.local.subcommands.as_mut().insert(
            "git:psh".into(),
            TomlSubcommand::Expansion(vec!["push".into()]),
        );
        assert!(!export.is_empty());
    }

    // ─── base64_decode error path ────────────────────────────────────────

    #[test]
    fn test_base64_decode_invalid_input() {
        let result = base64_decode("not-valid-base64!!!");
        assert!(result.is_err());
    }

    // ─── description-only conflict suppresses command/expansion lines ─────

    #[test]
    fn render_import_summary_description_only_conflict_omits_command_lines() {
        use crate::alias::{AliasConflict, MergeResult};
        use crate::{AliasDetail, AliasName};
        let result = MergeResult {
            new_aliases: AliasSet::default(),
            conflicts: vec![AliasConflict {
                name: AliasName::from("gs"),
                current: TomlAlias::Detailed(AliasDetail {
                    command: "git status".into(),
                    description: Some("short".into()),
                    raw: false,
                }),
                incoming: TomlAlias::Detailed(AliasDetail {
                    command: "git status".into(),
                    description: Some("detailed status".into()),
                    raw: false,
                }),
            }],
        };
        let output = render_import_summary("global", &result);
        // Command lines suppressed because both sides are identical
        let command_lines: Vec<&str> = output
            .lines()
            .filter(|l| l.trim_start().starts_with("- git") || l.trim_start().starts_with("+ git"))
            .collect();
        assert!(
            command_lines.is_empty(),
            "expected no command lines for description-only conflict, got:\n{output}"
        );
        // Description diff still appears
        assert!(output.contains("short"));
        assert!(output.contains("detailed status"));
    }

    #[test]
    fn render_import_summary_subcommands_expansion_only_conflict_omits_expansion_lines() {
        use crate::subcommand::{SubcommandDetail, TomlSubcommand};
        let result = SubcommandMergeResult {
            new_subcommands: SubcommandSet::new(),
            conflicts: vec![SubcommandConflict {
                key: "jj:ab".into(),
                current: TomlSubcommand::Detailed(SubcommandDetail {
                    expansions: vec!["abandon".into()],
                    description: Some("old".into()),
                }),
                incoming: TomlSubcommand::Detailed(SubcommandDetail {
                    expansions: vec!["abandon".into()],
                    description: Some("new".into()),
                }),
            }],
        };
        let output = render_import_summary_subcommands("global", &result);
        let expansion_lines: Vec<&str> = output
            .lines()
            .filter(|l| {
                l.trim_start().starts_with("- abandon") || l.trim_start().starts_with("+ abandon")
            })
            .collect();
        assert!(
            expansion_lines.is_empty(),
            "expected no expansion lines for description-only conflict, got:\n{output}"
        );
    }
}
