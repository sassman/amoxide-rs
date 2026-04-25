use std::fmt::{self, Display};
use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VarNameError {
    #[error("variable name must not be empty")]
    Empty,
    #[error("variable name '{0}' must start with a letter or underscore")]
    BadFirstChar(String),
    #[error("variable name '{0}' contains invalid character — allowed: letters, digits, '_', '-'")]
    BadBodyChar(String),
    #[error("variable name '{0}' is reserved (collides with positional-arg template syntax)")]
    Reserved(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VarName(String);

static VAR_NAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]*$").unwrap());

impl VarName {
    pub fn parse(name: &str) -> Result<Self, VarNameError> {
        if name.is_empty() {
            return Err(VarNameError::Empty);
        }
        if name == "@" || (name.len() == 1 && name.chars().next().unwrap().is_ascii_digit()) {
            return Err(VarNameError::Reserved(name.to_string()));
        }
        let first = name.chars().next().unwrap();
        if !(first.is_ascii_alphabetic() || first == '_') {
            return Err(VarNameError::BadFirstChar(name.to_string()));
        }
        if !VAR_NAME_RE.is_match(name) {
            return Err(VarNameError::BadBodyChar(name.to_string()));
        }
        Ok(Self(name.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for VarName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Serialize for VarName {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for VarName {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        VarName::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_accepts_valid_names() {
        for name in ["foo", "opt-flags", "_x", "a1", "FOO_BAR", "a_b-c"] {
            assert!(VarName::parse(name).is_ok(), "should accept {name}");
        }
    }

    #[test]
    fn parse_rejects_empty() {
        assert_eq!(VarName::parse(""), Err(VarNameError::Empty));
    }

    #[test]
    fn parse_rejects_positional_collisions() {
        for name in ["@", "1", "9"] {
            assert!(
                matches!(VarName::parse(name), Err(VarNameError::Reserved(_))),
                "should reject {name} as reserved"
            );
        }
    }

    #[test]
    fn parse_rejects_bad_first_char() {
        for name in ["1foo", "-foo", "9bar", "{x}"] {
            assert!(
                matches!(VarName::parse(name), Err(VarNameError::BadFirstChar(_))),
                "should reject {name}: bad first char"
            );
        }
    }

    #[test]
    fn parse_rejects_invalid_body_chars() {
        for name in ["foo bar", "foo.bar", "a+b", "x}", "a{b"] {
            assert!(
                matches!(VarName::parse(name), Err(VarNameError::BadBodyChar(_))),
                "should reject {name}: bad body"
            );
        }
    }

    #[test]
    fn display_returns_underlying_string() {
        let n = VarName::parse("opt-flags").unwrap();
        assert_eq!(n.to_string(), "opt-flags");
        assert_eq!(n.as_str(), "opt-flags");
    }
}
