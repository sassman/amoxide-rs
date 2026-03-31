pub mod alias;
pub mod cli;
pub mod config;
pub mod dirs;
pub mod display;
pub mod effects;
pub mod hook;
pub mod init;
pub mod messages;
pub mod profile;
pub mod project;
pub mod setup;
pub mod shell;
pub mod status;
pub mod update;

pub use alias::*;
pub use config::*;
pub use effects::*;
pub use messages::*;
pub use profile::*;
pub use project::*;

pub type Result<T> = anyhow::Result<T>;
