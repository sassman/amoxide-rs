use anyhow::bail;
use env_logger::Builder;
use log::info;
use std::io::{BufRead, Write};

use am::{
    cli::*,
    dirs::relative_path,
    project::{ProjectAliases, ALIASES_FILE},
    update::{update, AppModel},
    AddAliasProfile, Message,
};

fn setup_logging() {
    let filter_level = if cfg!(debug_assertions) {
        "debug"
    } else {
        "warn"
    };
    let mut builder = Builder::from_default_env();
    builder
        .filter_level(filter_level.parse().unwrap())
        .format(|buf, record| writeln!(buf, "# {} - {}", record.level(), record.args()))
        .init();
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut model = AppModel::default();

    // Don't log for commands whose stdout is eval'd by the shell
    if !matches!(
        &cli.command,
        Commands::Init { .. } | Commands::Hook { .. } | Commands::Reload { .. }
    ) {
        setup_logging();
    }

    let mut message = match &cli.command {
        Commands::Add(Alias {
            profile,
            local,
            global,
            raw,
            name,
            command,
        }) => {
            let alias_cmd = match command {
                Some(parts) => parts.join(" "),
                None => bail!("No command provided. Usage: am add <name> <command...>"),
            };

            if *global {
                model.config.add_alias(name.clone(), alias_cmd, *raw);
                info!("Added global alias `{name}`");
                Message::SaveConfig
            } else if *local {
                add_local_alias(name, &alias_cmd, *raw)?;
                Message::DoNothing
            } else {
                let target = profile
                    .as_deref()
                    .map(|p| AddAliasProfile::Profile(p.to_owned()))
                    .unwrap_or(AddAliasProfile::ActiveProfile);

                info!("Adding alias `{name}` = `{alias_cmd}` to {target}");
                update(
                    &mut model,
                    Message::AddAlias(name.clone(), alias_cmd, target, *raw),
                )?;
                Message::SaveProfiles
            }
        }
        Commands::Remove {
            profile,
            global,
            name,
        } => {
            if *global {
                model.config.remove_alias(name)?;
                info!("Removed global alias `{name}`");
                Message::SaveConfig
            } else {
                let target = profile
                    .as_deref()
                    .map(|p| AddAliasProfile::Profile(p.to_owned()))
                    .unwrap_or(AddAliasProfile::ActiveProfile);

                info!("Removing alias `{name}` from {target}");
                update(&mut model, Message::RemoveAlias(name.clone(), target))?;
                Message::SaveProfiles
            }
        }
        Commands::Ls => Message::ListProfiles,
        Commands::Status => {
            println!("{}", am::status::run_status());
            Message::DoNothing
        }
        Commands::Profile { action } => match action.as_ref().unwrap_or(&ProfileAction::List) {
            ProfileAction::Add { name, inherits } => {
                update(
                    &mut model,
                    Message::CreateOrUpdateProfile(name.clone(), inherits.clone()),
                )?;
                update(&mut model, Message::SaveProfiles)?;
                Message::SaveConfig
            }
            ProfileAction::Set { name } => {
                update(&mut model, Message::ActivateProfile(name.clone()))?;
                Message::SaveConfig
            }
            ProfileAction::Remove { name, force } => {
                if !force {
                    let profile = model
                        .profile_config()
                        .get_profile_by_name(name)
                        .ok_or_else(|| anyhow::anyhow!("Profile '{name}' not found"))?;
                    if !profile.aliases.is_empty() {
                        let count = profile.aliases.iter().count();
                        eprint!(
                            "Profile '{name}' has {count} alias{}. Remove? [y/N] ",
                            if count == 1 { "" } else { "es" }
                        );
                        std::io::stderr().flush()?;
                        let mut input = String::new();
                        std::io::stdin().lock().read_line(&mut input)?;
                        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
                            println!("Cancelled.");
                            return Ok(());
                        }
                    }
                }
                update(&mut model, Message::RemoveProfile(name.clone()))?;
                update(&mut model, Message::SaveProfiles)?;
                Message::SaveConfig
            }
            ProfileAction::List => Message::ListProfiles,
        },
        Commands::Init { shell } => Message::InitShell(shell.clone()),
        Commands::Hook { shell } => Message::Hook(shell.clone()),
        Commands::Reload { shell } => Message::Reload(shell.clone()),
    };

    while let Some(msg) = update(&mut model, message)? {
        message = msg;
    }

    Ok(())
}

fn add_local_alias(name: &str, command: &str, raw: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let local_path = cwd.join(ALIASES_FILE);

    if local_path.exists() {
        // .aliases exists in CWD — add to it
        let mut project = ProjectAliases::load(&local_path)?;
        project.add_alias(name.to_string(), command.to_string(), raw);
        project.save(&local_path)?;
        println!("Added `{name}` to {ALIASES_FILE}");
        return Ok(());
    }

    // No .aliases in CWD — check if one exists up the tree
    if let Some(parent) = cwd.parent() {
        if let Some(existing_path) = ProjectAliases::find_path(parent)? {
            match prompt_existing_aliases(&existing_path, &cwd)? {
                Choice::Yes => {
                    let mut project = ProjectAliases::load(&existing_path)?;
                    project.add_alias(name.to_string(), command.to_string(), raw);
                    project.save(&existing_path)?;
                    let rel = relative_path(&cwd, &existing_path);
                    println!("Added `{name}` to {}", rel.display());
                    return Ok(());
                }
                Choice::No => {} // fall through to create new
                Choice::Cancel => {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
        }
    }

    // Create new .aliases in CWD
    let mut project = ProjectAliases::default();
    project.add_alias(name.to_string(), command.to_string(), raw);
    project.save(&local_path)?;
    println!("Created {ALIASES_FILE} with alias `{name}`");
    Ok(())
}

enum Choice {
    Yes,
    No,
    Cancel,
}

fn prompt_existing_aliases(
    path: &std::path::Path,
    cwd: &std::path::Path,
) -> anyhow::Result<Choice> {
    let rel = relative_path(cwd, path);
    eprint!(
        "Found existing {ALIASES_FILE} at {}\nAdd to that file instead? [N/y/c] ",
        rel.display()
    );
    std::io::stderr().flush()?;

    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => Ok(Choice::Yes),
        "c" | "cancel" => Ok(Choice::Cancel),
        _ => Ok(Choice::No),
    }
}
