use crate::shell::Shells;
use crate::{AliasSet, Profile};

/// Generate the complete shell init script for the given shell and active profile.
pub fn generate_init(shell: &Shells, global_aliases: &AliasSet, profile: &Profile) -> String {
    let shell_impl = shell.clone().as_shell();
    let mut lines: Vec<String> = Vec::new();

    // Emit global aliases first
    for (alias_name, alias_value) in global_aliases.iter() {
        let name = alias_name.as_ref();
        lines.push(shell_impl.alias(&alias_value.as_entry(name)));
    }

    // Emit alias definitions from the active profile
    let mut alias_names: Vec<String> = Vec::new();
    for (alias_name, alias_value) in profile.aliases.iter() {
        let name = alias_name.as_ref();
        lines.push(shell_impl.alias(&alias_value.as_entry(name)));
        alias_names.push(name.to_string());
    }

    // Track which profile aliases are loaded (for reload cleanup)
    if !alias_names.is_empty() {
        lines.push(shell_impl.set_env("_AM_PROFILE_ALIASES", &alias_names.join(",")));
    }

    // Wrapper function: intercepts `am profile set` to reload aliases
    lines.push(String::new());
    lines.push(am_wrapper(shell));

    // cd hook for project aliases
    lines.push(String::new());
    lines.push(cd_hook_setup(shell));

    lines.join("\n")
}

/// Generate shell code to reload profile aliases after a profile switch.
/// Unloads old aliases, loads new ones, updates the tracking env var.
pub fn generate_reload(
    shell: &Shells,
    profile: &Profile,
    previous_aliases: Option<&str>,
) -> String {
    let shell_impl = shell.clone().as_shell();
    let mut lines: Vec<String> = Vec::new();

    // Unload previous profile aliases
    let prev: Vec<&str> = previous_aliases
        .filter(|s| !s.is_empty())
        .map(|s| s.split(',').collect())
        .unwrap_or_default();

    for alias_name in &prev {
        lines.push(shell_impl.unalias(alias_name));
    }

    // Load new profile aliases
    let mut alias_names: Vec<String> = Vec::new();
    for (alias_name, alias_value) in profile.aliases.iter() {
        let name = alias_name.as_ref();
        lines.push(shell_impl.alias(&alias_value.as_entry(name)));
        alias_names.push(name.to_string());
    }

    // Update tracking
    if alias_names.is_empty() {
        if !prev.is_empty() {
            lines.push(shell_impl.unset_env("_AM_PROFILE_ALIASES"));
        }
    } else {
        lines.push(shell_impl.set_env("_AM_PROFILE_ALIASES", &alias_names.join(",")));
    }

    lines.join("\n")
}

fn am_wrapper(shell: &Shells) -> String {
    let shell_name = shell.to_string();
    let reload_cmd = format!("command am reload {shell_name} | source");
    let hook_cmd = format!("command am hook {shell_name} | source");
    match shell {
        Shells::Fish => [
            "# am wrapper: reload after profile switch or local alias change",
            "function am --wraps=am",
            "    command am $argv",
            "    # reload after profile switch (handles short forms: p/profile, s/set)",
            "    if begin; test \"$argv[1]\" = profile; or test \"$argv[1]\" = p; end",
            "        if begin; test \"$argv[2]\" = set; or test \"$argv[2]\" = s; end",
            &format!("            {reload_cmd}"),
            "        end",
            "    # re-source hook after local alias change (only for add/a or remove/r)",
            "    else if begin; test \"$argv[1]\" = add; or test \"$argv[1]\" = a; or test \"$argv[1]\" = remove; or test \"$argv[1]\" = r; end",
            "        if contains -- -l $argv; or contains -- --local $argv",
            &format!("            {hook_cmd}"),
            "        end",
            "    end",
            "end",
        ]
        .join("\n"),
        Shells::Zsh => {
            format!(
                "am() {{\n  \
                command am \"$@\"\n  \
                case \"$1:$2\" in\n    \
                profile:set|p:set|profile:s|p:s) eval \"$({reload_cmd})\" ;;\n  \
                esac\n  \
                case \"$1\" in\n    \
                add|a|remove|r)\n      \
                case \"$*\" in\n        \
                *\\ -l\\ *|*\\ --local\\ *|*\\ -l|*\\ --local) eval \"$({hook_cmd})\" ;;\n      \
                esac ;;\n  \
                esac\n\
                }}"
            )
        }
    }
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
        p.add_alias("ll".to_string(), "ls -lha".to_string(), false)
            .unwrap();
        p.add_alias("gs".to_string(), "git status".to_string(), false)
            .unwrap();
        p
    }

    #[test]
    fn test_fish_init_contains_aliases() {
        let profile = test_profile();
        let output = generate_init(&Shells::Fish, &AliasSet::default(), &profile);
        assert!(output.contains("alias gs \"git status\""));
        assert!(output.contains("alias ll \"ls -lha\""));
    }

    #[test]
    fn test_fish_init_tracks_profile_aliases() {
        let profile = test_profile();
        let output = generate_init(&Shells::Fish, &AliasSet::default(), &profile);
        assert!(output.contains("_AM_PROFILE_ALIASES"));
    }

    #[test]
    fn test_fish_init_contains_wrapper() {
        let profile = test_profile();
        let output = generate_init(&Shells::Fish, &AliasSet::default(), &profile);
        assert!(output.contains("function am --wraps=am"));
        assert!(output.contains("am reload fish"));
        // wrapper also intercepts local alias changes
        assert!(output.contains("--local"));
        assert!(output.contains("am hook fish"));
    }

    #[test]
    fn test_fish_init_contains_cd_hook() {
        let profile = test_profile();
        let output = generate_init(&Shells::Fish, &AliasSet::default(), &profile);
        assert!(output.contains("--on-variable PWD"));
        assert!(output.contains("am hook fish"));
    }

    #[test]
    fn test_zsh_init_contains_aliases() {
        let profile = test_profile();
        let output = generate_init(&Shells::Zsh, &AliasSet::default(), &profile);
        assert!(output.contains("gs() { git status \"$@\"; }"));
        assert!(output.contains("ll() { ls -lha \"$@\"; }"));
    }

    #[test]
    fn test_zsh_init_contains_wrapper() {
        let profile = test_profile();
        let output = generate_init(&Shells::Zsh, &AliasSet::default(), &profile);
        assert!(output.contains("am()"));
        assert!(output.contains("am reload zsh"));
        assert!(output.contains("--local"));
        assert!(output.contains("am hook zsh"));
    }

    #[test]
    fn test_zsh_init_contains_cd_hook() {
        let profile = test_profile();
        let output = generate_init(&Shells::Zsh, &AliasSet::default(), &profile);
        assert!(output.contains("chpwd_functions"));
        assert!(output.contains("am hook zsh"));
    }

    #[test]
    fn test_init_empty_profile_no_tracking_var() {
        let profile = Profile::new("empty".to_string(), None);
        let output = generate_init(&Shells::Fish, &AliasSet::default(), &profile);
        assert!(output.contains("__am_hook"));
        assert!(!output.contains("_AM_PROFILE_ALIASES"));
    }

    #[test]
    fn test_reload_unloads_old_and_loads_new() {
        let new_profile = test_profile();
        let output = generate_reload(&Shells::Fish, &new_profile, Some("old1,old2"));
        // unloads old
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        // loads new
        assert!(output.contains("alias gs \"git status\""));
        assert!(output.contains("alias ll \"ls -lha\""));
        // updates tracking
        assert!(output.contains("_AM_PROFILE_ALIASES"));
    }

    #[test]
    fn test_reload_zsh_unloads_with_unset_f() {
        let new_profile = test_profile();
        let output = generate_reload(&Shells::Zsh, &new_profile, Some("old1"));
        assert!(output.contains("unset -f old1"));
        assert!(output.contains("gs() { git status \"$@\"; }"));
    }

    #[test]
    fn test_reload_no_previous() {
        let new_profile = test_profile();
        let output = generate_reload(&Shells::Fish, &new_profile, None);
        // no unalias lines
        assert!(!output.contains("functions -e"));
        // has new aliases
        assert!(output.contains("alias gs"));
    }

    #[test]
    fn test_reload_to_empty_profile_clears_tracking() {
        let empty = Profile::new("empty".to_string(), None);
        let output = generate_reload(&Shells::Fish, &empty, Some("old1"));
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("set -e _AM_PROFILE_ALIASES"));
    }

    #[test]
    fn test_init_includes_global_aliases() {
        let mut globals = AliasSet::default();
        globals.insert(
            "ll".into(),
            crate::TomlAlias::Command("ls -lha".to_string()),
        );

        let profile = Profile::new("empty".to_string(), None);
        let output = generate_init(&Shells::Fish, &globals, &profile);
        assert!(output.contains("alias ll \"ls -lha\""));
    }

    #[test]
    fn test_init_global_before_profile() {
        let mut globals = AliasSet::default();
        globals.insert(
            "gl".into(),
            crate::TomlAlias::Command("global cmd".to_string()),
        );

        let profile = test_profile();
        let output = generate_init(&Shells::Fish, &globals, &profile);
        let gl_pos = output.find("gl").unwrap();
        let gs_pos = output.find("gs").unwrap();
        assert!(
            gl_pos < gs_pos,
            "global aliases should appear before profile aliases"
        );
    }
}
