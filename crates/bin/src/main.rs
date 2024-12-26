use anyhow::bail;
use log::info;

use sm_cli::*;
use sm_lib::{context::Context, shells::ShellBuilder, *};

fn context(cli: &Cli) -> anyhow::Result<Context> {
    let shell = if let Some(shell) = &cli.current_shell {
        info!("Setting current shell to '{}'", shell);
        ShellBuilder::new().with_name(shell).build()?
    } else {
        bail!("No shell context provided, see `--current-shell`");
    };

    Ok(Context::new(shell))
}

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
                    let ctx = context(&cli)?;
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
                    todo!(
                        "Adding directory-specific path '{}' with value '{}'",
                        name,
                        value
                    );
                } else {
                    todo!("Adding path '{}' with value '{}'", name, value);
                }
            }
            AddCommands::Secret {
                name,
                value,
                directory,
            } => {
                if *directory {
                    todo!(
                        "Adding directory-specific secret '{}' with value '{}'",
                        name,
                        value
                    );
                } else {
                    todo!("Adding secret '{}' with value '{}'", name, value);
                }
            }
        },
        Commands::Env { shell } => verbs::env(shell)?,
        Commands::Init { shell } => verbs::init(shell)?,
        Commands::Import(import_command) => match import_command {
            ImportCommands::Alias => {
                let ctx = context(&cli)?;
                alias::import::import(&ctx)?;
            }
            ImportCommands::Path => {
                todo!("Importing paths");
            }
            ImportCommands::Secret => {
                todo!("Importing secrets");
            }
        },
    }

    Ok(())
}
