use std::collections::BTreeMap;
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct VarSet(BTreeMap<VarName, String>);

impl VarSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, name: &VarName) -> Option<&String> {
        self.0.get(name)
    }

    pub fn contains(&self, name: &VarName) -> bool {
        self.0.contains_key(name)
    }

    pub fn insert(&mut self, name: VarName, value: String) -> Option<String> {
        self.0.insert(name, value)
    }

    pub fn remove(&mut self, name: &VarName) -> Option<String> {
        self.0.remove(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&VarName, &String)> {
        self.0.iter()
    }
}

pub static VAR_TEMPLATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_-]*)\}\}").unwrap());

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstitutionResult {
    pub output: String,
    pub missing: Vec<VarName>,
}

/// Replace `{{name}}` references with values from `vars`.
/// Names not in `vars` are left literal in `output` and reported in `missing`.
///
/// Substitution is a plain text replacement — vars are literal strings and
/// preserve whatever quoting the user wrote around them. (`'{{x}}'` stays
/// single-quoted, `"{{x}}"` stays double-quoted, bare `{{x}}` lands unquoted.)
/// This is intentionally different from positional-arg substitution
/// (`{{1}}`/`{{@}}`), where the substituted form is a shell variable
/// reference that must escape single-quoted regions to expand correctly.
pub fn substitute_vars(cmd: &str, vars: &VarSet) -> SubstitutionResult {
    let mut missing: Vec<VarName> = Vec::new();

    let output = VAR_TEMPLATE_RE
        .replace_all(cmd, |caps: &regex::Captures| -> String {
            let name = &caps[1];
            // Regex enforces VarName-compatible shape, so parse() always succeeds here.
            let parsed = match VarName::parse(name) {
                Ok(n) => n,
                Err(_) => return format!("{{{{{name}}}}}"),
            };
            match vars.get(&parsed) {
                Some(v) => v.clone(),
                None => {
                    if !missing.iter().any(|x| x == &parsed) {
                        missing.push(parsed);
                    }
                    format!("{{{{{name}}}}}")
                }
            }
        })
        .into_owned();

    SubstitutionResult { output, missing }
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

    #[test]
    fn varset_default_is_empty() {
        let s = VarSet::default();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn varset_insert_get_remove() {
        let mut s = VarSet::default();
        let n = VarName::parse("path").unwrap();
        s.insert(n.clone(), "/v1".to_string());
        assert_eq!(s.get(&n).map(String::as_str), Some("/v1"));
        assert_eq!(s.len(), 1);
        assert_eq!(s.remove(&n), Some("/v1".to_string()));
        assert!(s.is_empty());
    }

    #[test]
    fn varset_serde_roundtrip_populated() {
        #[derive(Serialize, Deserialize)]
        struct Wrapper {
            #[serde(default)]
            vars: VarSet,
        }
        let toml_str = "[vars]\npath = \"/opt/v1\"\nopt-flags = \"-C opt-level=3\"\n";
        let w: Wrapper = toml::from_str(toml_str).unwrap();
        assert_eq!(w.vars.len(), 2);
        assert_eq!(
            w.vars
                .get(&VarName::parse("path").unwrap())
                .map(String::as_str),
            Some("/opt/v1")
        );

        let out = toml::to_string(&w.vars).unwrap();
        let parsed: VarSet = toml::from_str(&out).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn varset_missing_table_defaults_to_empty() {
        #[derive(Deserialize)]
        struct Wrapper {
            #[serde(default)]
            vars: VarSet,
        }
        let w: Wrapper = toml::from_str("").unwrap();
        assert!(w.vars.is_empty());
    }

    #[test]
    fn varset_rejects_invalid_name_at_deserialize() {
        let r: Result<VarSet, _> = toml::from_str("\"1foo\" = \"x\"\n");
        assert!(r.is_err(), "must fail to parse a name starting with digit");
    }

    #[test]
    fn substitute_vars_replaces_known() {
        let mut vs = VarSet::default();
        vs.insert(VarName::parse("path").unwrap(), "/opt/v1".into());
        let r = substitute_vars("run {{path}}/x.sh", &vs);
        assert_eq!(r.output, "run /opt/v1/x.sh");
        assert!(r.missing.is_empty());
    }

    #[test]
    fn substitute_vars_reports_missing_and_leaves_literal() {
        let vs = VarSet::default();
        let r = substitute_vars("run {{nope}}", &vs);
        assert_eq!(r.output, "run {{nope}}");
        assert_eq!(r.missing.len(), 1);
        assert_eq!(r.missing[0].as_str(), "nope");
    }

    #[test]
    fn substitute_vars_multiple_refs_same_var() {
        let mut vs = VarSet::default();
        vs.insert(VarName::parse("p").unwrap(), "/x".into());
        let r = substitute_vars("a {{p}} b {{p}} c", &vs);
        assert_eq!(r.output, "a /x b /x c");
        assert!(r.missing.is_empty());
    }

    #[test]
    fn substitute_vars_value_contains_shell_metacharacters_passed_through() {
        let mut vs = VarSet::default();
        vs.insert(
            VarName::parse("opt-flags").unwrap(),
            "-C opt-level=3".into(),
        );
        let r = substitute_vars("compile {{opt-flags}}", &vs);
        assert_eq!(r.output, "compile -C opt-level=3");
    }

    #[test]
    fn substitute_vars_empty_set_is_passthrough() {
        let vs = VarSet::default();
        let r = substitute_vars("git status", &vs);
        assert_eq!(r.output, "git status");
        assert!(r.missing.is_empty());
    }

    #[test]
    fn substitute_vars_does_not_match_positional_args() {
        let vs = VarSet::default();
        let r = substitute_vars("echo {{1}} {{@}}", &vs);
        assert_eq!(r.output, "echo {{1}} {{@}}");
        assert!(r.missing.is_empty(), "positional args are not vars");
    }

    #[test]
    fn substitute_vars_inside_single_quotes_preserves_quotes() {
        // Vars are literal text, so substitution lands inside the user's
        // quotes — the value stays a single token even with whitespace.
        let mut vs = VarSet::default();
        vs.insert(VarName::parse("p").unwrap(), "X".into());
        let r = substitute_vars("awk '{{p}}'", &vs);
        assert_eq!(r.output, "awk 'X'");
    }

    #[test]
    fn substitute_vars_value_with_spaces_inside_single_quotes() {
        // Real-world case: `RUSTFLAGS='{{opts}}' cargo run` with
        // opts="-C opt-level=3" must yield a single `RUSTFLAGS=...` token,
        // not split into separate words. This was the user-reported bug
        // where `''-C opt-level=3''` made fish parse `-C` as a command.
        let mut vs = VarSet::default();
        vs.insert(VarName::parse("opts").unwrap(), "-C opt-level=3".into());
        let r = substitute_vars("RUSTFLAGS='{{opts}}' cargo run --release", &vs);
        assert_eq!(r.output, "RUSTFLAGS='-C opt-level=3' cargo run --release");
    }
}
