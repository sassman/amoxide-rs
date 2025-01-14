use std::{collections::BTreeMap, fmt::Display};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AliasName<T: AsRef<str> = String>(T);

impl From<&str> for AliasName {
    fn from(name: &str) -> Self {
        Self(name.into())
    }
}

impl From<String> for AliasName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

impl AsRef<str> for AliasName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Display for AliasName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type AliasSet = BTreeMap<AliasName, TomlAlias>;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum TomlAlias {
    Command(String),
    Detailed(AliasDetail),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AliasDetail {
    pub name: String,
    pub command: String,
    pub description: Option<String>,
}

impl Eq for AliasDetail {}

impl PartialEq<AliasDetail> for AliasDetail {
    fn eq(&self, other: &AliasDetail) -> bool {
        self.name == other.name
    }
}

impl Ord for AliasDetail {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd<AliasDetail> for AliasDetail {
    fn partial_cmp(&self, other: &AliasDetail) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
