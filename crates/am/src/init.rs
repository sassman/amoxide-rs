use crate::shell::Shells;
use crate::AliasSet;

const WRAPPER_BASH: &str = include_str!("shell_wrappers/wrapper.bash");
const WRAPPER_FISH: &str = include_str!("shell_wrappers/wrapper.fish");
const WRAPPER_ZSH: &str = include_str!("shell_wrappers/wrapper.zsh");
const WRAPPER_PS1: &str = include_str!("shell_wrappers/wrapper.ps1");
const HOOK_BASH: &str = include_str!("shell_wrappers/hook.bash");
const HOOK_FISH: &str = include_str!("shell_wrappers/hook.fish");
const HOOK_ZSH: &str = include_str!("shell_wrappers/hook.zsh");
const HOOK_PS1: &str = include_str!("shell_wrappers/hook.ps1");
const COMPLETIONS_FISH: &str = include_str!(concat!(env!("OUT_DIR"), "/am.fish"));
const COMPLETIONS_ZSH: &str = include_str!(concat!(env!("OUT_DIR"), "/_am"));
// PowerShell completions use `using namespace` which can't be inside Invoke-Expression.
// We strip `using namespace` lines and expand type names at runtime for Invoke-Expression compat.
const COMPLETIONS_PS1: &str = include_str!(concat!(env!("OUT_DIR"), "/_am.ps1"));

/// Generate the complete shell init script.
/// `global_aliases` — always loaded, independent of profile.
/// `profile_aliases` — merged alias set from all active profiles.
pub fn generate_init(
    shell: &Shells,
    global_aliases: &AliasSet,
    profile_aliases: &AliasSet,
) -> String {
    let shell_impl = shell.clone().as_shell();
    let mut lines: Vec<String> = Vec::new();
    let mut all_names: Vec<String> = Vec::new();

    // Emit global aliases
    for (alias_name, alias_value) in global_aliases.iter() {
        let name = alias_name.as_ref();
        lines.push(shell_impl.alias(&alias_value.as_entry(name)));
        all_names.push(name.to_string());
    }

    // Emit profile aliases (merged from all active profiles)
    for (alias_name, alias_value) in profile_aliases.iter() {
        let name = alias_name.as_ref();
        lines.push(shell_impl.alias(&alias_value.as_entry(name)));
        all_names.push(name.to_string());
    }

    // Track all loaded aliases (global + profile) for reload cleanup
    if !all_names.is_empty() {
        all_names.sort();
        all_names.dedup();
        lines.push(shell_impl.set_env("_AM_ALIASES", &all_names.join(",")));
    }
    // Clean up legacy tracking var from older versions
    lines.push(shell_impl.unset_env("_AM_PROFILE_ALIASES"));

    // Wrapper function
    lines.push(String::new());
    lines.push(am_wrapper(shell));

    // cd hook for project aliases
    lines.push(String::new());
    lines.push(cd_hook_setup(shell));

    // Shell completions
    lines.push(String::new());
    lines.push(completions(shell));

    lines.join("\n")
}

/// Generate shell code to reload all aliases (global + profile) after a mutation.
/// Unloads old aliases, loads new ones, updates the tracking env var.
pub fn generate_reload(
    shell: &Shells,
    global_aliases: &AliasSet,
    profile_aliases: &AliasSet,
    previous_aliases: Option<&str>,
) -> String {
    let shell_impl = shell.clone().as_shell();
    let mut lines: Vec<String> = Vec::new();

    // Unload all previously tracked aliases
    let prev: Vec<&str> = previous_aliases
        .filter(|s| !s.is_empty())
        .map(|s| s.split(',').collect())
        .unwrap_or_default();

    for alias_name in &prev {
        lines.push(shell_impl.unalias(alias_name));
    }

    // Load global + profile aliases
    let mut all_names: Vec<String> = Vec::new();

    for (alias_name, alias_value) in global_aliases.iter() {
        let name = alias_name.as_ref();
        lines.push(shell_impl.alias(&alias_value.as_entry(name)));
        all_names.push(name.to_string());
    }

    for (alias_name, alias_value) in profile_aliases.iter() {
        let name = alias_name.as_ref();
        lines.push(shell_impl.alias(&alias_value.as_entry(name)));
        all_names.push(name.to_string());
    }

    // Update tracking
    if all_names.is_empty() {
        if !prev.is_empty() {
            lines.push(shell_impl.unset_env("_AM_ALIASES"));
        }
    } else {
        all_names.sort();
        all_names.dedup();
        lines.push(shell_impl.set_env("_AM_ALIASES", &all_names.join(",")));
    }

    lines.join("\n")
}

fn shell_script(template: &str, shell: &Shells) -> String {
    template.replace("__SHELL__", &shell.to_string())
}

fn am_wrapper(shell: &Shells) -> String {
    match shell {
        Shells::Bash => shell_script(WRAPPER_BASH, shell),
        Shells::Fish => shell_script(WRAPPER_FISH, shell),
        Shells::Powershell => shell_script(WRAPPER_PS1, shell),
        Shells::Zsh => shell_script(WRAPPER_ZSH, shell),
    }
}

fn cd_hook_setup(shell: &Shells) -> String {
    match shell {
        Shells::Bash => shell_script(HOOK_BASH, shell),
        Shells::Fish => shell_script(HOOK_FISH, shell),
        Shells::Powershell => shell_script(HOOK_PS1, shell),
        Shells::Zsh => shell_script(HOOK_ZSH, shell),
    }
}

fn completions(shell: &Shells) -> String {
    match shell {
        Shells::Bash => include_str!(concat!(env!("OUT_DIR"), "/am.bash")).to_string(),
        Shells::Fish => COMPLETIONS_FISH.to_string(),
        Shells::Powershell => powershell_completions(),
        Shells::Zsh => COMPLETIONS_ZSH.to_string(),
    }
}

/// PowerShell completions use `using namespace` which can't be inside Invoke-Expression.
/// We strip those lines and replace short type names with fully qualified ones.
fn powershell_completions() -> String {
    COMPLETIONS_PS1
        .lines()
        .filter(|line| !line.starts_with("using namespace"))
        .collect::<Vec<_>>()
        .join("\n")
        .replace(
            "[CompletionResult]",
            "[System.Management.Automation.CompletionResult]",
        )
        .replace(
            "[CompletionResultType]",
            "[System.Management.Automation.CompletionResultType]",
        )
        .replace(
            "[StringConstantExpressionAst]",
            "[System.Management.Automation.Language.StringConstantExpressionAst]",
        )
        .replace(
            "[StringConstantType]",
            "[System.Management.Automation.Language.StringConstantType]",
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AliasName, TomlAlias};

    fn test_aliases() -> AliasSet {
        let mut aliases = AliasSet::default();
        aliases.insert(
            AliasName::from("ll"),
            TomlAlias::Command("ls -lha".to_string()),
        );
        aliases.insert(
            AliasName::from("gs"),
            TomlAlias::Command("git status".to_string()),
        );
        aliases
    }

    #[test]
    fn test_fish_init_contains_aliases() {
        let aliases = test_aliases();
        let output = generate_init(&Shells::Fish, &AliasSet::default(), &aliases);
        assert!(output.contains("alias gs \"git status\""));
        assert!(output.contains("alias ll \"ls -lha\""));
    }

    #[test]
    fn test_fish_init_tracks_all_aliases() {
        let aliases = test_aliases();
        let output = generate_init(&Shells::Fish, &AliasSet::default(), &aliases);
        assert!(output.contains("_AM_ALIASES"));
    }

    #[test]
    fn test_fish_init_contains_wrapper() {
        let aliases = test_aliases();
        let output = generate_init(&Shells::Fish, &AliasSet::default(), &aliases);
        assert!(output.contains("function am --wraps=am"));
        assert!(output.contains("am reload fish"));
        assert!(output.contains("--local"));
        assert!(output.contains("am hook fish"));
    }

    #[test]
    fn test_fish_init_contains_cd_hook() {
        let aliases = test_aliases();
        let output = generate_init(&Shells::Fish, &AliasSet::default(), &aliases);
        assert!(output.contains("--on-variable PWD"));
        assert!(output.contains("am hook fish"));
    }

    #[test]
    fn test_zsh_init_contains_aliases() {
        let aliases = test_aliases();
        let output = generate_init(&Shells::Zsh, &AliasSet::default(), &aliases);
        assert!(output.contains("gs() { git status \"$@\"; }"));
        assert!(output.contains("ll() { ls -lha \"$@\"; }"));
    }

    #[test]
    fn test_zsh_init_contains_wrapper() {
        let aliases = test_aliases();
        let output = generate_init(&Shells::Zsh, &AliasSet::default(), &aliases);
        assert!(output.contains("am()"));
        assert!(output.contains("am reload zsh"));
        assert!(output.contains("--local"));
        assert!(output.contains("am hook zsh"));
    }

    #[test]
    fn test_zsh_init_contains_cd_hook() {
        let aliases = test_aliases();
        let output = generate_init(&Shells::Zsh, &AliasSet::default(), &aliases);
        assert!(output.contains("chpwd_functions"));
        assert!(output.contains("am hook zsh"));
    }

    #[test]
    fn test_init_empty_no_tracking_var() {
        let output = generate_init(&Shells::Fish, &AliasSet::default(), &AliasSet::default());
        assert!(output.contains("__am_hook"));
        assert!(!output.contains("_AM_ALIASES"));
    }

    #[test]
    fn test_reload_unloads_old_and_loads_new() {
        let aliases = test_aliases();
        let output = generate_reload(
            &Shells::Fish,
            &AliasSet::default(),
            &aliases,
            Some("old1,old2"),
        );
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("alias gs \"git status\""));
        assert!(output.contains("alias ll \"ls -lha\""));
        assert!(output.contains("_AM_ALIASES"));
    }

    #[test]
    fn test_reload_zsh_unloads_with_unset_f() {
        let aliases = test_aliases();
        let output = generate_reload(&Shells::Zsh, &AliasSet::default(), &aliases, Some("old1"));
        assert!(output.contains("unset -f old1"));
        assert!(output.contains("gs() { git status \"$@\"; }"));
    }

    #[test]
    fn test_reload_no_previous() {
        let aliases = test_aliases();
        let output = generate_reload(&Shells::Fish, &AliasSet::default(), &aliases, None);
        assert!(!output.contains("functions -e"));
        assert!(output.contains("alias gs"));
    }

    #[test]
    fn test_reload_to_empty_clears_tracking() {
        let output = generate_reload(
            &Shells::Fish,
            &AliasSet::default(),
            &AliasSet::default(),
            Some("old1"),
        );
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("set -e _AM_ALIASES"));
    }

    #[test]
    fn test_init_includes_global_aliases() {
        let mut globals = AliasSet::default();
        globals.insert(
            "ll".into(),
            crate::TomlAlias::Command("ls -lha".to_string()),
        );
        let output = generate_init(&Shells::Fish, &globals, &AliasSet::default());
        assert!(output.contains("alias ll \"ls -lha\""));
    }

    #[test]
    fn test_init_global_before_profile() {
        let mut globals = AliasSet::default();
        globals.insert(
            "gl".into(),
            crate::TomlAlias::Command("global cmd".to_string()),
        );
        let aliases = test_aliases();
        let output = generate_init(&Shells::Fish, &globals, &aliases);
        let gl_pos = output.find("gl").unwrap();
        let gs_pos = output.find("gs").unwrap();
        assert!(
            gl_pos < gs_pos,
            "global aliases should appear before profile aliases"
        );
    }

    #[test]
    fn test_reload_includes_globals() {
        let mut globals = AliasSet::default();
        globals.insert(
            "ll".into(),
            crate::TomlAlias::Command("ls -lha".to_string()),
        );
        let output = generate_reload(&Shells::Fish, &globals, &AliasSet::default(), Some("old"));
        assert!(output.contains("functions -e old"));
        assert!(output.contains("alias ll \"ls -lha\""));
    }

    #[test]
    fn test_bash_init_contains_aliases() {
        let aliases = test_aliases();
        let output = generate_init(&Shells::Bash, &AliasSet::default(), &aliases);
        assert!(output.contains("gs() { git status \"$@\"; }"));
        assert!(output.contains("ll() { ls -lha \"$@\"; }"));
    }

    #[test]
    fn test_bash_init_contains_wrapper() {
        let aliases = test_aliases();
        let output = generate_init(&Shells::Bash, &AliasSet::default(), &aliases);
        assert!(output.contains("am()"));
        assert!(output.contains("am reload bash"));
        assert!(output.contains("--local"));
        assert!(output.contains("am hook bash"));
    }

    #[test]
    fn test_bash_init_contains_cd_hook() {
        let aliases = test_aliases();
        let output = generate_init(&Shells::Bash, &AliasSet::default(), &aliases);
        assert!(output.contains("PROMPT_COMMAND"));
        assert!(output.contains("__am_hook"));
        assert!(output.contains("__am_prev_dir"));
        assert!(output.contains("am hook bash"));
    }

    #[test]
    fn test_reload_bash_unloads_with_unset_f() {
        let aliases = test_aliases();
        let output = generate_reload(&Shells::Bash, &AliasSet::default(), &aliases, Some("old1"));
        assert!(output.contains("unset -f old1"));
        assert!(output.contains("gs() { git status \"$@\"; }"));
    }
}
