mod bash;
mod brush;
mod fish;
mod nix;
mod powershell;
#[allow(clippy::module_inception)]
mod shell;
pub mod zsh;

pub use bash::*;
pub use brush::*;
pub use fish::*;
pub use nix::*;
pub use powershell::*;
pub use shell::*;
pub use zsh::*;

#[cfg(test)]
pub(crate) mod test_helpers {
    use crate::alias::AliasEntry;

    pub fn simple<'a>(name: &'a str, cmd: &'a str) -> AliasEntry<'a> {
        AliasEntry {
            name,
            command: cmd,
            raw: false,
        }
    }

    pub fn raw<'a>(name: &'a str, cmd: &'a str) -> AliasEntry<'a> {
        AliasEntry {
            name,
            command: cmd,
            raw: true,
        }
    }
}
