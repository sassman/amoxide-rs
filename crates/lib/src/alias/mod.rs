use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

pub mod add;
pub mod env;
pub mod shared;

pub use shared::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasEntry {
    value: String,
    directory: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AliasConfig {
    aliases: HashMap<String, AliasEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShellAlias {
    pub name: String,
    pub value: String,
}

impl From<(String, AliasEntry)> for ShellAlias {
    fn from((name, entry): (String, AliasEntry)) -> Self {
        Self {
            name,
            value: entry.value,
        }
    }
}
