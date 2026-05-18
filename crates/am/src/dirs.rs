use std::path::PathBuf;

/// Returns the path to the config directory.
///
/// Located at `$HOME/.config/amoxide`.
///
/// Under the `test-util` feature, `AMOXIDE_CONFIG_DIR` can override this so
/// completion fixture tests can point at a tempdir without touching the
/// user's real config. The override is not compiled into release builds.
pub fn config_dir() -> PathBuf {
    #[cfg(feature = "test-util")]
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
