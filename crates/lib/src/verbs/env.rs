use crate::{context::Context, shells::ShellBuilder};

pub fn env(shell: &str) -> anyhow::Result<()> {
    let sh = ShellBuilder::new().with_name(shell).build()?;
    let ctx = Context::new(sh);

    // compose the whole env
    crate::alias::env::env_alias(&ctx)

    // TODO: next is env for paths
}
