use std::fmt;

/// One entry in the `_AM_ALIASES` / `_AM_SUBCOMMANDS` env var, in the
/// `"name|hash"` format (or a legacy bare `"name"` with no hash).
///
/// `hash = None` means the shell reloaded from an older amoxide that only
/// tracked names; the diff treats such entries as "always differs" so they
/// get reloaded on the next sync.
#[derive(Debug, Clone, PartialEq)]
pub struct AliasWithHash {
    name: String,
    hash: Option<String>,
}

impl AliasWithHash {
    pub fn new(name: impl Into<String>, hash: Option<String>) -> Self {
        Self {
            name: name.into(),
            hash,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn hash(&self) -> Option<&str> {
        self.hash.as_deref()
    }

    /// Parse one `"name|hash"` (or bare `"name"`) token. Returns `None` when
    /// the name segment is empty — callers skip such entries silently.
    pub fn parse(token: &str) -> Option<Self> {
        match token.split_once('|') {
            Some((name, hash)) if !name.is_empty() => Some(Self {
                name: name.to_string(),
                hash: Some(hash.to_string()),
            }),
            Some(_) => None, // empty name before '|'
            None if token.is_empty() => None,
            None => Some(Self {
                name: token.to_string(),
                hash: None,
            }),
        }
    }
}

impl fmt::Display for AliasWithHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.hash {
            Some(h) => write!(f, "{}|{}", self.name, h),
            None => write!(f, "{}", self.name),
        }
    }
}

/// A comma-separated list of [`AliasWithHash`] entries — the on-the-wire
/// format of `_AM_ALIASES` and `_AM_SUBCOMMANDS`.
///
/// Owns round-trip parsing and rendering so no other module has to know
/// about the `"name|hash,name|hash,..."` layout.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct AliasWithHashList(Vec<AliasWithHash>);

impl AliasWithHashList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, entry: AliasWithHash) {
        self.0.push(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, AliasWithHash> {
        self.0.iter()
    }

    /// Parse an `_AM_ALIASES` / `_AM_SUBCOMMANDS` value. `None` or empty
    /// string yields an empty list; malformed tokens are skipped.
    pub fn parse(raw: Option<&str>) -> Self {
        let Some(s) = raw.filter(|s| !s.is_empty()) else {
            return Self::new();
        };
        Self(s.split(',').filter_map(AliasWithHash::parse).collect())
    }
}

impl fmt::Display for AliasWithHashList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, entry) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{entry}")?;
        }
        Ok(())
    }
}

impl FromIterator<AliasWithHash> for AliasWithHashList {
    fn from_iter<I: IntoIterator<Item = AliasWithHash>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<'a> IntoIterator for &'a AliasWithHashList {
    type Item = &'a AliasWithHash;
    type IntoIter = std::slice::Iter<'a, AliasWithHash>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl IntoIterator for AliasWithHashList {
    type Item = AliasWithHash;
    type IntoIter = std::vec::IntoIter<AliasWithHash>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alias_with_hash_parse_new_format() {
        let e = AliasWithHash::parse("b|abc1234").unwrap();
        assert_eq!(e.name(), "b");
        assert_eq!(e.hash(), Some("abc1234"));
    }

    #[test]
    fn alias_with_hash_parse_bare_name() {
        let e = AliasWithHash::parse("t").unwrap();
        assert_eq!(e.name(), "t");
        assert_eq!(e.hash(), None);
    }

    #[test]
    fn alias_with_hash_parse_empty_returns_none() {
        assert!(AliasWithHash::parse("").is_none());
        assert!(AliasWithHash::parse("|abc").is_none());
    }

    #[test]
    fn alias_with_hash_display_roundtrip() {
        assert_eq!(
            AliasWithHash::new("b", Some("abc1234".into())).to_string(),
            "b|abc1234"
        );
        assert_eq!(AliasWithHash::new("t", None).to_string(), "t");
    }

    #[test]
    fn alias_with_hash_list_parse_and_render() {
        let list = AliasWithHashList::parse(Some("b|abc1234,t|def5678"));
        assert_eq!(list.iter().count(), 2);
        assert_eq!(list.to_string(), "b|abc1234,t|def5678");
    }

    #[test]
    fn alias_with_hash_list_parse_mixed_format() {
        let list = AliasWithHashList::parse(Some("b|abc1234,t,gs|fed9876"));
        let names: Vec<&str> = list.iter().map(|e| e.name()).collect();
        assert_eq!(names, vec!["b", "t", "gs"]);
        assert_eq!(list.iter().nth(1).unwrap().hash(), None);
    }

    #[test]
    fn alias_with_hash_list_parse_empty_and_none() {
        assert!(AliasWithHashList::parse(None).is_empty());
        assert!(AliasWithHashList::parse(Some("")).is_empty());
    }

    #[test]
    fn alias_with_hash_list_parse_skips_malformed_tokens() {
        // Leading empty token and "|xxx" token get dropped silently.
        let list = AliasWithHashList::parse(Some(",|xxx,b|abc1234"));
        let names: Vec<&str> = list.iter().map(|e| e.name()).collect();
        assert_eq!(names, vec!["b"]);
    }

    #[test]
    fn alias_with_hash_list_display_empty() {
        assert_eq!(AliasWithHashList::new().to_string(), "");
    }
}
