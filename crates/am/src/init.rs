use crate::shell::{Shell, ShellContext};
use crate::subcommand::SubcommandSet;
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
/// `global_vars` — global var set used to substitute `{{name}}` in global aliases.
/// `profile_layers` — per-profile aliases/subcommands/vars in activation order.
/// `global_subcommands` — global subcommand aliases (independent of profile).
pub fn generate_init(
    ctx: &ShellContext,
    global_aliases: &AliasSet,
    global_vars: &crate::vars::VarSet,
    profile_layers: &[crate::precedence::ProfileLayer],
    global_subcommands: &SubcommandSet,
) -> String {
    use crate::precedence::Precedence;

    let shell_impl = ctx.shell.clone().as_shell(
        ctx.cfg,
        ctx.external_functions.clone(),
        ctx.external_aliases.clone(),
    );

    let outcome = Precedence::new()
        .with_global(global_aliases, global_subcommands, global_vars)
        .with_profiles(profile_layers)
        .resolve();
    let diff = outcome.diff;

    let mut output = diff.render(shell_impl.as_ref());

    // Wrapper function + cd hook + completions.
    if !output.is_empty() {
        output.push('\n');
    }
    output.push_str(&am_wrapper(ctx.shell));
    output.push('\n');
    output.push_str(&cd_hook_setup(ctx.shell));
    output.push('\n');
    output.push_str(&completions(ctx.shell));

    output
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
    use crate::env_vars;
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
        subs.as_mut().insert("jj:ab".into(), vec!["abandon".into()]);
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

    /// Test adapter mirroring the pre-vars `generate_init` shape: takes a single
    /// merged profile alias set and a global subcommand set, wraps them in a
    /// `ProfileLayer` with empty vars.
    fn init_for_test(
        ctx: &ShellContext,
        global_aliases: &AliasSet,
        profile_aliases: &AliasSet,
        subs: &SubcommandSet,
    ) -> String {
        generate_init(
            ctx,
            global_aliases,
            &crate::vars::VarSet::default(),
            &[crate::precedence::ProfileLayer {
                name: "_init".into(),
                aliases: profile_aliases.clone(),
                subcommands: subs.clone(),
                vars: crate::vars::VarSet::default(),
            }],
            &SubcommandSet::new(),
        )
    }

    #[test]
    fn test_fish_init_contains_aliases() {
        let aliases = test_aliases();
        let output = init_for_test(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("function gs\n    git status $argv\nend"));
        assert!(output.contains("function ll\n    ls -lha $argv\nend"));
    }

    #[test]
    fn test_fish_init_tracks_all_aliases() {
        let aliases = test_aliases();
        let output = init_for_test(
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
        let output = init_for_test(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("function am --wraps=am"));
        assert!(output.contains("am sync fish"));
    }

    #[test]
    fn test_fish_init_contains_cd_hook() {
        let aliases = test_aliases();
        let output = init_for_test(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("--on-variable PWD"));
        assert!(output.contains("am sync fish"));
    }

    #[test]
    fn test_zsh_init_contains_aliases() {
        let aliases = test_aliases();
        let output = init_for_test(
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
        let output = init_for_test(
            &default_ctx(&Shell::Zsh),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("am()"));
        assert!(output.contains("am sync zsh"));
    }

    #[test]
    fn test_zsh_init_contains_cd_hook() {
        let aliases = test_aliases();
        let output = init_for_test(
            &default_ctx(&Shell::Zsh),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("chpwd_functions"));
        assert!(output.contains("am sync zsh"));
    }

    #[test]
    fn test_init_empty_no_tracking_var() {
        let output = init_for_test(
            &default_ctx(&Shell::Fish),
            &AliasSet::default(),
            &AliasSet::default(),
            &SubcommandSet::new(),
        );
        assert!(output.contains("__am_hook"));
        assert!(!output.contains(env_vars::AM_ALIASES));
    }

    #[test]
    fn test_init_includes_global_aliases() {
        let mut globals = AliasSet::default();
        globals.insert(
            "ll".into(),
            crate::TomlAlias::Command("ls -lha".to_string()),
        );
        let output = init_for_test(
            &default_ctx(&Shell::Fish),
            &globals,
            &AliasSet::default(),
            &SubcommandSet::new(),
        );
        assert!(output.contains("function ll\n    ls -lha $argv\nend"));
    }

    #[test]
    fn test_init_global_before_profile() {
        let mut globals = AliasSet::default();
        globals.insert(
            "gl".into(),
            crate::TomlAlias::Command("global cmd".to_string()),
        );
        let aliases = test_aliases();
        let output = init_for_test(
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
    fn test_bash_init_contains_aliases() {
        let aliases = test_aliases();
        let output = init_for_test(
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
        let output = init_for_test(
            &default_ctx(&Shell::Bash),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("am()"));
        assert!(output.contains("am sync bash"));
    }

    #[test]
    fn test_bash_init_contains_cd_hook() {
        let aliases = test_aliases();
        let output = init_for_test(
            &default_ctx(&Shell::Bash),
            &AliasSet::default(),
            &aliases,
            &SubcommandSet::new(),
        );
        assert!(output.contains("PROMPT_COMMAND"));
        assert!(output.contains("__am_hook"));
        assert!(output.contains("__am_prev_dir"));
        assert!(output.contains("am sync bash"));
    }

    #[test]
    fn test_bash_init_contains_subcommand_wrapper() {
        let subs = test_subcommands();
        let output = init_for_test(
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
        let output = init_for_test(
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
        let output = init_for_test(
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
        let output = init_for_test(
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
        let output = init_for_test(&ctx, &AliasSet::default(), &aliases, &SubcommandSet::new());
        assert!(output.contains("abbr --add gs \"git status\""));
    }

    #[test]
    fn init_delegates_alias_emission_to_precedence() {
        // init output must match PrecedenceDiff::render output for the same inputs.
        let aliases = test_aliases();
        let ctx = default_ctx(&Shell::Fish);
        let output = init_for_test(&ctx, &AliasSet::default(), &aliases, &SubcommandSet::new());
        // Everything should be in _AM_ALIASES with name|hash format (not bare names).
        let gs_hash = crate::trust::compute_short_hash(b"git status");
        assert!(
            output.contains(&format!("gs|{gs_hash}")),
            "init must use name|hash format in _AM_ALIASES, got: {output}"
        );
    }
}
