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
        Bash,
        Fish,
        Powershell,
        Zsh,
    }
}

include!("src/cli.rs");

fn main() {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    // Write completions to OUT_DIR (always works, including in cargo publish --dry-run)
    let out_dir = std::env::var("OUT_DIR").unwrap();

    generate_to(Zsh, &mut cmd, &name, &out_dir).unwrap();
    generate_to(Bash, &mut cmd, &name, &out_dir).unwrap();
    generate_to(Fish, &mut cmd, &name, &out_dir).unwrap();
    generate_to(PowerShell, &mut cmd, &name, &out_dir).unwrap();

    // Also write to repo completions/ if running in the workspace (not in a tarball)
    let repo_completions = std::path::Path::new("../../completions");
    if repo_completions.exists() {
        generate_to(Zsh, &mut cmd, &name, repo_completions.join("zsh")).unwrap();
        generate_to(Bash, &mut cmd, &name, repo_completions.join("bash")).unwrap();
        generate_to(Fish, &mut cmd, &name, repo_completions.join("fish")).unwrap();
        generate_to(
            PowerShell,
            &mut cmd,
            &name,
            repo_completions.join("powershell"),
        )
        .unwrap();
    }
}
