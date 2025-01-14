pub mod alias;
pub mod cli;
pub mod dirs;
pub mod messages;
pub mod profile;
pub mod setup;
mod shell;
pub mod update;

pub use alias::*;
pub use messages::*;
pub use profile::*;

pub type Result<T> = anyhow::Result<T>;
