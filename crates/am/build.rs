use clap::CommandFactory;
use clap_complete::{
    aot::{Bash, Fish, PowerShell, Zsh},
    generate_to,
};

/// Minimal stub of the Shells enum for build-script shell-completion generation.
/// The real implementation lives in src/shell/shell.rs.
pub mod shell {
    #[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
    pub enum Shells {
        Fish,
        Zsh,
    }
}

include!("src/cli.rs");

fn main() {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    generate_to(Zsh, &mut cmd, &name, "../../completions/zsh").unwrap();
    generate_to(Bash, &mut cmd, &name, "../../completions/bash").unwrap();
    generate_to(Fish, &mut cmd, &name, "../../completions/fish").unwrap();
    generate_to(PowerShell, &mut cmd, &name, "../../completions/powershell").unwrap();
}
