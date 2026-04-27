use anyhow::bail;
use env_logger::Builder;
use log::info;
use std::io::Write;

use amoxide::{
    cli::*,
    dirs::relative_path,
    effects::Effect,
    env_vars,
    exchange::{render_suspicious_warning, scan_suspicious, ExportAll},
    import_export::{handle_export, handle_import, handle_share},
    profile::AliasCollection,
    project::{ProjectAliases, ALIASES_FILE},
    prompt::{ask_user, Answer},
    trust::compute_file_hash,
    update::{update, AppModel},
    AliasTarget, Echo, Message,
};

fn target_from_scope(scope: &TargetScopeArgs) -> AliasTarget {
    if scope.global {
        AliasTarget::Global
    } else if scope.local {
        AliasTarget::Local
    } else {
        scope
            .profile
            .as_deref()
            .map(|p| AliasTarget::Profile(p.to_owned()))
            .unwrap_or(AliasTarget::ActiveProfile)
    }
}

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
    // Guard against recursive invocation during alias scanning.
    // When `zsh -i -c alias` is spawned to enumerate existing shell aliases it
    // sources the user's startup files, which call `am sync` (or `am init`).
    // If those calls were allowed to run normally they could trigger another
    // scan, causing infinite recursion.  Exiting here makes `eval "$(...)"` a
    // no-op, which is safe.
    if std::env::var(env_vars::AM_DETECTING_ALIASES).is_ok() {
        return Ok(());
    }

    let cli = Cli::parse();
    let mut model = AppModel::default();

    // Don't log for commands whose stdout is eval'd by the shell
    if !matches!(&cli.command, Commands::Init { .. } | Commands::Sync { .. }) {
        setup_logging();
    }

    let message = match &cli.command {
        Commands::Add(Alias {
            scope,
            raw,
            name,
            command,
            sub,
        }) => {
            let target = target_from_scope(scope);

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
        Commands::Remove { scope, name, sub } => {
            let target = target_from_scope(scope);

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
        Commands::Ls { used } => Message::ListProfiles { used: *used },
        Commands::Status => {
            println!("{}", amoxide::status::run_status());
            return Ok(());
        }
        Commands::Use(ProfileUse {
            enable,
            disable,
            priority,
            inverse,
            names,
        })
        | Commands::Profile {
            action:
                Some(ProfileAction::Use(ProfileUse {
                    enable,
                    disable,
                    priority,
                    inverse,
                    names,
                })),
        } => {
            let ordered: Vec<String> = if *inverse {
                names.iter().rev().cloned().collect()
            } else {
                names.clone()
            };
            let msg = match priority {
                Some(n) => Message::UseProfilesAt(ordered, *n),
                None => {
                    if *enable {
                        Message::EnableProfiles(ordered)
                    } else if *disable {
                        Message::DeactivateProfiles(ordered)
                    } else {
                        Message::ToggleProfiles(ordered)
                    }
                }
            };
            let result = update(&mut model, msg)?;
            execute_effects(&mut model, result.effects)?;
            model.save_config()?;
            return Ok(());
        }
        Commands::Profile { action } => match action
            .as_ref()
            .unwrap_or(&ProfileAction::List { used: false })
        {
            ProfileAction::Add { name } => {
                let result = update(&mut model, Message::CreateProfile(name.clone()))?;
                execute_effects(&mut model, result.effects)?;
                model.save_config()?;
                return Ok(());
            }
            ProfileAction::Remove { name, force } => {
                if !force {
                    let profile = model
                        .profile_config()
                        .get_profile_by_name(name)
                        .ok_or_else(|| anyhow::anyhow!("Profile '{name}' not found"))?;
                    if !profile.is_empty() {
                        let alias_count = profile.aliases.iter().count();
                        let subcmd_count = profile.subcommands.as_ref().len();
                        let question = match (alias_count, subcmd_count) {
                            (a, 0) => format!(
                                "Profile '{name}' has {a} alias{}. Remove?",
                                if a == 1 { "" } else { "es" }
                            ),
                            (0, s) => format!(
                                "Profile '{name}' has {s} subcommand alias{}. Remove?",
                                if s == 1 { "" } else { "es" }
                            ),
                            (a, s) => format!(
                                "Profile '{name}' has {a} alias{} and {s} subcommand alias{}. Remove?",
                                if a == 1 { "" } else { "es" },
                                if s == 1 { "" } else { "es" },
                            ),
                        };
                        if ask_user(&question, Answer::No, false, &mut std::io::stdin().lock())?
                            != Answer::Yes
                        {
                            println!("Cancelled.");
                            return Ok(());
                        }
                    }
                }
                let result = update(&mut model, Message::RemoveProfile(name.clone()))?;
                execute_effects(&mut model, result.effects)?;
                model.save_config()?;
                return Ok(());
            }
            ProfileAction::List { used } => Message::ListProfiles { used: *used },
            ProfileAction::Use(_) => unreachable!("handled by outer match arm"),
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
                local_subcommands: project.subcommands.clone(),
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
            let subcmd_groups = project.subcommands.group_by_program();
            if !subcmd_groups.is_empty() {
                println!();
                for (program, entries) in &subcmd_groups {
                    println!("  {program} (subcommands):");
                    let max_short_len = entries
                        .iter()
                        .map(|e| e.short_subcommands.join(" ").len())
                        .max()
                        .unwrap_or(0);
                    for entry in entries {
                        let shorts = entry.short_subcommands.join(" ");
                        let longs = entry.long_subcommands.join(" ");
                        println!(
                            "    {:width$} \u{2192} {longs}",
                            shorts,
                            width = max_short_len
                        );
                    }
                }
            }
            println!();

            let has_subcommands = !project.subcommands.is_empty();
            let prompt = if has_subcommands {
                "Trust these aliases and subcommand aliases?"
            } else {
                "Trust these aliases?"
            };

            // Prompt
            let answer = ask_user(prompt, Answer::Yes, false, &mut std::io::stdin().lock())?;

            if answer == Answer::Yes {
                let result = update(&mut model, Message::Trust)?;
                execute_effects(&mut model, result.effects)?;
                // The shell wrapper calls `am sync` after this, which loads
                // the aliases and shows the load message.
            } else {
                let result = update(&mut model, Message::Untrust { forget: false })?;
                execute_effects(&mut model, result.effects)?;
            }
            return Ok(());
        }
        Commands::Untrust { forget } => {
            let result = update(&mut model, Message::Untrust { forget: *forget })?;
            execute_effects(&mut model, result.effects)?;
            return Ok(());
        }
        Commands::Init { shell, force } => Message::InitShell(shell.clone(), *force),
        Commands::Sync { shell, quiet } => Message::Sync(shell.clone(), *quiet),
        Commands::Var { action } => match action {
            VarAction::Set { scope, name, value } => Message::SetVar {
                target: target_from_scope(scope),
                name: name.clone(),
                value: value.clone(),
            },
            VarAction::Unset { scope, name } => Message::UnsetVar {
                target: target_from_scope(scope),
                name: name.clone(),
            },
            VarAction::Get { scope, name } => Message::GetVar {
                target: target_from_scope(scope),
                name: name.clone(),
            },
            VarAction::List { scope } => {
                let target = if scope.global || scope.local || scope.profile.is_some() {
                    Some(target_from_scope(scope))
                } else {
                    None
                };
                Message::ListVars { target }
            }
        },
    };

    let result = update(&mut model, message)?;
    execute_effects(&mut model, result.effects)?;
    if let Some(msg) = result.next {
        let follow_up = update(&mut model, msg)?;
        execute_effects(&mut model, follow_up.effects)?;
    }

    Ok(())
}

fn execute_effects(model: &mut AppModel, effects: Vec<Effect>) -> anyhow::Result<()> {
    let has_local_mutation = effects.iter().any(|e| {
        matches!(
            e,
            Effect::AddLocalAlias { .. }
                | Effect::RemoveLocalAlias { .. }
                | Effect::AddLocalSubcommand { .. }
                | Effect::RemoveLocalSubcommand { .. }
                | Effect::AddLocalVar { .. }
                | Effect::RemoveLocalVar { .. }
        )
    });

    for effect in effects {
        match effect {
            Effect::SaveConfig => model.save_config()?,
            Effect::SaveSession => model.save_session()?,
            Effect::SaveProfiles => model.save_profiles()?,
            Effect::AddLocalAlias { name, cmd, raw } => add_local_alias(&name, &cmd, raw)?,
            Effect::RemoveLocalAlias { name } => remove_local_alias(&name)?,
            Effect::AddLocalSubcommand {
                key,
                long_subcommands,
            } => add_local_subcommand(&key, &long_subcommands)?,
            Effect::RemoveLocalSubcommand { key } => remove_local_subcommand(&key)?,
            Effect::AddLocalVar { name, value } => add_local_var(&name, &value)?,
            Effect::RemoveLocalVar { name } => remove_local_var(&name)?,
            Effect::Print(text) => println!("{text}"),
            Effect::PrintLines(lines) => {
                let output: String = lines
                    .into_iter()
                    .filter_map(|e| match e {
                        Echo::Line(s) => Some(s),
                        Echo::Silent => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                if !output.is_empty() {
                    print!("{output}");
                }
            }
            Effect::RenderSync(outcome) => {
                let echo_lines = outcome.render(&model.config.logging);
                execute_effects(model, vec![Effect::PrintLines(echo_lines)])?;
            }
            Effect::SaveSecurity => model.save_security()?,
        }
    }

    // After local alias mutations, update the security hash
    if has_local_mutation {
        if let Some(path) = model.project_path() {
            let path = path.to_path_buf();
            let new_hash = compute_file_hash(&path)?;
            model.security_config_mut().update_hash(&path, &new_hash);
            model.save_security()?;
        }
    }

    Ok(())
}

/// Add or update an entry in the local `.aliases` file.
///
/// Resolves the target file in this order: existing `.aliases` in CWD;
/// otherwise prompt to add to a `.aliases` discovered in a parent directory;
/// otherwise create a new `.aliases` in CWD.
///
/// `item_desc` is the human-readable description of what's being added,
/// e.g. `"alias \`gs\`"` or `"var \`path\`"` — used in the success message.
fn upsert_local_aliases<F>(item_desc: &str, mutate: F) -> anyhow::Result<()>
where
    F: FnOnce(&mut ProjectAliases) -> anyhow::Result<()>,
{
    let cwd = std::env::current_dir()?;
    let local_path = cwd.join(ALIASES_FILE);

    // Case 1: .aliases in CWD — append.
    if local_path.exists() {
        let mut project = ProjectAliases::load(&local_path)?;
        mutate(&mut project)?;
        project.save(&local_path)?;
        println!("Added {item_desc} to {ALIASES_FILE}");
        return Ok(());
    }

    // Case 2: .aliases in a parent dir — prompt before adding there.
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
                    mutate(&mut project)?;
                    project.save(&existing_path)?;
                    println!("Added {item_desc} to {}", rel.display());
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

    // Case 3: Create new .aliases in CWD.
    let mut project = ProjectAliases::default();
    mutate(&mut project)?;
    project.save(&local_path)?;
    println!("Created {ALIASES_FILE} with {item_desc}");
    Ok(())
}

/// Remove an entry from an existing local `.aliases` file (walking up from CWD).
/// Errors if no `.aliases` is found in the directory tree.
fn mutate_existing_local<F>(verb: &str, item_desc: &str, mutate: F) -> anyhow::Result<()>
where
    F: FnOnce(&mut ProjectAliases) -> anyhow::Result<()>,
{
    let cwd = std::env::current_dir()?;
    let path = ProjectAliases::find_local_path()
        .ok_or_else(|| anyhow::anyhow!("No {ALIASES_FILE} found"))?;
    let mut project = ProjectAliases::load(&path)?;
    mutate(&mut project)?;
    project.save(&path)?;
    let rel = relative_path(&cwd, &path);
    println!("{verb} {item_desc} from {}", rel.display());
    Ok(())
}

fn add_local_alias(name: &str, command: &str, raw: bool) -> anyhow::Result<()> {
    upsert_local_aliases(&format!("alias `{name}`"), |project| {
        project.add_alias(name.to_string(), command.to_string(), raw);
        Ok(())
    })
}

fn remove_local_alias(name: &str) -> anyhow::Result<()> {
    mutate_existing_local("Removed", &format!("alias `{name}`"), |project| {
        project.remove_alias(name)?;
        Ok(())
    })
}

fn add_local_subcommand(key: &str, long_subcommands: &[String]) -> anyhow::Result<()> {
    upsert_local_aliases(&format!("subcommand alias `{key}`"), |project| {
        project.add_subcommand(key.to_string(), long_subcommands.to_vec());
        Ok(())
    })
}

fn remove_local_subcommand(key: &str) -> anyhow::Result<()> {
    mutate_existing_local("Removed", &format!("subcommand alias `{key}`"), |project| {
        project.remove_subcommand(key)?;
        Ok(())
    })
}

fn add_local_var(name: &str, value: &str) -> anyhow::Result<()> {
    let parsed = amoxide::vars::VarName::parse(name).map_err(|e| anyhow::anyhow!("{e}"))?;
    upsert_local_aliases(&format!("var `{name}`"), |project| {
        project.set_var(parsed, value.to_string());
        Ok(())
    })
}

fn remove_local_var(name: &str) -> anyhow::Result<()> {
    let parsed = amoxide::vars::VarName::parse(name).map_err(|e| anyhow::anyhow!("{e}"))?;
    mutate_existing_local("Unset", &format!("var `{name}`"), |project| {
        project
            .unset_var(&parsed)
            .ok_or_else(|| anyhow::anyhow!("variable '{name}' not found in {ALIASES_FILE}"))?;
        Ok(())
    })
}
