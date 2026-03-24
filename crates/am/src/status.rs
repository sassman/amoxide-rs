use std::path::Path;

use crate::dirs::{config_dir, home_dir};

/// Result of a single status check.
pub enum Check {
    Ok(String),
    Warn(String),
}

impl Check {
    fn icon(&self) -> &str {
        match self {
            Check::Ok(_) => "ok",
            Check::Warn(_) => "!!",
        }
    }

    fn message(&self) -> &str {
        match self {
            Check::Ok(msg) | Check::Warn(msg) => msg,
        }
    }
}

/// Detect the current shell from $SHELL env var.
pub fn detect_shell() -> Check {
    match std::env::var("SHELL") {
        Ok(shell) => {
            let name = Path::new(&shell)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| shell.clone());
            Check::Ok(format!("{name} ({shell})"))
        }
        Err(_) => Check::Warn("$SHELL not set — cannot detect shell".to_string()),
    }
}

/// Check if the shell config file contains `am init`.
pub fn check_shell_config() -> Check {
    let shell = std::env::var("SHELL").unwrap_or_default();
    let shell_name = Path::new(&shell)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let (config_path, init_line) = match shell_name.as_str() {
        "fish" => (
            home_dir().map(|h| h.join(".config/fish/config.fish")),
            "am init fish | source",
        ),
        "zsh" => (
            home_dir().map(|h| h.join(".zshrc")),
            "eval \"$(am init zsh)\"",
        ),
        "bash" => (
            home_dir().map(|h| h.join(".bashrc")),
            "eval \"$(am init bash)\"",
        ),
        _ => return Check::Warn(format!("unknown shell: {shell_name}")),
    };

    let Some(path) = config_path else {
        return Check::Warn("cannot determine home directory".to_string());
    };

    if !path.exists() {
        return Check::Warn(format!(
            "{} does not exist\n          add: {init_line}",
            path.display()
        ));
    }

    match std::fs::read_to_string(&path) {
        Ok(content) if content.contains("am init") => {
            Check::Ok(format!("{} contains `am init`", path.display()))
        }
        Ok(_) => Check::Warn(format!(
            "{} missing `am init`\n          add: {init_line}",
            path.display()
        )),
        Err(e) => Check::Warn(format!("cannot read {}: {e}", path.display())),
    }
}

/// Check if the config directory and config.toml exist.
pub fn check_config_dir() -> Check {
    let dir = config_dir();
    if !dir.exists() {
        return Check::Warn(format!("{} not found — run `am init` first", dir.display()));
    }

    let config_file = dir.join("config.toml");
    let profiles_file = dir.join("profiles.toml");

    let mut parts = vec![format!("{}", dir.display())];
    if config_file.exists() {
        parts.push("config.toml".to_string());
    }
    if profiles_file.exists() {
        parts.push("profiles.toml".to_string());
    }

    if !config_file.exists() && !profiles_file.exists() {
        Check::Warn(format!(
            "{} exists but no config files — run `am init` first",
            dir.display()
        ))
    } else {
        Check::Ok(parts.join(", "))
    }
}

/// Run all status checks and format the output.
pub fn run_status() -> String {
    let checks = [
        ("shell", detect_shell()),
        ("init", check_shell_config()),
        ("config", check_config_dir()),
    ];

    let mut lines = Vec::new();
    for (label, check) in &checks {
        lines.push(format!(
            "  {label:<8} [{icon}]  {msg}",
            icon = check.icon(),
            msg = check.message()
        ));
    }

    lines.join("\n")
}
