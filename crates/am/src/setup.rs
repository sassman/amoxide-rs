use std::io::{BufRead, Write};
use std::path::PathBuf;

use crate::shell::Shells;

/// Ask PowerShell for its $PROFILE path by shelling out.
/// Works for both PS 5.1 (WindowsPowerShell) and PS 7+ (PowerShell).
pub fn detect_powershell_profile() -> Option<PathBuf> {
    // Try pwsh (PS 7+) first, then powershell.exe (PS 5.1)
    for cmd in &["pwsh", "powershell"] {
        if let Ok(output) = std::process::Command::new(cmd)
            .args(["-NoProfile", "-Command", "echo $PROFILE"])
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }
    }
    None
}

/// Returns the profile file path and the init line for the given shell.
fn shell_config(shell: &Shells) -> (PathBuf, &'static str) {
    match shell {
        Shells::Fish => {
            let mut path = dirs_lite::config_dir().unwrap_or_else(|| PathBuf::from(".config"));
            path.push("fish/config.fish");
            (path, "am init fish | source")
        }
        Shells::Zsh => {
            let home = crate::dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            (home.join(".zshrc"), r#"eval "$(am init zsh)""#)
        }
        Shells::Powershell => {
            let path = detect_powershell_profile().unwrap_or_else(|| {
                let home = crate::dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                home.join("Documents/WindowsPowerShell/Microsoft.PowerShell_profile.ps1")
            });
            (
                path,
                "(am init powershell) -join \"`n\" | Invoke-Expression",
            )
        }
    }
}

/// Returns how to reload the shell after setup.
fn reload_hint(shell: &Shells, profile_path: &std::path::Path) -> String {
    let path = profile_path.display();
    match shell {
        Shells::Fish => format!("Run: source {path}"),
        Shells::Zsh => format!("Run: source {path}"),
        Shells::Powershell => format!(
            "Reload your profile:\n\n  . \"{path}\"\n\n\
            If you get a \"running scripts is disabled\" error, run this first:\n\n  \
            Set-ExecutionPolicy -Scope CurrentUser RemoteSigned"
        ),
    }
}

/// Run the interactive setup for the given shell.
pub fn run_setup(shell: &Shells) -> anyhow::Result<()> {
    let (profile_path, init_line) = shell_config(shell);

    eprintln!("Detected shell: {shell}");
    eprintln!("Profile path:   {}\n", profile_path.display());

    // Check if file exists
    let file_exists = profile_path.exists();
    let already_configured = if file_exists {
        let content = std::fs::read_to_string(&profile_path)?;
        content.contains("am init")
    } else {
        false
    };

    if already_configured {
        eprintln!("\u{2713} amoxide is already configured in your shell profile.");
        return Ok(());
    }

    if !file_exists {
        eprintln!("Profile file does not exist.");
        eprint!("Create it? [Y/n] ");
        std::io::stderr().flush()?;
        let mut input = String::new();
        std::io::stdin().lock().read_line(&mut input)?;
        if matches!(input.trim().to_lowercase().as_str(), "n" | "no") {
            eprintln!("Cancelled.");
            return Ok(());
        }
        // Create parent directories
        if let Some(parent) = profile_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    eprintln!(
        "The following line will be added to {}:\n",
        profile_path.display()
    );
    eprintln!("  {init_line}\n");
    eprint!("Add it now? [Y/n] ");
    std::io::stderr().flush()?;
    let mut input = String::new();
    std::io::stdin().lock().read_line(&mut input)?;
    if matches!(input.trim().to_lowercase().as_str(), "n" | "no") {
        eprintln!("Cancelled.");
        return Ok(());
    }

    // Append init line
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&profile_path)?;

    // Add a newline before our line if the file isn't empty and doesn't end with one
    if file_exists {
        let content = std::fs::read_to_string(&profile_path)?;
        if !content.is_empty() && !content.ends_with('\n') {
            writeln!(file)?;
        }
    }
    writeln!(file, "{init_line}")?;

    eprintln!("\n\u{2713} Added to {}", profile_path.display());
    eprintln!("  {}", reload_hint(shell, &profile_path));

    Ok(())
}
