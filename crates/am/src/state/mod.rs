pub(crate) mod actions;
pub(crate) mod middleware;
pub(crate) mod reducers;
pub(crate) mod state;
pub(crate) mod store;

pub type Result<T> = anyhow::Result<T>;
