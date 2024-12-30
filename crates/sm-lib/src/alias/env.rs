use std::{collections::HashMap, fs, io};

use anyhow::bail;
use dirs::config_dir;
use log::debug;

use crate::alias::{AliasConfig, ShellAlias};
use crate::context::Context;

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

    pub fn execute(self, ctx: &Context) {
        for alias in self.active_aliases.0 {
            println!("{}", ctx.shell().render_alias(&alias));
        }

        for unalias in self.inactive_aliases.0 {
            println!("{}", ctx.shell().render_unalias(&unalias));
        }
    }
}

pub fn current_shell_aliases(ctx: &Context) -> anyhow::Result<HashMap<String, ()>> {
    dbg!(format!("{}", ctx.cmd("alias")?));

    todo!("This function needs some kind of alias caching, that is fed via command line input, during the initialization in the specific sheell- rc file");
}

pub fn env_alias(ctx: &Context) -> anyhow::Result<()> {
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
                // todo: when an alias is not even set, then the unalias command causes an error message, so it souldn't be added to the inactive_aliases
                let is_alias_set = false;
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

    ap.execute(ctx);

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use crate::shells::Zsh;

    use super::*;

    #[test]
    fn test_current_aliases() {
        let a = current_shell_aliases(&Context::new(Box::new(Zsh))).unwrap();
        assert!(!a.is_empty());
    }

    #[test]
    fn test_alias_shell_out() {
        let x = String::from_utf8(
            Command::new("/bin/bash")
                .arg("-c")
                .arg("alias")
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();

        dbg!(x);
    }
}
