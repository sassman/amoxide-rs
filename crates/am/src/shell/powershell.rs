use std::collections::BTreeMap;
use std::fmt::Debug;

use super::{
    build_wrapper_trie, has_template_args, substitute_powershell, substitute_quote_aware,
    ShellAdapter, WrapperNode,
};

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

impl ShellAdapter for PowerShell {
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
        let roots = build_wrapper_trie(entries);
        emit_ps_switch(&mut lines, &roots, 0, &ps_base, "  ");
        lines.push("}".into());
        lines.join("\n")
    }
}

/// Emit a `switch ($args[depth])` block for the given set of trie nodes.
fn emit_ps_switch(
    lines: &mut Vec<String>,
    nodes: &BTreeMap<String, WrapperNode>,
    depth: usize,
    ps_base: &str,
    indent: &str,
) {
    lines.push(format!("{indent}switch ($args[{depth}]) {{"));
    for (short, node) in nodes {
        lines.push(format!("{indent}  '{short}' {{"));
        emit_ps_node_body(lines, node, depth, ps_base, &format!("{indent}    "));
        lines.push(format!("{indent}  }}"));
    }
    lines.push(format!("{indent}  default {{ {ps_base} @args }}"));
    lines.push(format!("{indent}}}"));
}

/// Emit the body of a matched case at `depth`.
/// If the node has children, a nested switch is emitted; otherwise the leaf expansion.
fn emit_ps_node_body(
    lines: &mut Vec<String>,
    node: &WrapperNode,
    depth: usize,
    ps_base: &str,
    indent: &str,
) {
    let next_depth = depth + 1;
    if node.children.is_empty() {
        // Leaf node: emit the expansion.
        let expansion = node.leaf_longs.as_deref().unwrap_or_default().join(" ");
        if has_template_args(&expansion) {
            lines.push(format!(
                "{indent}{ps_base} {}",
                substitute_offset(&expansion, next_depth)
            ));
        } else {
            lines.push(format!(
                "{indent}{ps_base} {expansion} ($args | Select-Object -Skip {next_depth})"
            ));
        }
    } else {
        // Intermediate node with children: emit a nested switch.
        lines.push(format!("{indent}switch ($args[{next_depth}]) {{"));
        for (short, child) in &node.children {
            lines.push(format!("{indent}  '{short}' {{"));
            emit_ps_node_body(lines, child, next_depth, ps_base, &format!("{indent}    "));
            lines.push(format!("{indent}  }}"));
        }
        // Default for the nested switch: fall back to this node's leaf (if any) or @args.
        if let Some(longs) = &node.leaf_longs {
            let expansion = longs.join(" ");
            if has_template_args(&expansion) {
                lines.push(format!(
                    "{indent}  default {{ {ps_base} {} }}",
                    substitute_offset(&expansion, next_depth)
                ));
            } else {
                lines.push(format!(
                    "{indent}  default {{ {ps_base} {expansion} ($args | Select-Object -Skip {next_depth}) }}"
                ));
            }
        } else {
            lines.push(format!("{indent}  default {{ {ps_base} @args }}"));
        }
        lines.push(format!("{indent}}}"));
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

    #[test]
    fn test_powershell_subcommand_wrapper_two_level() {
        // jj:b:l → branch list
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["b".into(), "l".into()],
            long_subcommands: vec!["branch".into(), "list".into()],
        }];
        let output = PowerShell.subcommand_wrapper("jj", "command jj", &entries);
        assert!(
            output.contains("function global:jj {"),
            "missing function header"
        );
        // outer switch on $args[0]
        assert!(output.contains("switch ($args[0])"), "missing outer switch");
        // 'b' case
        assert!(output.contains("'b'"), "missing 'b' case");
        // nested switch on $args[1]
        assert!(
            output.contains("switch ($args[1])"),
            "missing nested switch"
        );
        // 'l' case inside nested switch
        assert!(output.contains("'l'"), "missing 'l' case");
        // expansion: branch list with skip 2
        assert!(
            output.contains("branch list ($args | Select-Object -Skip 2)"),
            "missing two-level expansion: {output}"
        );
    }

    #[test]
    fn test_powershell_subcommand_wrapper_template_nested() {
        // jj:b:l → branch list --limit {{1}}  (template at depth 2, offset=2 → $($args[2]))
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["b".into(), "l".into()],
            long_subcommands: vec!["branch".into(), "list --limit {{1}}".into()],
        }];
        let output = PowerShell.subcommand_wrapper("jj", "command jj", &entries);
        // offset=2: {{1}} → $($args[2])
        assert!(
            output.contains("$($args[2])"),
            "expected $($args[2]) for nested template: {output}"
        );
        assert!(
            !output.contains("Select-Object -Skip 2"),
            "should not have trailing spread when template present: {output}"
        );
    }

    #[test]
    fn test_powershell_subcommand_wrapper_mixed_levels() {
        // Mix of single-level 'ab' and two-level 'b:l' in the same wrapper.
        let entries = vec![
            SubcommandEntry {
                program: "jj".into(),
                short_subcommands: vec!["ab".into()],
                long_subcommands: vec!["abandon".into()],
            },
            SubcommandEntry {
                program: "jj".into(),
                short_subcommands: vec!["b".into(), "l".into()],
                long_subcommands: vec!["branch".into(), "list".into()],
            },
        ];
        let output = PowerShell.subcommand_wrapper("jj", "command jj", &entries);
        // Single-level: 'ab' → abandon with skip 1
        assert!(output.contains("'ab'"), "missing 'ab'");
        assert!(
            output.contains("abandon ($args | Select-Object -Skip 1)"),
            "missing single-level expansion: {output}"
        );
        // Two-level: 'b' → nested switch with 'l'
        assert!(output.contains("'b'"), "missing 'b'");
        assert!(
            output.contains("switch ($args[1])"),
            "missing nested switch"
        );
        assert!(output.contains("'l'"), "missing 'l'");
        assert!(
            output.contains("branch list ($args | Select-Object -Skip 2)"),
            "missing two-level expansion: {output}"
        );
    }
}
