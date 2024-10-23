pub fn env() -> anyhow::Result<()> {
    // compose the whole env
    crate::alias::env::env_alias()

    // TODO: next is env for paths
}
