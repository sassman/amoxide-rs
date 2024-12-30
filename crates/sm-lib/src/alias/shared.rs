use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use crate::context::Context;

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasEntry {
    pub value: String,
    pub directory: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AliasConfig {
    pub aliases: HashMap<String, AliasEntry>,
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

impl TryFrom<String> for ShellAlias {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some((name, alias)) = value.split_once('=') {
            if name.starts_with("alias ") {
                let name = name.replacen("alias ", "", 1);
                Ok(Self {
                    name: name.to_string(),
                    value: alias.to_string(),
                })
            } else {
                bail!(
                    "Invalid alias format: `{}` should start with `alias `",
                    value
                );
            }
        } else {
            bail!(
                "Invalid alias format: `{}`, expected is `alias l=\"ls -lha\"`",
                value
            );
        }
    }
}

#[derive(Clone)]
pub struct Alias(String);

impl Alias {
    pub fn from_last_command(ctx: &Context) -> anyhow::Result<Self> {
        ctx.shell().last_command_from_history().map(Self::from)
    }
}

impl From<String> for Alias {
    fn from(value: String) -> Self {
        // todo: would be good to validate the alias somehow
        Self(value)
    }
}

impl std::fmt::Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
