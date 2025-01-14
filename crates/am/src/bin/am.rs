use anyhow::bail;
use log::info;

use am::{
    cli::*,
    update::{update, AppModel},
    AddAliasProfile, Message,
};

fn main() -> anyhow::Result<()> {
    // setup env logger
    let filter_level = if !cfg!(debug_assertions) {
        "info"
    } else {
        "debug"
    };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(filter_level))
        .init();
    let cli = Cli::parse();
    let mut model = AppModel::default();

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
            Message::AddAlias(name.clone(), alias, profile)
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
                Message::LoadOrCreateProfile(name.as_str(), inherits)
            } else {
                bail!("No profile name provided or use the --list flag to list all profiles")
            }
        }
        Commands::Env { shell } => {
            info!("Setting up environment for {}", shell);
            Message::ListAliasesForShell(shell)
        }
        Commands::Init { shell } => Message::InitShell(shell),
    };

    while let Some(msg) = update(&mut model, message)? {
        message = msg;
    }

    Ok(())
}
