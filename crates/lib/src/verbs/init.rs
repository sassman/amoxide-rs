use anyhow::{bail, Context};

pub fn init() -> anyhow::Result<()> {
    match std::env::var("SHELL").context("$SHELL not set")?.as_str() {
        "/bin/zsh" => {
            println!("# zsh initialization for shell-manager");
            println!("# put this in your ~/.zshrc");
            println!(r#"chpwd() {{ eval "$(sm env)" }}"#);
        }
        _ => {
            bail!("Unsupported shell");
        }
    }

    Ok(())
}
