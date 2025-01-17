use anyhow::bail;
use env_logger::Builder;
use log::{info, warn};
use std::io::Write;

use am::{
    cli::*,
    update::{update, AppModel},
    AddAliasProfile, Message,
};

fn setup_logging() {
    // setup env logger
    let filter_level = if !cfg!(debug_assertions) {
        "info"
    } else {
        "debug"
    };
    let mut builder = Builder::from_default_env();

    builder
        .filter_level(filter_level.parse().unwrap())
        .default_format()
        .format(|buf, record| writeln!(buf, "# {} - {}", record.level(), record.args()))
        .init();
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut model = AppModel::default();

    if let Some(shell) = cli.shell {
        update(&mut model, Message::SetShell(&shell))?;
    }

    if !matches!(&cli.command, Commands::Env { .. } | Commands::Init { .. }) {
        setup_logging();
    }

    if let Some(session_key) = &cli.session_key {
        update(&mut model, Message::RestoreState(session_key))?;
    } else {
        warn!("session key not provided, not restoring the state like active profile etc.");
    }

    let mut message = match &cli.command {
        Commands::Add(Alias {
            profile,
            name,
            command,
        }) => {
            let alias = if let Some(command) = command {
                command.join(" ")
            } else {
                info!("Fetching the last command from history");
                todo!("Fetching the last command from history is not implemented yet")
            };
            let profile = profile
                .as_deref()
                .map(|p| AddAliasProfile::Profile(p.to_owned()))
                .unwrap_or(AddAliasProfile::ActiveProfile);

            info!("Adding alias `{name}` with command `{alias}` to {profile}",);
            update(&mut model, Message::AddAlias(name.clone(), alias, profile))?;

            Message::SaveProfiles
        }
        Commands::Profiles => Message::ListProfiles,
        Commands::Profile(Profile {
            name,
            inherits,
            list,
            on_activate,
            print_full_init,
        }) => {
            info!("profile command");
            if *list {
                Message::ListProfiles
            } else if let Some(ref name) = name {
                info!("updating profile {name}");
                update(
                    &mut model,
                    Message::CreateOrUpdateProfile(name.as_str(), inherits),
                )?;
                update(&mut model, Message::SaveProfiles)?;

                if let Some(_on_activate) = on_activate {
                    warn!("todo: on_activate is not implemented yet");
                }
                info!("activating profile {name}");
                update(&mut model, Message::ActivateProfile(name))?;

                if *print_full_init {
                    Message::ListActiveAliases
                } else {
                    Message::DoNothing
                }
            } else {
                bail!(
                    "No profile name provided please use `am profile --list` to list all profiles"
                )
            }
        }
        Commands::Init { shell } | Commands::Env { shell } => Message::InitShell(shell),
    };

    while let Some(msg) = update(&mut model, message)? {
        message = msg;
    }

    if let Some(session_key) = &cli.session_key {
        update(&mut model, Message::SaveState(session_key))?;
    } else {
        warn!("session key not provided, not saving the state like active profile etc.");
    }

    Ok(())
}
