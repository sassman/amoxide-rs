use crate::env_vars;
use crate::shell::{Shell, ShellContext};
use crate::subcommand::{group_by_program, SubcommandSet};
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
/// `subcommands` — merged subcommand aliases (global + active profiles).
pub fn generate_init(
    ctx: &ShellContext,
    global_aliases: &AliasSet,
    profile_aliases: &AliasSet,
    subcommands: &SubcommandSet,
) -> String {
    let shell_impl = ctx.shell.clone().as_shell(
        ctx.cfg,
        ctx.external_functions.clone(),
        ctx.external_aliases.clone(),
    );
    let mut lines: Vec<String> = Vec::new();
    let mut all_names: Vec<String> = Vec::new();

    // Determine which program names have subcommand wrappers
    let subcmd_groups = group_by_program(subcommands);
    let programs_with_wrappers: std::collections::BTreeSet<&str> =
        subcmd_groups.keys().map(|s| s.as_str()).collect();

    // Emit global aliases (skip those absorbed by subcommand wrappers)
    for (alias_name, alias_value) in global_aliases.iter() {
        let name = alias_name.as_ref();
        if !programs_with_wrappers.contains(name) {
            lines.push(shell_impl.alias(&alias_value.as_entry(name)));
        }
        all_names.push(name.to_string());
    }

    // Emit profile aliases (skip those absorbed by subcommand wrappers)
    for (alias_name, alias_value) in profile_aliases.iter() {
        let name = alias_name.as_ref();
        if !programs_with_wrappers.contains(name) {
            lines.push(shell_impl.alias(&alias_value.as_entry(name)));
        }
        all_names.push(name.to_string());
    }

    // Emit subcommand wrappers
    for (program, entries) in &subcmd_groups {
        // Determine base command: alias value if regular alias exists, else "command <program>"
        let all_aliases = global_aliases.iter().chain(profile_aliases.iter());
        let base_cmd = all_aliases
            .filter(|(n, _)| n.as_ref() == program.as_str())
            .map(|(_, v)| v.command().to_string())
            .last()
            .unwrap_or_else(|| format!("command {program}"));

        lines.push(shell_impl.subcommand_wrapper(program, &base_cmd, entries));
        all_names.push(program.to_string());
    }

    // Track all loaded aliases (global + profile + subcommand wrappers) for reload cleanup
    if !all_names.is_empty() {
        all_names.sort();
        all_names.dedup();
        lines.push(shell_impl.set_env(env_vars::AM_ALIASES, &all_names.join(",")));
    }
    // Clean up legacy tracking var from older versions
    lines.push(shell_impl.unset_env(env_vars::AM_PROFILE_ALIASES_LEGACY));

    // Wrapper function
    lines.push(String::new());
    lines.push(am_wrapper(ctx.shell));

    // cd hook for project aliases
    lines.push(String::new());
    lines.push(cd_hook_setup(ctx.shell));

    // Shell completions
    lines.push(String::new());
    lines.push(completions(ctx.shell));

    lines.join("\n")
}

/// Like [`generate_init`] but prepends force-cleanup lines for `prev_names`.
/// Each name is unloaded using all possible shell forms before the normal init runs.
/// Intended for testing; production code reads prev_names from env vars in `update.rs`.
pub fn generate_force_init(
    ctx: &ShellContext,
    global_aliases: &AliasSet,
    profile_aliases: &AliasSet,
    subcommands: &SubcommandSet,
    prev_names: &[String],
) -> String {
    let shell_impl = ctx
        .shell
        .clone()
        .as_shell(ctx.cfg, Default::default(), Default::default());
    let mut output = String::new();
    for name in prev_names {
        output.push_str(&shell_impl.force_unalias(name));
        output.push('\n');
    }
    // Clear project-alias tracking so __am_hook reloads them fresh.
    output.push_str(&shell_impl.unset_env(crate::env_vars::AM_PROJECT_ALIASES));
    output.push('\n');
    output.push_str(&shell_impl.unset_env(crate::env_vars::AM_PROJECT_PATH));
    output.push('\n');
    output.push_str(&generate_init(
        ctx,
        global_aliases,
        profile_aliases,
        subcommands,
    ));
    output
}

/// Generate shell code to reload all aliases (global + profile) after a mutation.
/// Unloads old aliases, loads new ones, updates the tracking env var.
pub fn generate_reload(
    ctx: &ShellContext,
    global_aliases: &AliasSet,
    profile_aliases: &AliasSet,
    subcommands: &SubcommandSet,
    previous_aliases: Option<&str>,
) -> String {
    let shell_impl = ctx.shell.clone().as_shell(
        ctx.cfg,
        ctx.external_functions.clone(),
        ctx.external_aliases.clone(),
    );
    let mut lines: Vec<String> = Vec::new();

    // Unload all previously tracked aliases
    let prev: Vec<&str> = previous_aliases
        .filter(|s| !s.is_empty())
        .map(|s| s.split(',').collect())
        .unwrap_or_default();

    for alias_name in &prev {
        lines.push(shell_impl.unalias(alias_name));
    }

    // Determine which program names have subcommand wrappers
    let subcmd_groups = group_by_program(subcommands);
    let programs_with_wrappers: std::collections::BTreeSet<&str> =
        subcmd_groups.keys().map(|s| s.as_str()).collect();

    // Load global + profile aliases (skip those absorbed by subcommand wrappers)
    let mut all_names: Vec<String> = Vec::new();

    for (alias_name, alias_value) in global_aliases.iter() {
        let name = alias_name.as_ref();
        if !programs_with_wrappers.contains(name) {
            lines.push(shell_impl.alias(&alias_value.as_entry(name)));
        }
        all_names.push(name.to_string());
    }

    for (alias_name, alias_value) in profile_aliases.iter() {
        let name = alias_name.as_ref();
        if !programs_with_wrappers.contains(name) {
            lines.push(shell_impl.alias(&alias_value.as_entry(name)));
        }
        all_names.push(name.to_string());
    }

    // Emit subcommand wrappers
    for (program, entries) in &subcmd_groups {
        // Determine base command: alias value if regular alias exists, else "command <program>"
        let all_aliases = global_aliases.iter().chain(profile_aliases.iter());
        let base_cmd = all_aliases
            .filter(|(n, _)| n.as_ref() == program.as_str())
            .map(|(_, v)| v.command().to_string())
            .last()
            .unwrap_or_else(|| format!("command {program}"));

        lines.push(shell_impl.subcommand_wrapper(program, &base_cmd, entries));
        all_names.push(program.to_string());
    }

    // Update tracking
    if all_names.is_empty() {
        if !prev.is_empty() {
            lines.push(shell_impl.unset_env(env_vars::AM_ALIASES));
        }
    } else {
        all_names.sort();
        all_names.dedup();
        lines.push(shell_impl.set_env(env_vars::AM_ALIASES, &all_names.join(",")));
    }

    lines.join("\n")
}

fn shell_script(template: &str, shell: &Shell) -> String {
    template.replace("__SHELL__", &shell.to_string())
}

fn am_wrapper(shell: &Shell) -> String {
    match shell {
        Shell::Bash | Shell::Brush => shell_script(WRAPPER_BASH, shell),
        Shell::Fish => shell_script(WRAPPER_FISH, shell),
        Shell::Powershell => shell_script(WRAPPER_PS1, shell),
        Shell::Zsh => shell_script(WRAPPER_ZSH, shell),
    }
}

fn cd_hook_setup(shell: &Shell) -> String {
    match shell {
        Shell::Bash | Shell::Brush => shell_script(HOOK_BASH, shell),
        Shell::Fish => shell_script(HOOK_FISH, shell),
        Shell::Powershell => shell_script(HOOK_PS1, shell),
        Shell::Zsh => shell_script(HOOK_ZSH, shell),
    }
}

fn completions(shell: &Shell) -> String {
    match shell {
        Shell::Bash | Shell::Brush => {
            include_str!(concat!(env!("OUT_DIR"), "/am.bash")).to_string()
        }
        Shell::Fish => COMPLETIONS_FISH.to_string(),
        Shell::Powershell => powershell_completions(),
        Shell::Zsh => COMPLETIONS_ZSH.to_string(),
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
    use crate::config::ShellsTomlConfig;
    use crate::shell::ShellContext;
    use crate::subcommand::SubcommandSet;
    use crate::{AliasName, TomlAlias};

    static DEFAULT_CFG: std::sync::LazyLock<ShellsTomlConfig> =
        std::sync::LazyLock::new(ShellsTomlConfig::default);

    fn default_ctx(shell: &Shell) -> ShellContext<'_> {
        ShellContext {
            shell,
            cfg: &DEFAULT_CFG,
            cwd: std::path::Path::new("/tmp"),
            external_functions: Default::default(),
            external_aliases: Default::default(),
        }
    }

    fn test_subcommands() -> SubcommandSet {
        let mut subs = SubcommandSet::new();
        subs.insert("jj:ab".into(), vec!["abandon".into()]);
        subs
    }

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
        let output = generate_init(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("alias gs \"git status\""));
        assert!(output.contains("alias ll \"ls -lha\""));
    }

    #[test]
    fn test_fish_init_tracks_all_aliases() {
        let aliases = test_aliases();
        let output = generate_init(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains(env_vars::AM_ALIASES));
    }

    #[test]
    fn test_fish_init_contains_wrapper() {
        let aliases = test_aliases();
        let output = generate_init(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("function am --wraps=am"));
        assert!(output.contains("am reload fish"));
        assert!(output.contains("--local"));
        assert!(output.contains("am hook fish"));
    }

    #[test]
    fn test_fish_init_contains_cd_hook() {
        let aliases = test_aliases();
        let output = generate_init(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("--on-variable PWD"));
        assert!(output.contains("am hook fish"));
    }

    #[test]
    fn test_zsh_init_contains_aliases() {
        let aliases = test_aliases();
        let output = generate_init(
            &default_ctx(&Shell::Zsh),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("alias gs=\"git status\""));
        assert!(output.contains("alias ll=\"ls -lha\""));
    }

    #[test]
    fn test_zsh_init_contains_wrapper() {
        let aliases = test_aliases();
        let output = generate_init(
            &default_ctx(&Shell::Zsh),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("am()"));
        assert!(output.contains("am reload zsh"));
        assert!(output.contains("--local"));
        assert!(output.contains("am hook zsh"));
    }

    #[test]
    fn test_zsh_init_contains_cd_hook() {
        let aliases = test_aliases();
        let output = generate_init(
            &default_ctx(&Shell::Zsh),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("chpwd_functions"));
        assert!(output.contains("am hook zsh"));
    }

    #[test]
    fn test_init_empty_no_tracking_var() {
        let output = generate_init(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &AliasSet::default(),
            &SubcommandSet::new(),
        );
        assert!(output.contains("__am_hook"));
        assert!(!output.contains(env_vars::AM_ALIASES));
    }

    #[test]
    fn test_reload_unloads_old_and_loads_new() {
        let aliases = test_aliases();
        let output = generate_reload(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
            Some("old1,old2"),
        );
        assert!(output.contains("functions -e old1"));
        assert!(output.contains("functions -e old2"));
        assert!(output.contains("alias gs \"git status\""));
        assert!(output.contains("alias ll \"ls -lha\""));
        assert!(output.contains(env_vars::AM_ALIASES));
    }

    #[test]
    fn test_reload_zsh_unloads_with_unset_f() {
        let aliases = test_aliases();
        let output = generate_reload(
            &default_ctx(&Shell::Zsh),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
            Some("old1"),
        );
        assert!(output.contains("unset -f old1"));
        assert!(output.contains("alias gs=\"git status\""));
    }

    #[test]
    fn test_reload_no_previous() {
        let aliases = test_aliases();
        let output = generate_reload(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
            None,
        );
        assert!(!output.contains("functions -e"));
        assert!(output.contains("alias gs"));
    }

    #[test]
    fn test_reload_to_empty_clears_tracking() {
        let output = generate_reload(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &AliasSet::default(),
            &SubcommandSet::new(),
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
        let output = generate_init(
            &default_ctx(&Shell::Fish),
            &globals,
            &AliasSet::default(),
            &SubcommandSet::new(),
        );
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
        let output = generate_init(
            &default_ctx(&Shell::Fish),
            &globals,
            &aliases,
            &SubcommandSet::new(),
        );
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
        let output = generate_reload(
            &default_ctx(&Shell::Fish),
            &globals,
            &AliasSet::default(),
            &SubcommandSet::new(),
            Some("old"),
        );
        assert!(output.contains("functions -e old"));
        assert!(output.contains("alias ll \"ls -lha\""));
    }

    #[test]
    fn test_bash_init_contains_aliases() {
        let aliases = test_aliases();
        let output = generate_init(
            &default_ctx(&Shell::Bash),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("alias gs=\"git status\""));
        assert!(output.contains("alias ll=\"ls -lha\""));
    }

    #[test]
    fn test_bash_init_contains_wrapper() {
        let aliases = test_aliases();
        let output = generate_init(
            &default_ctx(&Shell::Bash),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("am()"));
        assert!(output.contains("am reload bash"));
        assert!(output.contains("--local"));
        assert!(output.contains("am hook bash"));
    }

    #[test]
    fn test_bash_init_contains_cd_hook() {
        let aliases = test_aliases();
        let output = generate_init(
            &default_ctx(&Shell::Bash),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("PROMPT_COMMAND"));
        assert!(output.contains("__am_hook"));
        assert!(output.contains("__am_prev_dir"));
        assert!(output.contains("am hook bash"));
    }

    #[test]
    fn test_reload_bash_unloads_with_unset_f() {
        let aliases = test_aliases();
        let output = generate_reload(
            &default_ctx(&Shell::Bash),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
            Some("old1"),
        );
        assert!(output.contains("unset -f old1"));
        assert!(output.contains("alias gs=\"git status\""));
    }

    #[test]
    fn test_bash_init_contains_subcommand_wrapper() {
        let subs = test_subcommands();
        let output = generate_init(
            &default_ctx(&Shell::Bash),
            &AliasSet::default(),
            &AliasSet::default(),
            &subs,
        );
        assert!(output.contains("jj() {"));
        assert!(output.contains("ab) shift; command jj abandon"));
        assert!(output.contains("*) command jj \"$@\""));
    }

    #[test]
    fn test_fish_init_contains_subcommand_wrapper() {
        let subs = test_subcommands();
        let output = generate_init(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &AliasSet::default(),
            &subs,
        );
        assert!(output.contains("function jj --wraps=jj"));
        assert!(output.contains("case 'ab'"));
        assert!(output.contains("command jj abandon"));
    }

    #[test]
    fn test_init_subcommand_absorbs_regular_alias() {
        let mut aliases = AliasSet::default();
        aliases.insert("jj".into(), TomlAlias::Command("just-a-joke".into()));
        let subs = test_subcommands();
        let output = generate_init(
            &default_ctx(&Shell::Bash),
            &aliases,
            &AliasSet::default(),
            &subs,
        );
        // Wrapper should use the alias value as base_cmd
        assert!(output.contains("just-a-joke abandon"));
        assert!(output.contains("just-a-joke \"$@\""));
        // Should NOT also emit a regular alias function for jj
        // (the wrapper subsumes it — count occurrences of "jj() {")
        let wrapper_count = output.matches("jj() {").count();
        assert_eq!(wrapper_count, 1);
    }

    #[test]
    fn test_init_subcommand_tracked_in_am_aliases() {
        let subs = test_subcommands();
        let output = generate_init(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &AliasSet::default(),
            &subs,
        );
        assert!(output.contains(env_vars::AM_ALIASES));
        assert!(output.contains("jj"));
    }
    #[test]
    fn test_fish_init_with_abbr_mode() {
        use crate::config::{FishConfig, ShellsTomlConfig};
        let cfg = ShellsTomlConfig {
            fish: Some(FishConfig { use_abbr: true }),
        };
        let cwd = std::path::Path::new("/tmp");
        let ctx = ShellContext {
            shell: &Shell::Fish,
            cfg: &cfg,
            cwd,
            external_functions: Default::default(),
            external_aliases: Default::default(),
        };
        let mut aliases = AliasSet::default();
        aliases.insert(
            AliasName::from("gs"),
            crate::TomlAlias::Command("git status".to_string()),
        );
        let output = generate_init(&ctx, &AliasSet::default(), &aliases, &SubcommandSet::new());
        assert!(output.contains("abbr --add gs \"git status\""));
    }

    #[test]
    fn test_fish_reload_with_abbr_unloads_via_abbr_erase() {
        use crate::config::{FishConfig, ShellsTomlConfig};
        let cfg = ShellsTomlConfig {
            fish: Some(FishConfig { use_abbr: true }),
        };
        let cwd = std::path::Path::new("/tmp");
        let ctx = ShellContext {
            shell: &Shell::Fish,
            cfg: &cfg,
            cwd,
            external_functions: Default::default(),
            external_aliases: Default::default(),
        };
        let output = generate_reload(
            &ctx,
            &AliasSet::default(),
            &AliasSet::default(),
            &SubcommandSet::new(),
            Some("old1"),
        );
        assert!(output.contains("abbr --erase old1"));
    }
}
