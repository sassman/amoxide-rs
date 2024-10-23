use clap::Parser;

mod cli;

use cli::{AddCommands, Cli, Commands};
use shell_manager_lib::*;

fn main() -> anyhow::Result<()> {
    // setup env logger
    env_logger::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Add(add_command) => match add_command {
            AddCommands::Alias {
                name,
                value,
                directory,
                long,
            } => {
                if let Err(e) = alias::add::add_alias(name, value, *directory, *long) {
                    eprintln!("Failed to add alias: {}", e);
                }
            }
            AddCommands::Path {
                name,
                value,
                directory,
            } => {
                if *directory {
                    println!(
                        "Adding directory-specific path '{}' with value '{}'",
                        name, value
                    );
                } else {
                    println!("Adding path '{}' with value '{}'", name, value);
                }
            }
            AddCommands::Secret {
                name,
                value,
                directory,
            } => {
                if *directory {
                    println!(
                        "Adding directory-specific secret '{}' with value '{}'",
                        name, value
                    );
                } else {
                    println!("Adding secret '{}' with value '{}'", name, value);
                }
            }
        },
        Commands::Env => verbs::env()?,
        Commands::Init => verbs::init()?,
    }

    Ok(())
}
