#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum Shell {
    Bash,
    Brush,
    // Elvish,
    Fish,
    // Ksh,
    // Nushell,
    // Posix,
    Powershell,
    // Xonsh,
    Zsh,
    // #[cfg(windows)]
    // Cmd,
}
