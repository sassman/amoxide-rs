//! Shared description helpers for items that may carry a human-readable note.

/// Trait implemented by any item that may optionally carry a description.
/// Used by renderers to handle regular aliases and subcommand aliases
/// uniformly without caring about the concrete TOML variant.
pub trait Described {
    fn description(&self) -> Option<&str>;
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
pub(crate) fn deserialize_normalized_description<'de, D>(
    d: D,
) -> Result<Option<String>, D::Error>
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
        assert_eq!(
            normalize_description("  hi  "),
            Some("hi".to_string())
        );
    }

    #[test]
    fn normalize_keeps_internal_spaces() {
        assert_eq!(
            normalize_description("  a b c  "),
            Some("a b c".to_string())
        );
    }
}
