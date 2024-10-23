use anyhow::bail;

use crate::shells::ShellBuilder;

pub fn init(shell: &str) -> anyhow::Result<()> {
    let sh = ShellBuilder::new().with_name(shell).build()?;

    let sh = format!("{sh:?}");
    match sh.to_lowercase().as_str() {
        "zsh" => {
            println!("# zsh initialization for shell-manager");
            println!("# put this in your ~/.zshrc");
            println!(r#"chpwd() {{ eval "$(sm env)" }}"#);
            println!("# the following will bring auto-completion to your shell");
            println!("{}", include_str!("../../../../completions/zsh/_sm"));
        }
        _ => {
            bail!("Unsupported shell")
        }
    }

    Ok(())
}
