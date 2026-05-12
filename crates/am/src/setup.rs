use std::io::{BufRead, Write};
use std::path::PathBuf;

use crate::prompt::{ask_user, Answer};
use crate::shell::Shell;

/// Supported AI coding agents for `am context --setup <agent>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assistant {
    Claude,
}

impl Assistant {
    /// Parse an agent name from a CLI flag value.
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s {
            "claude" => Ok(Self::Claude),
            other => Err(anyhow::anyhow!(
                "unsupported agent '{other}'. supported: claude"
            )),
        }
    }
}

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
fn shell_config(shell: &Shell) -> (PathBuf, &'static str) {
    match shell {
        Shell::Bash => {
            let home = crate::dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            (home.join(".bashrc"), r#"eval "$(am init bash)""#)
        }
        Shell::Brush => {
            let home = crate::dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            (home.join(".brushrc"), r#"eval "$(am init brush)""#)
        }
        Shell::Fish => {
            let mut path = dirs_lite::config_dir().unwrap_or_else(|| PathBuf::from(".config"));
            path.push("fish/config.fish");
            (path, "am init fish | source")
        }
        Shell::Zsh => {
            let home = crate::dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            (home.join(".zshrc"), r#"eval "$(am init zsh)""#)
        }
        Shell::Powershell => {
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
fn reload_hint(shell: &Shell, profile_path: &std::path::Path) -> String {
    let path = profile_path.display();
    match shell {
        Shell::Bash | Shell::Brush => format!("Run: source {path}"),
        Shell::Fish => format!("Run: source {path}"),
        Shell::Zsh => format!("Run: source {path}"),
        Shell::Powershell => format!(
            "Reload your profile:\n\n  . \"{path}\"\n\n\
            If you get a \"running scripts is disabled\" error, run this first:\n\n  \
            Set-ExecutionPolicy -Scope CurrentUser RemoteSigned"
        ),
    }
}

/// Run the interactive setup for the given shell.
pub fn run_setup(shell: &Shell) -> anyhow::Result<()> {
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
    shell: &Shell,
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

/// In-place merge of our v2 hook entry into `settings.hooks.SessionStart`.
///
/// Idempotent: if `am context` is already wired (per `claude_settings_already_wired`),
/// no change is made. Preserves all other top-level keys, hook events, and
/// existing `SessionStart` entries.
pub fn merge_claude_hook(settings: &mut serde_json::Value) {
    if claude_settings_already_wired(settings) {
        return;
    }

    if !settings.is_object() {
        *settings = serde_json::json!({});
    }

    let hooks = settings
        .as_object_mut()
        .unwrap()
        .entry("hooks".to_string())
        .or_insert_with(|| serde_json::json!({}));

    if !hooks.is_object() {
        *hooks = serde_json::json!({});
    }

    let session_start = hooks
        .as_object_mut()
        .unwrap()
        .entry("SessionStart".to_string())
        .or_insert_with(|| serde_json::json!([]));

    if !session_start.is_array() {
        *session_start = serde_json::json!([]);
    }

    session_start
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "matcher": "startup|clear|compact",
            "hooks": [
                { "type": "command", "command": "am context", "async": false }
            ]
        }));
}

/// Write JSON atomically: temp file in same dir + rename.
/// Pretty-printed for human review. Creates parent dir if needed.
pub fn write_settings_atomic(
    path: &std::path::Path,
    value: &serde_json::Value,
) -> anyhow::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("settings path has no parent: {}", path.display()))?;
    std::fs::create_dir_all(parent)?;

    let serialized = serde_json::to_string_pretty(value)?;
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("settings path has no file name: {}", path.display()))?
        .to_string_lossy();
    let tmp = parent.join(format!(".{file_name}.tmp"));
    std::fs::write(&tmp, &serialized)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Returns true if `settings` already contains a `SessionStart` hook entry
/// whose command contains `am context`. Detects both the v2 schema
/// (`SessionStart[].hooks[].command`) and the legacy flat shape
/// (`SessionStart[].command`).
pub fn claude_settings_already_wired(settings: &serde_json::Value) -> bool {
    let Some(session_start) = settings
        .get("hooks")
        .and_then(|h| h.get("SessionStart"))
        .and_then(|s| s.as_array())
    else {
        return false;
    };

    for entry in session_start {
        // Legacy flat shape: { "command": "am context" }
        if let Some(cmd) = entry.get("command").and_then(|c| c.as_str()) {
            if cmd.contains("am context") {
                return true;
            }
        }
        // V2 nested shape: { "matcher": "...", "hooks": [{ "command": "..." }] }
        if let Some(nested) = entry.get("hooks").and_then(|h| h.as_array()) {
            for h in nested {
                if let Some(cmd) = h.get("command").and_then(|c| c.as_str()) {
                    if cmd.contains("am context") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// What `run_assistant_setup` did. Returned for the handler to render.
#[derive(Debug, Clone, PartialEq)]
pub enum SetupOutcome {
    Created(std::path::PathBuf),
    Updated(std::path::PathBuf),
    AlreadyConfigured(std::path::PathBuf),
}

impl SetupOutcome {
    pub fn render(&self) -> String {
        match self {
            Self::Created(p) => {
                format!(
                    "am: created {} with am context SessionStart hook",
                    p.display()
                )
            }
            Self::Updated(p) => {
                format!("am: added am context SessionStart hook to {}", p.display())
            }
            Self::AlreadyConfigured(p) => {
                format!("am: am context already wired into {}", p.display())
            }
        }
    }
}

/// Resolve the Claude Code settings file path: `~/.claude/settings.json`.
pub fn claude_settings_path() -> anyhow::Result<std::path::PathBuf> {
    let home = crate::dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("could not determine HOME directory"))?;
    Ok(home.join(".claude/settings.json"))
}

/// Drive the full setup pipeline for the given assistant.
pub fn run_assistant_setup(assistant: Assistant) -> anyhow::Result<SetupOutcome> {
    match assistant {
        Assistant::Claude => run_claude_setup(&claude_settings_path()?),
    }
}

/// Underlying impl for Claude — takes the target path for test injectability.
///
/// Read+parse failures abort without overwriting. Idempotent: returns
/// `AlreadyConfigured` when our hook is already present.
pub fn run_claude_setup(path: &std::path::Path) -> anyhow::Result<SetupOutcome> {
    let (mut settings, existed) = if path.exists() {
        let contents = std::fs::read_to_string(path)?;
        let parsed: serde_json::Value = serde_json::from_str(&contents)
            .map_err(|e| anyhow::anyhow!("could not parse {}: {}", path.display(), e))?;
        (parsed, true)
    } else {
        (serde_json::json!({}), false)
    };

    if claude_settings_already_wired(&settings) {
        return Ok(SetupOutcome::AlreadyConfigured(path.to_path_buf()));
    }

    merge_claude_hook(&mut settings);
    write_settings_atomic(path, &settings)?;

    Ok(if existed {
        SetupOutcome::Updated(path.to_path_buf())
    } else {
        SetupOutcome::Created(path.to_path_buf())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn already_wired_returns_true_for_v2_schema_with_am_context() {
        let json = serde_json::json!({
            "hooks": {
                "SessionStart": [
                    {
                        "matcher": "startup|clear|compact",
                        "hooks": [
                            { "type": "command", "command": "am context", "async": false }
                        ]
                    }
                ]
            }
        });
        assert!(claude_settings_already_wired(&json));
    }

    #[test]
    fn already_wired_returns_false_when_no_hooks_section() {
        let json = serde_json::json!({});
        assert!(!claude_settings_already_wired(&json));
    }

    #[test]
    fn already_wired_returns_false_when_session_start_has_other_command() {
        let json = serde_json::json!({
            "hooks": {
                "SessionStart": [
                    {
                        "matcher": "startup",
                        "hooks": [
                            { "type": "command", "command": "echo hi", "async": false }
                        ]
                    }
                ]
            }
        });
        assert!(!claude_settings_already_wired(&json));
    }

    #[test]
    fn already_wired_handles_legacy_flat_shape() {
        // Pre-v2 schema: `{ "command": "am context" }` directly in SessionStart.
        let json = serde_json::json!({
            "hooks": {
                "SessionStart": [
                    { "command": "am context" }
                ]
            }
        });
        assert!(claude_settings_already_wired(&json));
    }

    #[test]
    fn assistant_parse_accepts_claude() {
        assert!(matches!(Assistant::parse("claude"), Ok(Assistant::Claude)));
    }

    #[test]
    fn assistant_parse_rejects_unknown() {
        let err = Assistant::parse("openai").unwrap_err();
        assert!(err.to_string().contains("unsupported agent"), "got: {err}");
        assert!(
            err.to_string().contains("openai"),
            "error should mention input: {err}"
        );
        assert!(
            err.to_string().contains("claude"),
            "error should list supported: {err}"
        );
    }

    const INIT_LINE: &str = r#"eval "$(am init zsh)""#;

    /// Run setup on an existing profile file with the given stdin input.
    /// Returns the file content after setup completes.
    fn run_setup_on_existing(input: &[u8]) -> String {
        let dir = tempfile::tempdir().unwrap();
        let profile = dir.path().join(".zshrc");
        std::fs::write(&profile, "# existing config\n").unwrap();

        let mut reader = Cursor::new(input.to_vec());
        run_setup_inner(&Shell::Zsh, &profile, INIT_LINE, &mut reader).unwrap();

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
        run_setup_inner(&Shell::Bash, &profile, init_line, &mut reader).unwrap();

        let content = std::fs::read_to_string(&profile).unwrap();
        assert!(content.contains("am init bash"), "got: {content}");
    }

    /// File doesn't exist, user says "y" to create, but then EOF for add prompt.
    #[test]
    fn setup_on_eof_second_prompt_does_not_add_line() {
        let dir = tempfile::tempdir().unwrap();
        let profile = dir.path().join("subdir/.zshrc");

        let mut reader = Cursor::new(b"y\n");
        run_setup_inner(&Shell::Zsh, &profile, INIT_LINE, &mut reader).unwrap();

        if profile.exists() {
            let content = std::fs::read_to_string(&profile).unwrap();
            assert!(!content.contains("am init"), "got: {content}");
        }
    }

    #[test]
    fn merge_adds_session_start_when_no_hooks_key() {
        let mut settings = serde_json::json!({});
        merge_claude_hook(&mut settings);
        assert!(claude_settings_already_wired(&settings));
    }

    #[test]
    fn merge_preserves_existing_top_level_keys() {
        let mut settings = serde_json::json!({
            "model": "claude-opus-4",
            "theme": "dark"
        });
        merge_claude_hook(&mut settings);
        assert_eq!(settings["model"], "claude-opus-4");
        assert_eq!(settings["theme"], "dark");
        assert!(claude_settings_already_wired(&settings));
    }

    #[test]
    fn merge_preserves_existing_hook_events() {
        let mut settings = serde_json::json!({
            "hooks": {
                "PreToolUse": [{ "matcher": "*", "hooks": [{ "type": "command", "command": "echo pre" }] }]
            }
        });
        merge_claude_hook(&mut settings);
        assert_eq!(
            settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "echo pre"
        );
        assert!(claude_settings_already_wired(&settings));
    }

    #[test]
    fn merge_appends_to_existing_session_start_without_dropping_other_entries() {
        let mut settings = serde_json::json!({
            "hooks": {
                "SessionStart": [
                    { "matcher": "startup", "hooks": [{ "type": "command", "command": "echo other" }] }
                ]
            }
        });
        merge_claude_hook(&mut settings);
        let session_start = settings["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(session_start.len(), 2, "should append, not replace");
        assert!(claude_settings_already_wired(&settings));
    }

    #[test]
    fn merge_is_idempotent_when_already_wired() {
        let mut settings = serde_json::json!({
            "hooks": {
                "SessionStart": [
                    {
                        "matcher": "startup|clear|compact",
                        "hooks": [{ "type": "command", "command": "am context", "async": false }]
                    }
                ]
            }
        });
        let before = settings.clone();
        merge_claude_hook(&mut settings);
        assert_eq!(before, settings, "idempotent: no change on second run");
    }

    #[test]
    fn write_settings_atomic_creates_parent_dir() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a/b/c/settings.json");
        let value = serde_json::json!({"x": 1});
        write_settings_atomic(&nested, &value).unwrap();
        let read = std::fs::read_to_string(&nested).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&read).unwrap();
        assert_eq!(parsed, value);
    }

    #[test]
    fn write_settings_atomic_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        std::fs::write(&path, r#"{"old": true}"#).unwrap();
        let value = serde_json::json!({"new": true});
        write_settings_atomic(&path, &value).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(parsed, value);
    }
}
