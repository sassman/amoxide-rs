use log::info;

use am::{
    cli::*,
    update::{update, AppModel},
    Message,
};

fn main() -> anyhow::Result<()> {
    // setup env logger
    if !cfg!(debug_assertions) {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    }
    let cli = Cli::parse();
    let mut model = AppModel::default();

    match &cli.command {
        Commands::Add(Alias { name, command }) => {
            let alias = if let Some(command) = command {
                command.clone()
            } else {
                info!("Fetching the last command from history");
                todo!("Fetching the last command from history is not implemented yet")
            };
            info!("Adding alias {} for command {}", name, alias);
            todo!();
        }
        Commands::Profile(Profile {
            name,
            inherits,
            list,
        }) => {
            if *list || name.is_none() {
                info!("Listing all profiles");
                update(&mut model, Message::ListProfiles);
                return Ok(());
            }
            let Some(name) = name else { unreachable!() };
            update(&mut model, Message::LoadOrCreateProfile(name, inherits));
            Ok(())
        }
        Commands::Env { shell } => {
            info!("Setting up environment for {}", shell);
            todo!()
        }
    }
}
