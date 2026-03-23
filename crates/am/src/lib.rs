pub mod alias;
pub mod cli;
pub mod config;
pub mod dirs;
pub mod hook;
pub mod init;
pub mod messages;
pub mod profile;
pub mod project;
pub mod shell;
pub mod update;

pub use alias::*;
pub use config::*;
pub use messages::*;
pub use profile::*;
pub use project::*;

pub type Result<T> = anyhow::Result<T>;
