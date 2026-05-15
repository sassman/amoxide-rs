use std::path::PathBuf;

/// Returns the path to the config directory.
///
/// Honors `AMOXIDE_CONFIG_DIR` if set — used by completion fixture tests and
/// for ad-hoc local experimentation without touching the user's real config.
/// Otherwise located at `$HOME/.config/amoxide`.
pub fn config_dir() -> PathBuf {
    if let Some(override_path) = std::env::var_os("AMOXIDE_CONFIG_DIR") {
        return PathBuf::from(override_path);
    }
    dirs_lite::config_dir().unwrap().join("amoxide")
}

/// Returns the user's home directory.
pub fn home_dir() -> Option<PathBuf> {
    std::env::home_dir()
}

/// Compute a relative path from `from` to `to` using `../` components.
/// e.g. from `/a/b/c` to `/a/b/.aliases` → `../.aliases`
/// e.g. from `/a/b` to `/a/b/.aliases` → `.aliases`
pub fn relative_path(from: &std::path::Path, to: &std::path::Path) -> PathBuf {
    let from_parts: Vec<_> = from.components().collect();
    let to_parts: Vec<_> = to.components().collect();
    let common = from_parts
        .iter()
        .zip(to_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let ups = from_parts.len() - common;
    let mut rel = PathBuf::new();
    for _ in 0..ups {
        rel.push("..");
    }
    for part in &to_parts[common..] {
        rel.push(part);
    }
    rel
}
