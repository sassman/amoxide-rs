use crate::shell::Shells;
use crate::{Profile, TomlAlias};

/// Generate the complete shell init script for the given shell and active profile.
pub fn generate_init(shell: &Shells, profile: &Profile) -> String {
    let shell_impl = shell.clone().as_shell();
    let mut lines: Vec<String> = Vec::new();

    // Emit alias definitions from the active profile
    for (alias_name, alias_value) in profile.aliases.iter() {
        let name = alias_name.as_ref();
        let cmd = match alias_value {
            TomlAlias::Command(cmd) => cmd.as_str(),
            TomlAlias::Detailed(detail) => detail.command.as_str(),
        };
        lines.push(shell_impl.alias(name, cmd));
    }

    // Emit cd hook for project aliases
    lines.push(String::new());
    lines.push(cd_hook_setup(shell));

    lines.join("\n")
}

fn cd_hook_setup(shell: &Shells) -> String {
    let shell_name = shell.to_string();
    match shell {
        Shells::Fish => [
            "# am cd hook",
            "function __am_hook --on-variable PWD",
            &format!("    am hook {shell_name} | source"),
            "end",
            "__am_hook",
        ]
        .join("\n"),
        Shells::Zsh => [
            "# am cd hook",
            &format!("__am_hook() {{ eval \"$(am hook {shell_name})\"; }}"),
            "chpwd_functions+=(__am_hook)",
            "__am_hook",
        ]
        .join("\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Profile;

    fn test_profile() -> Profile {
        let mut p = Profile::new("test".to_string(), None);
        p.add_alias("ll".to_string(), "ls -lha".to_string())
            .unwrap();
        p.add_alias("gs".to_string(), "git status".to_string())
            .unwrap();
        p
    }

    #[test]
    fn test_fish_init_contains_aliases() {
        let profile = test_profile();
        let output = generate_init(&Shells::Fish, &profile);
        assert!(output.contains("alias gs \"git status\""));
        assert!(output.contains("alias ll \"ls -lha\""));
    }

    #[test]
    fn test_fish_init_contains_cd_hook() {
        let profile = test_profile();
        let output = generate_init(&Shells::Fish, &profile);
        assert!(output.contains("--on-variable PWD"));
        assert!(output.contains("am hook fish"));
    }

    #[test]
    fn test_zsh_init_contains_aliases() {
        let profile = test_profile();
        let output = generate_init(&Shells::Zsh, &profile);
        assert!(output.contains("alias gs=\"git status\""));
        assert!(output.contains("alias ll=\"ls -lha\""));
    }

    #[test]
    fn test_zsh_init_contains_cd_hook() {
        let profile = test_profile();
        let output = generate_init(&Shells::Zsh, &profile);
        assert!(output.contains("chpwd_functions"));
        assert!(output.contains("am hook zsh"));
    }

    #[test]
    fn test_init_empty_profile() {
        let profile = Profile::new("empty".to_string(), None);
        let output = generate_init(&Shells::Fish, &profile);
        // should still have the cd hook even with no aliases
        assert!(output.contains("__am_hook"));
        // should not have any alias lines
        assert!(!output.contains("alias "));
    }
}
