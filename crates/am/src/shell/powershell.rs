use std::fmt::Debug;

use super::{has_template_args, substitute_powershell, substitute_quote_aware, Shell};

/// Substitute `{{N}}` → `$($args[N-1+offset])` and `{{@}}` → `($args | Select-Object -Skip offset)`.
/// Used in subcommand wrappers where PowerShell doesn't shift args.
fn substitute_offset(cmd: &str, offset: usize) -> String {
    substitute_quote_aware(cmd, |n| match n {
        "@" => format!("($args | Select-Object -Skip {offset})"),
        n => {
            let idx: usize = n.parse::<usize>().unwrap() - 1 + offset;
            format!("$($args[{idx}])")
        }
    })
}
use crate::alias::AliasEntry;
use crate::subcommand::SubcommandEntry;

#[derive(Debug, Default)]
pub struct PowerShell;

impl Shell for PowerShell {
    fn unalias(&self, alias_name: &str) -> String {
        format!("if (Test-Path Function:\\{alias_name}) {{ Remove-Item Function:\\{alias_name} }}")
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        if !entry.raw && has_template_args(entry.command) {
            let body = substitute_powershell(entry.command);
            format!("function global:{} {{ {} }}", entry.name, body)
        } else {
            format!(
                "function global:{} {{ {} @args }}",
                entry.name, entry.command
            )
        }
    }

    fn set_env(&self, var_name: &str, value: &str) -> String {
        format!("$env:{var_name} = \"{value}\"")
    }

    fn unset_env(&self, var_name: &str) -> String {
        format!("Remove-Item -ErrorAction SilentlyContinue Env:{var_name}")
    }

    fn echo(&self, message: &str) -> String {
        format!("Write-Host '{}'", message.replace('\'', "''"))
    }

    fn subcommand_wrapper(
        &self,
        program: &str,
        base_cmd: &str,
        entries: &[SubcommandEntry],
    ) -> String {
        // For PowerShell, base_cmd is used with & operator for external commands.
        // "command jj" → "& (Get-Command jj -CommandType Application).Source"
        // alias value → "& alias-value"
        let ps_base = if let Some(bin) = base_cmd.strip_prefix("command ") {
            format!("& (Get-Command {bin} -CommandType Application).Source")
        } else {
            format!("& {base_cmd}")
        };

        let mut lines = Vec::new();
        lines.push(format!("function global:{program} {{"));
        lines.push("  switch ($args[0]) {".into());

        // Only handle single-level for now (multi-level nesting in PS is verbose)
        for entry in entries {
            if entry.short_subcommands.len() == 1 {
                let short = &entry.short_subcommands[0];
                let expansion = &entry.long_subcommands[0];
                if has_template_args(expansion) {
                    lines.push(format!(
                        "    '{short}' {{ {ps_base} {} }}",
                        substitute_offset(expansion, 1)
                    ));
                } else {
                    lines.push(format!(
                        "    '{short}' {{ {ps_base} {expansion} ($args | Select-Object -Skip 1) }}"
                    ));
                }
            }
        }

        lines.push(format!("    default {{ {ps_base} @args }}"));
        lines.push("  }".into());
        lines.push("}".into());
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::test_helpers::{raw, simple};
    use crate::subcommand::SubcommandEntry;

    #[test]
    fn test_simple_alias() {
        assert_eq!(
            PowerShell.alias(&simple("gs", "git status")),
            "function global:gs { git status @args }"
        );
    }

    #[test]
    fn test_parameterized_alias() {
        assert_eq!(
            PowerShell.alias(&simple("cmf", "cm feat: {{@}}")),
            "function global:cmf { cm feat: $args }"
        );
        assert_eq!(
            PowerShell.alias(&simple("x", "echo {{1}} and {{2}}")),
            "function global:x { echo $($args[0]) and $($args[1]) }"
        );
    }

    #[test]
    fn test_raw_alias() {
        assert_eq!(
            PowerShell.alias(&raw("my-awk", "awk '{print {{1}}}'")),
            "function global:my-awk { awk '{print {{1}}}' @args }"
        );
    }

    #[test]
    fn test_unalias() {
        assert_eq!(
            PowerShell.unalias("gs"),
            "if (Test-Path Function:\\gs) { Remove-Item Function:\\gs }"
        );
    }

    #[test]
    fn test_set_env() {
        assert_eq!(PowerShell.set_env("FOO", "bar"), "$env:FOO = \"bar\"");
    }

    #[test]
    fn test_unset_env() {
        assert_eq!(
            PowerShell.unset_env("FOO"),
            "Remove-Item -ErrorAction SilentlyContinue Env:FOO"
        );
    }

    #[test]
    fn test_powershell_subcommand_wrapper_parameterized() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["ab".into()],
            long_subcommands: vec!["abandon --rev {{1}}".into()],
        }];
        let output = PowerShell.subcommand_wrapper("jj", "command jj", &entries);
        // offset=1: {{1}} → $($args[1])
        assert!(output.contains("$($args[1])"));
        assert!(!output.contains("Select-Object -Skip 1")); // no trailing spread when templates used
    }

    #[test]
    fn test_powershell_subcommand_wrapper_parameterized_all_args() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["l".into()],
            long_subcommands: vec!["log {{@}}".into()],
        }];
        let output = PowerShell.subcommand_wrapper("jj", "command jj", &entries);
        // offset=1: {{@}} → ($args | Select-Object -Skip 1)
        assert!(output.contains("($args | Select-Object -Skip 1)"));
    }

    #[test]
    fn test_powershell_subcommand_wrapper() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["ab".into()],
            long_subcommands: vec!["abandon".into()],
        }];
        let output = PowerShell.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("function global:jj {"));
        assert!(output.contains("'ab'"));
        assert!(output.contains("abandon"));
        assert!(output.contains("default"));
    }
}
