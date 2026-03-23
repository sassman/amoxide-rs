use anyhow::bail;
use env_logger::Builder;
use log::info;
use std::io::Write;

use am::{
    cli::*,
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
    if !matches!(&cli.command, Commands::Init { .. } | Commands::Hook { .. }) {
        setup_logging();
    }

    let mut message = match &cli.command {
        Commands::Add(Alias {
            profile,
            name,
            command,
        }) => {
            let alias_cmd = match command {
                Some(parts) => parts.join(" "),
                None => bail!("No command provided. Usage: am add <name> <command...>"),
            };
            let target = profile
                .as_deref()
                .map(|p| AddAliasProfile::Profile(p.to_owned()))
                .unwrap_or(AddAliasProfile::ActiveProfile);

            info!("Adding alias `{name}` = `{alias_cmd}` to {target}");
            update(
                &mut model,
                Message::AddAlias(name.clone(), alias_cmd, target),
            )?;
            Message::SaveProfiles
        }
        Commands::Profiles => Message::ListProfiles,
        Commands::Profile(Profile {
            name,
            inherits,
            list,
        }) => {
            if *list {
                Message::ListProfiles
            } else if let Some(ref name) = name {
                update(
                    &mut model,
                    Message::CreateOrUpdateProfile(name.clone(), inherits.clone()),
                )?;
                update(&mut model, Message::SaveProfiles)?;
                Message::SaveConfig
            } else {
                bail!("No profile name provided. Use `am profile --list` to list profiles.")
            }
        }
        Commands::Init { shell } => Message::InitShell(shell.clone()),
        Commands::Hook { shell } => Message::Hook(shell.clone()),
    };

    while let Some(msg) = update(&mut model, message)? {
        message = msg;
    }

    Ok(())
}
