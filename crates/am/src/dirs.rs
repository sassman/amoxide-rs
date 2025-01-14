use std::path::PathBuf;

use dirs::home_dir;

/// Returns the path to the config directory
///
/// The config directory is located at `$HOME/.config/alias-manager`
/// Note: We decided against using the dirs::config_dir() because of the specific location on macos that is `~/Library/Application Support/alias-manager` and we want to have a consistent location across all platforms
pub fn config_dir() -> PathBuf {
    home_dir().unwrap().join(".config").join("alias-manager")
}

pub fn fish_config_dir() -> PathBuf {
    home_dir().unwrap().join(".config").join("fish")
}

pub fn fish_functions_dir() -> PathBuf {
    fish_config_dir().join("functions")
}
