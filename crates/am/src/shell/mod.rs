mod bash;
mod fish;
mod nix;
mod powershell;
#[allow(clippy::module_inception)]
mod shell;
mod zsh;

pub use bash::*;
pub use fish::*;
pub use nix::*;
pub use powershell::*;
pub use shell::*;
pub use zsh::*;
