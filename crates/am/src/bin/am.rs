use anyhow::bail;
use env_logger::Builder;
use log::info;
use std::io::Write;

use amoxide::{
    cli::*,
    dirs::relative_path,
    effects::Effect,
    exchange::{render_suspicious_warning, scan_suspicious, ExportAll},
    import_export::{handle_export, handle_import, handle_share},
    project::{ProjectAliases, ALIASES_FILE},
    prompt::{ask_user, Answer},
    trust::compute_file_hash,
    update::{update, AppModel},
    AliasTarget, Message,
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

    let message = match &cli.command {
        Commands::Add(Alias {
            profile,
            local,
            global,
            raw,
            name,
            command,
            sub,
        }) => {
            let target = if *global {
                AliasTarget::Global
            } else if *local {
                AliasTarget::Local
            } else {
                profile
                    .as_deref()
                    .map(|p| AliasTarget::Profile(p.to_owned()))
                    .unwrap_or(AliasTarget::ActiveProfile)
            };

            // Check if this is a subcommand alias
            let is_colon_notation = name.contains(':');
            let has_sub_flag = !sub.is_empty();

            if is_colon_notation || has_sub_flag {
                // Build the subcommand key and long_subcommands
                let (key, long_subcommands) = if is_colon_notation {
                    // Colon notation: jj:ab abandon or jj:b:l branch list
                    let cmd_parts: Vec<String> = match command {
                        Some(parts) => parts.clone(),
                        None => bail!("No expansion provided. Usage: am add jj:ab abandon"),
                    };
                    (name.clone(), cmd_parts)
                } else {
                    // --sub flag: jj --sub ab abandon --sub b branch
                    // sub is a flat Vec: ["ab", "abandon", "b", "branch", ...]
                    let pairs: Vec<(&str, &str)> = sub
                        .chunks(2)
                        .map(|chunk| (chunk[0].as_str(), chunk[1].as_str()))
                        .collect();
                    let key = std::iter::once(name.as_str())
                        .chain(pairs.iter().map(|(short, _)| *short))
                        .collect::<Vec<_>>()
                        .join(":");
                    let longs: Vec<String> =
                        pairs.iter().map(|(_, long)| long.to_string()).collect();
                    (key, longs)
                };

                // Validate
                let _entry = amoxide::subcommand::SubcommandEntry::parse_key(
                    &key,
                    long_subcommands.clone(),
                )?;

                info!("Adding subcommand alias `{key}` to {target}");
                Message::AddSubcommandAlias(key, long_subcommands, target)
            } else {
                // Regular alias
                let alias_cmd = match command {
                    Some(parts) => parts.join(" "),
                    None => bail!("No command provided. Usage: am add <name> <command...>"),
                };
                info!("Adding alias `{name}` = `{alias_cmd}` to {target}");
                Message::AddAlias(name.clone(), alias_cmd, target, *raw)
            }
        }
        Commands::Remove {
            profile,
            global,
            name,
            sub,
        } => {
            let target = if *global {
                AliasTarget::Global
            } else {
                profile
                    .as_deref()
                    .map(|p| AliasTarget::Profile(p.to_owned()))
                    .unwrap_or(AliasTarget::ActiveProfile)
            };

            let is_colon_notation = name.contains(':');
            let has_sub_flag = !sub.is_empty();

            if is_colon_notation || has_sub_flag {
                let key = if is_colon_notation {
                    name.clone()
                } else {
                    std::iter::once(name.as_str())
                        .chain(sub.iter().map(|s| s.as_str()))
                        .collect::<Vec<_>>()
                        .join(":")
                };
                info!("Removing subcommand alias `{key}` from {target}");
                Message::RemoveSubcommandAlias(key, target)
            } else {
                info!("Removing alias `{name}` from {target}");
                Message::RemoveAlias(name.clone(), target)
            }
        }
        Commands::Ls => Message::ListProfiles,
        Commands::Status => {
            println!("{}", amoxide::status::run_status());
            return Ok(());
        }
        Commands::Use {
            names,
            priority,
            inverse,
        } => {
            let ordered: Vec<String> = if *inverse {
                names.iter().rev().cloned().collect()
            } else {
                names.clone()
            };
            let msg = match priority {
                Some(n) => Message::UseProfilesAt(ordered, *n),
                None => Message::ToggleProfiles(ordered),
            };
            let result = update(&mut model, msg)?;
            execute_effects(&mut model, &result.effects)?;
            model.config.save()?;
            return Ok(());
        }
        Commands::Profile { action } => match action.as_ref().unwrap_or(&ProfileAction::List) {
            ProfileAction::Add { name } => {
                let result = update(&mut model, Message::CreateProfile(name.clone()))?;
                execute_effects(&mut model, &result.effects)?;
                model.config.save()?;
                return Ok(());
            }
            ProfileAction::Use {
                names,
                priority,
                inverse,
            } => {
                let ordered: Vec<String> = if *inverse {
                    names.iter().rev().cloned().collect()
                } else {
                    names.clone()
                };
                let msg = match priority {
                    Some(n) => Message::UseProfilesAt(ordered, *n),
                    None => Message::ToggleProfiles(ordered),
                };
                let result = update(&mut model, msg)?;
                execute_effects(&mut model, &result.effects)?;
                model.config.save()?;
                return Ok(());
            }
            ProfileAction::Remove { name, force } => {
                if !force {
                    let profile = model
                        .profile_config()
                        .get_profile_by_name(name)
                        .ok_or_else(|| anyhow::anyhow!("Profile '{name}' not found"))?;
                    if !profile.aliases.is_empty() {
                        let count = profile.aliases.iter().count();
                        let question = format!(
                            "Profile '{name}' has {count} alias{}. Remove?",
                            if count == 1 { "" } else { "es" }
                        );
                        if ask_user(&question, Answer::No, false, &mut std::io::stdin().lock())?
                            != Answer::Yes
                        {
                            println!("Cancelled.");
                            return Ok(());
                        }
                    }
                }
                let result = update(&mut model, Message::RemoveProfile(name.clone()))?;
                execute_effects(&mut model, &result.effects)?;
                model.config.save()?;
                return Ok(());
            }
            ProfileAction::List => Message::ListProfiles,
        },
        Commands::Setup { shell } => {
            amoxide::setup::run_setup(shell)?;
            return Ok(());
        }
        Commands::Tui => {
            use std::process::Command;
            let status = Command::new("am-tui").status();
            match status {
                Ok(s) if s.success() => return Ok(()),
                Ok(s) => anyhow::bail!("am-tui exited with status {}", s.code().unwrap_or(1)),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    eprintln!("am-tui is not installed. Install it with:\n");
                    eprintln!("  Homebrew:        brew install sassman/tap/amoxide-tui");
                    eprintln!("  Shell Script:    curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.sh | sh");
                    eprintln!("  PowerShell:      powershell -ExecutionPolicy Bypass -c \"irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.ps1 | iex\"");
                    eprintln!("  Cargo:           cargo install amoxide-tui");
                    eprintln!("  Cargo binstall:  cargo binstall amoxide-tui\n");
                    anyhow::bail!("am-tui not found");
                }
                Err(e) => return Err(e.into()),
            }
        }
        Commands::Export(args) => {
            let output = handle_export(&model, args)?;
            print!("{output}");
            return Ok(());
        }
        Commands::Import(args) => {
            handle_import(&mut model, args)?;
            return Ok(());
        }
        Commands::Share(args) => {
            let output = handle_share(args);
            print!("{output}");
            return Ok(());
        }
        Commands::Trust => {
            // Interactive trust review flow
            let project_trust = model.project_trust();
            let path = match project_trust {
                Some(t) => t.path().to_path_buf(),
                None => bail!("No .aliases file found in directory tree"),
            };

            if project_trust.map(|t| t.is_trusted()).unwrap_or(false) {
                println!("Already trusted: {}", path.display());
                return Ok(());
            }

            // Parse and review
            let project = ProjectAliases::load(&path)?;

            // Check for suspicious characters (reuse from exchange)
            let export = ExportAll {
                local_aliases: project.aliases.clone(),
                ..Default::default()
            };
            let findings = scan_suspicious(&export);
            if !findings.is_empty() {
                eprint!("{}", render_suspicious_warning(&findings));
            }

            // Show aliases for review — display filename + parent directory for context
            let filename = path
                .file_name()
                .map(|f| f.to_string_lossy())
                .unwrap_or_default();
            let folder = path
                .parent()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            println!("Reviewing {filename} at {folder}");
            println!();
            let max_name_len = project
                .aliases
                .iter()
                .map(|(n, _)| n.as_ref().len())
                .max()
                .unwrap_or(0);
            for (alias_name, alias_value) in project.aliases.iter() {
                let name = alias_name.as_ref();
                let cmd = alias_value.command();
                println!("  {:width$} \u{2192} {cmd}", name, width = max_name_len);
            }
            println!();

            // Prompt
            let answer = ask_user(
                "Trust these aliases?",
                Answer::Yes,
                false,
                &mut std::io::stdin().lock(),
            )?;

            if answer == Answer::Yes {
                let result = update(&mut model, Message::Trust)?;
                execute_effects(&mut model, &result.effects)?;
                // The shell wrapper calls `am hook` after this, which loads
                // the aliases and shows the load message.
            } else {
                let result = update(&mut model, Message::Untrust { forget: false })?;
                execute_effects(&mut model, &result.effects)?;
            }
            return Ok(());
        }
        Commands::Untrust { forget } => {
            let result = update(&mut model, Message::Untrust { forget: *forget })?;
            execute_effects(&mut model, &result.effects)?;
            return Ok(());
        }
        Commands::Init { shell } => Message::InitShell(shell.clone()),
        Commands::Hook { shell, quiet } => Message::Hook(shell.clone(), *quiet),
        Commands::Reload { shell } => Message::Reload(shell.clone()),
    };

    let result = update(&mut model, message)?;
    execute_effects(&mut model, &result.effects)?;
    if let Some(msg) = result.next {
        let follow_up = update(&mut model, msg)?;
        execute_effects(&mut model, &follow_up.effects)?;
    }

    Ok(())
}

fn execute_effects(model: &mut AppModel, effects: &[Effect]) -> anyhow::Result<()> {
    let has_local_mutation = effects.iter().any(|e| {
        matches!(
            e,
            Effect::AddLocalAlias { .. }
                | Effect::RemoveLocalAlias { .. }
                | Effect::AddLocalSubcommand { .. }
                | Effect::RemoveLocalSubcommand { .. }
        )
    });

    for effect in effects {
        match effect {
            Effect::SaveConfig => model.config.save()?,
            Effect::SaveProfiles => model.profile_config().save()?,
            Effect::AddLocalAlias { name, cmd, raw } => add_local_alias(name, cmd, *raw)?,
            Effect::RemoveLocalAlias { name } => remove_local_alias(name)?,
            Effect::AddLocalSubcommand {
                key,
                long_subcommands,
            } => add_local_subcommand(key, long_subcommands)?,
            Effect::RemoveLocalSubcommand { key } => remove_local_subcommand(key)?,
            Effect::Print(text) => println!("{text}"),
            Effect::SaveSecurity => model.security_config().save()?,
        }
    }

    // After local alias mutations, update the security hash
    if has_local_mutation {
        if let Some(path) = model.project_path() {
            let path = path.to_path_buf();
            let new_hash = compute_file_hash(&path)?;
            model.security_config_mut().update_hash(&path, &new_hash);
            model.security_config().save()?;
        }
    }

    Ok(())
}

fn add_local_alias(name: &str, command: &str, raw: bool) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let local_path = cwd.join(ALIASES_FILE);

    if local_path.exists() {
        let mut project = ProjectAliases::load(&local_path)?;
        project.add_alias(name.to_string(), command.to_string(), raw);
        project.save(&local_path)?;
        println!("Added `{name}` to {ALIASES_FILE}");
        return Ok(());
    }

    // No .aliases in CWD — check if one exists up the tree
    if let Some(parent) = cwd.parent() {
        if let Some(existing_path) = ProjectAliases::find_path(parent)? {
            let rel = relative_path(&cwd, &existing_path);
            let question = format!(
                "Found existing {ALIASES_FILE} at {}\nAdd to that file instead?",
                rel.display()
            );
            match ask_user(&question, Answer::No, true, &mut std::io::stdin().lock())? {
                Answer::Yes => {
                    let mut project = ProjectAliases::load(&existing_path)?;
                    project.add_alias(name.to_string(), command.to_string(), raw);
                    project.save(&existing_path)?;
                    println!("Added `{name}` to {}", rel.display());
                    return Ok(());
                }
                Answer::No => {} // fall through to create new
                Answer::Cancel => {
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

fn remove_local_alias(name: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let path = ProjectAliases::remove_from_local(name)?;
    let rel = relative_path(&cwd, &path);
    println!("Removed `{name}` from {}", rel.display());
    Ok(())
}

fn add_local_subcommand(key: &str, long_subcommands: &[String]) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let local_path = cwd.join(ALIASES_FILE);

    if local_path.exists() {
        let mut project = ProjectAliases::load(&local_path)?;
        project.add_subcommand(key.to_string(), long_subcommands.to_vec());
        project.save(&local_path)?;
        println!("Added subcommand alias `{key}` to {ALIASES_FILE}");
        return Ok(());
    }

    // No .aliases in CWD — check if one exists up the tree
    if let Some(parent) = cwd.parent() {
        if let Some(existing_path) = ProjectAliases::find_path(parent)? {
            let rel = relative_path(&cwd, &existing_path);
            let question = format!(
                "Found existing {ALIASES_FILE} at {}\nAdd to that file instead?",
                rel.display()
            );
            match ask_user(&question, Answer::No, true, &mut std::io::stdin().lock())? {
                Answer::Yes => {
                    let mut project = ProjectAliases::load(&existing_path)?;
                    project.add_subcommand(key.to_string(), long_subcommands.to_vec());
                    project.save(&existing_path)?;
                    println!("Added subcommand alias `{key}` to {}", rel.display());
                    return Ok(());
                }
                Answer::No => {} // fall through to create new
                Answer::Cancel => {
                    println!("Cancelled.");
                    return Ok(());
                }
            }
        }
    }

    // Create new .aliases in CWD
    let mut project = ProjectAliases::default();
    project.add_subcommand(key.to_string(), long_subcommands.to_vec());
    project.save(&local_path)?;
    println!("Created {ALIASES_FILE} with subcommand alias `{key}`");
    Ok(())
}

fn remove_local_subcommand(key: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let path = ProjectAliases::find_local_path()
        .ok_or_else(|| anyhow::anyhow!("No {ALIASES_FILE} found"))?;
    let mut project = ProjectAliases::load(&path)?;
    project.remove_subcommand(key)?;
    project.save(&path)?;
    let rel = relative_path(&cwd, &path);
    println!("Removed subcommand alias `{key}` from {}", rel.display());
    Ok(())
}
