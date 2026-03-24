mod fish;
mod nix;
#[allow(clippy::module_inception)]
mod shell;
mod zsh;

pub use fish::*;
pub use nix::*;
pub use shell::*;
pub use zsh::*;
