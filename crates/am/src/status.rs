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

/// Detect the current shell using multiple strategies:
/// 1. $SHELL env var (Unix standard)
/// 2. PowerShell-specific env vars ($PSModulePath, $PSVersionTable markers)
/// 3. $COMSPEC for cmd.exe on Windows
pub fn detect_shell() -> Check {
    // 1. $SHELL — standard on Unix
    if let Ok(shell) = std::env::var("SHELL") {
        let name = Path::new(&shell)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| shell.clone());
        return Check::Ok(format!("{name} ({shell})"));
    }

    // 2. PowerShell — check for PSModulePath (set in all PowerShell versions)
    if std::env::var("PSModulePath").is_ok() {
        return Check::Ok("powershell".to_string());
    }

    // 3. cmd.exe — check COMSPEC on Windows
    if let Ok(comspec) = std::env::var("COMSPEC") {
        let name = Path::new(&comspec)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| comspec.clone());
        return Check::Warn(format!("{name} — not supported, use PowerShell instead"));
    }

    Check::Warn("cannot detect shell — $SHELL not set".to_string())
}

/// Returns the detected shell name for use by other status checks.
pub fn detected_shell_name() -> Option<String> {
    if let Ok(shell) = std::env::var("SHELL") {
        return Path::new(&shell)
            .file_name()
            .map(|s| s.to_string_lossy().to_string());
    }
    if std::env::var("PSModulePath").is_ok() {
        return Some("powershell".to_string());
    }
    None
}

/// Check if the shell config file contains `am init`.
pub fn check_shell_config() -> Check {
    let shell_name = match detected_shell_name() {
        Some(name) => name,
        None => return Check::Warn("cannot detect shell — run `am setup <shell>`".to_string()),
    };

    let (config_path, init_line) = match shell_name.as_str() {
        "fish" => (
            home_dir().map(|h| h.join(".config/fish/config.fish")),
            "am init fish | source",
        ),
        "zsh" => (
            home_dir().map(|h| h.join(".zshrc")),
            "eval \"$(am init zsh)\"",
        ),
        "bash" => {
            let init_line = "eval \"$(am init bash)\"";
            let home_dir = home_dir();
            let candidates: Vec<std::path::PathBuf> = [".bash_profile", ".bashrc"]
                .iter()
                .filter_map(|f| home_dir.as_ref().map(|h| h.join(f)))
                .collect();

            // Check both files — user may have am init in either
            for path in &candidates {
                if path.exists() {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        if content.contains("am init") {
                            return Check::Ok(format!("{} contains `am init`", path.display()));
                        }
                    }
                }
            }

            // Neither file contains am init — pick the preferred one for the warning
            let preferred = candidates
                .iter()
                .find(|p| p.exists())
                .or_else(|| {
                    if cfg!(target_os = "macos") {
                        candidates.first()
                    } else {
                        candidates.last()
                    }
                })
                .cloned();

            let Some(path) = preferred else {
                return Check::Warn("cannot determine home directory".to_string());
            };

            if !path.exists() {
                return Check::Warn(format!(
                    "{} does not exist\n          add: {init_line}",
                    path.display()
                ));
            }

            return Check::Warn(format!(
                "{} missing `am init`\n          add: {init_line}",
                path.display()
            ));
        }
        "powershell" => {
            let path = crate::setup::detect_powershell_profile();
            (
                path,
                "(am init powershell) -join \"`n\" | Invoke-Expression",
            )
        }
        _ => {
            return Check::Warn(format!(
                "unknown shell: {shell_name} — run `am setup <shell>`"
            ))
        }
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
