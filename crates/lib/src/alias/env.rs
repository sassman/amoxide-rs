use std::{collections::HashMap, fs, io};

use anyhow::bail;
use dirs::config_dir;
use log::debug;

use crate::alias::{AliasConfig, ShellAlias};
use crate::shells::{Shell, ShellBuilder};

#[derive(Debug, Default)]
struct ActiveAliases(Vec<ShellAlias>);

#[derive(Debug, Default)]
struct InactiveAliases(Vec<ShellAlias>);

#[derive(Debug, Default)]
pub struct ActionPlan {
    active_aliases: ActiveAliases,
    inactive_aliases: InactiveAliases,
}

impl ActionPlan {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn execute(self, shell: Box<dyn Shell>) {
        for alias in self.active_aliases.0 {
            println!("{}", shell.render_alias(&alias));
        }

        for unalias in self.inactive_aliases.0 {
            println!("{}", shell.render_unalias(&unalias));
        }
    }
}

#[allow(dead_code)]
pub fn current_shell_aliases() -> anyhow::Result<HashMap<String, ()>> {
    todo!("This function needs some kind of alias caching, that is fed via command line input, during the initialization in the specific sheell- rc file");
}

pub fn env_alias() -> anyhow::Result<()> {
    let config_dir = config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Config directory not found"))?;
    let aliases_file_path = config_dir.join("shell-manager/aliases.toml");
    debug!("Aliases file path: {:?}", aliases_file_path);

    if !aliases_file_path.exists() {
        bail!("Aliases file not found");
    }
    let aliases = if aliases_file_path.exists() {
        let content = fs::read_to_string(&aliases_file_path)?;
        toml::from_str::<AliasConfig>(&content)?
    } else {
        AliasConfig::default()
    };

    let mut ap = ActionPlan::new();
    let current_dir = std::env::current_dir()?;
    // let mut current_aliases = current_shell_aliases()?;

    for (name, value) in aliases.aliases.iter() {
        let sha = ShellAlias {
            name: name.clone(),
            value: value.value.clone(),
        };

        if let Some(directory) = &value.directory {
            // here we also allow if the current directory is a subdirectory of the alias directory
            if current_dir.starts_with(directory) {
                ap.active_aliases.0.push(sha);
            } else {
                // let is_alias_set = current_aliases.contains_key(name);
                let is_alias_set = true;
                if is_alias_set {
                    // current_aliases.remove(name);
                    ap.inactive_aliases.0.push(sha);
                } else {
                    debug!("Skipping alias `{}` as it is directory-specific and current directory does not match to `{}`", name, directory.display());
                }
            }
        } else {
            ap.active_aliases.0.push(sha);
        }
    }

    let shell = ShellBuilder::new().guess().build()?;
    ap.execute(shell);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_current_aliases() {
        let a = current_shell_aliases().unwrap();
        assert!(!a.is_empty());
    }
}
