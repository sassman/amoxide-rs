use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
};

use dirs::config_dir;

use super::{Alias, AliasConfig, AliasEntry};

pub fn add_alias(name: &str, value: &Alias, directory: bool, long: bool) -> anyhow::Result<()> {
    if long {
        todo!("Adding long alias '{}' with value:\n{}", name, value);
    } else if directory {
        println!(
            "Adding directory-specific alias '{}' with value '{}'",
            name, value
        );
    } else {
        println!("Adding alias '{}' with value '{}'", name, value);
    }

    let config_dir = config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Config directory not found"))?;
    let aliases_file_path = config_dir.join("shell-manager/aliases.toml");

    // Ensure the directory exists
    if let Some(parent) = aliases_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Read the existing aliases file if it exists
    let mut aliases = if aliases_file_path.exists() {
        let content = fs::read_to_string(&aliases_file_path)?;
        toml::from_str::<AliasConfig>(&content)?
    } else {
        AliasConfig {
            aliases: Default::default(),
        }
    };

    let alias_entry = AliasEntry {
        value: value.to_string(),
        directory: if directory {
            Some(std::env::current_dir()?)
        } else {
            None
        },
    };

    // Insert the alias into the top-level table
    aliases
        .aliases
        .entry(name.to_string())
        .and_modify(|e| {
            e.value = alias_entry.value.clone();
            e.directory = alias_entry.directory.clone();
        })
        .or_insert(alias_entry);

    // Write the updated aliases back to the file
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&aliases_file_path)?;
    file.write_all(toml::to_string(&aliases)?.as_bytes())?;

    Ok(())
}
