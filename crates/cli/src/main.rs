use anyhow::bail;
use clap::Parser;

mod cli;

use cli::{AddCommands, Cli, Commands};
use context::Context;
use log::info;
use shell_manager_lib::*;
use shells::ShellBuilder;

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
                // todo: try to hand this over to clap
                let alias = if let Some(value) = value {
                    alias::Alias::from(value.clone())
                } else {
                    let ctx = if let Some(shell) = &cli.current_shell {
                        info!("Setting current shell to '{}'", shell);
                        let sh = ShellBuilder::new().with_name(shell).build()?;
                        Context::new(sh)
                    } else {
                        bail!("No shell context provided, see `--current-shell`");
                    };

                    alias::Alias::from_last_command(&ctx)?
                };
                if let Err(e) = alias::add::add_alias(name, &alias, *directory, *long) {
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
        Commands::Env { shell } => verbs::env(shell)?,
        Commands::Init { shell } => verbs::init(shell)?,
    }

    Ok(())
}
