use std::io::{BufRead, Write};
use std::path::PathBuf;

use crate::prompt::{ask_user, Answer};
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
        Shells::Bash => {
            let home = crate::dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            (home.join(".bashrc"), r#"eval "$(am init bash)""#)
        }
        Shells::Brush => {
            let home = crate::dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            (home.join(".brushrc"), r#"eval "$(am init brush)""#)
        }
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
        Shells::Bash | Shells::Brush => format!("Run: source {path}"),
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
    run_setup_inner(
        shell,
        &profile_path,
        init_line,
        &mut std::io::stdin().lock(),
    )
}

/// Core setup logic, testable with custom path and reader.
fn run_setup_inner(
    shell: &Shells,
    profile_path: &std::path::Path,
    init_line: &str,
    reader: &mut dyn BufRead,
) -> anyhow::Result<()> {
    eprintln!("Detected shell: {shell}");
    eprintln!("Profile path:   {}\n", profile_path.display());

    // Check if file exists
    let file_exists = profile_path.exists();
    let already_configured = if file_exists {
        let content = std::fs::read_to_string(profile_path)?;
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
        if ask_user("Create it?", Answer::Yes, false, reader)? != Answer::Yes {
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
    if ask_user("Add it now?", Answer::Yes, false, reader)? != Answer::Yes {
        eprintln!("Cancelled.");
        return Ok(());
    }

    // Append init line
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(profile_path)?;

    // Add a newline before our line if the file isn't empty and doesn't end with one
    if file_exists {
        let content = std::fs::read_to_string(profile_path)?;
        if !content.is_empty() && !content.ends_with('\n') {
            writeln!(file)?;
        }
    }
    writeln!(file, "{init_line}")?;

    eprintln!("\n\u{2713} Added to {}", profile_path.display());
    eprintln!("  {}", reload_hint(shell, profile_path));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    const INIT_LINE: &str = r#"eval "$(am init zsh)""#;

    /// Run setup on an existing profile file with the given stdin input.
    /// Returns the file content after setup completes.
    fn run_setup_on_existing(input: &[u8]) -> String {
        let dir = tempfile::tempdir().unwrap();
        let profile = dir.path().join(".zshrc");
        std::fs::write(&profile, "# existing config\n").unwrap();

        let mut reader = Cursor::new(input.to_vec());
        run_setup_inner(&Shells::Zsh, &profile, INIT_LINE, &mut reader).unwrap();

        std::fs::read_to_string(&profile).unwrap()
    }

    #[test]
    fn setup_respects_no_on_existing_file() {
        let content = run_setup_on_existing(b"n\n");
        assert!(!content.contains("am init"), "got: {content}");
    }

    #[test]
    fn setup_respects_uppercase_no() {
        let content = run_setup_on_existing(b"N\n");
        assert!(!content.contains("am init"), "got: {content}");
    }

    #[test]
    fn setup_respects_no_word() {
        let content = run_setup_on_existing(b"no\n");
        assert!(!content.contains("am init"), "got: {content}");
    }

    #[test]
    fn setup_adds_line_on_yes() {
        let content = run_setup_on_existing(b"y\n");
        assert!(content.contains("am init"), "got: {content}");
    }

    #[test]
    fn setup_on_eof_does_not_add_line() {
        let content = run_setup_on_existing(b"");
        assert!(!content.contains("am init"), "got: {content}");
    }

    #[test]
    fn setup_bash_adds_line_on_yes() {
        let dir = tempfile::tempdir().unwrap();
        let profile = dir.path().join(".bashrc");
        std::fs::write(&profile, "# existing config\n").unwrap();

        let init_line = r#"eval "$(am init bash)""#;
        let mut reader = Cursor::new(b"y\n".to_vec());
        run_setup_inner(&Shells::Bash, &profile, init_line, &mut reader).unwrap();

        let content = std::fs::read_to_string(&profile).unwrap();
        assert!(content.contains("am init bash"), "got: {content}");
    }

    /// File doesn't exist, user says "y" to create, but then EOF for add prompt.
    #[test]
    fn setup_on_eof_second_prompt_does_not_add_line() {
        let dir = tempfile::tempdir().unwrap();
        let profile = dir.path().join("subdir/.zshrc");

        let mut reader = Cursor::new(b"y\n");
        run_setup_inner(&Shells::Zsh, &profile, INIT_LINE, &mut reader).unwrap();

        if profile.exists() {
            let content = std::fs::read_to_string(&profile).unwrap();
            assert!(!content.contains("am init"), "got: {content}");
        }
    }
}
