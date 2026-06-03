//! Shared description helpers for items that may carry a human-readable note.

/// Trait implemented by any item that may optionally carry a description.
/// Used by renderers to handle regular aliases and subcommand aliases
/// uniformly without caring about the concrete TOML variant.
pub trait Described {
    fn description(&self) -> Option<&str>;
}

/// Three-state description update used by `am add` / `am edit` style flows.
///
/// Distinguishes "user did not pass `-d`" (`Preserve`) from "user passed
/// `-d ""`" (`Clear`) — important when an alias already exists, so the
/// caller's silence isn't silently destructive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescriptionUpdate {
    /// No `-d` flag — keep any existing description when overwriting an
    /// alias, or set `None` when creating a fresh one.
    Preserve,
    /// `-d ""` (or whitespace-only) — explicitly clear any existing
    /// description.
    Clear,
    /// `-d "value"` — set the description to this value.
    Set(String),
}

impl DescriptionUpdate {
    /// Convert a clap-style `Option<String>` (from `-d <VALUE>`) into a
    /// `DescriptionUpdate`. `None` means the flag was not passed
    /// (`Preserve`). `Some(s)` is normalised: whitespace-only becomes
    /// `Clear`, otherwise `Set(normalised)`.
    pub fn from_cli_arg(arg: Option<&str>) -> Self {
        match arg {
            None => DescriptionUpdate::Preserve,
            Some(s) => match normalize_description(s) {
                None => DescriptionUpdate::Clear,
                Some(t) => DescriptionUpdate::Set(t),
            },
        }
    }

    /// Resolve to a final `Option<String>` given the description that was
    /// already on the existing entry (if any).
    pub fn resolve(self, existing: Option<&str>) -> Option<String> {
        match self {
            DescriptionUpdate::Preserve => existing.map(str::to_owned),
            DescriptionUpdate::Clear => None,
            DescriptionUpdate::Set(s) => Some(s),
        }
    }
}

impl From<Option<String>> for DescriptionUpdate {
    /// Convert an `Option<String>` representing an explicit edit (TUI form,
    /// internal re-add). `Some(s)` → `Set(s)`; `None` → `Clear` — the value
    /// was explicitly chosen.
    fn from(opt: Option<String>) -> Self {
        match opt {
            Some(s) => DescriptionUpdate::Set(s),
            None => DescriptionUpdate::Clear,
        }
    }
}

/// Normalise a raw description string: trim whitespace; treat the
/// resulting empty string as `None`. Used everywhere a description
/// enters the system (CLI args, TUI confirm, serde deserialize).
pub fn normalize_description(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Serde deserializer helper: deserialise an `Option<String>` description
/// and normalise it (trim + empty→None) in one step.
/// Used by [`SubcommandDetail`] and [`AliasDetail`].
pub(crate) fn deserialize_normalized_description<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw: Option<String> = Option::deserialize(d)?;
    Ok(raw.and_then(|s| normalize_description(&s)))
}

// Bring `Deserialize` into scope only for the helper above.
use serde::Deserialize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_empty_string_returns_none() {
        assert_eq!(normalize_description(""), None);
    }

    #[test]
    fn normalize_whitespace_only_returns_none() {
        assert_eq!(normalize_description("   "), None);
        assert_eq!(normalize_description("\t"), None);
        assert_eq!(normalize_description("\n  "), None);
    }

    #[test]
    fn normalize_trims_and_returns_some() {
        assert_eq!(normalize_description("  hi  "), Some("hi".to_string()));
    }

    #[test]
    fn normalize_keeps_internal_spaces() {
        assert_eq!(
            normalize_description("  a b c  "),
            Some("a b c".to_string())
        );
    }
}
