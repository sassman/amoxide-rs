use std::path::PathBuf;

use dirs::home_dir;

/// Returns the path to the config directory
///
/// Located at `$HOME/.config/alias-manager`
pub fn config_dir() -> PathBuf {
    home_dir().unwrap().join(".config").join("alias-manager")
}
