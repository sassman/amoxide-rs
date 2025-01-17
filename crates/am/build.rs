use clap::CommandFactory;
use clap_complete::{
    aot::{Bash, Fish, PowerShell, Zsh},
    generate_to,
};

include!("src/cli.rs");
#[allow(unused_imports)]
pub mod shell {
    include!("src/shell/mod.rs");
}

fn main() {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();

    generate_to(Zsh, &mut cmd, &name, "../../completions/zsh").unwrap();
    generate_to(Bash, &mut cmd, &name, "../../completions/bash").unwrap();
    generate_to(Fish, &mut cmd, &name, "../../completions/fish").unwrap();
    generate_to(PowerShell, &mut cmd, &name, "../../completions/powershell").unwrap();
}
