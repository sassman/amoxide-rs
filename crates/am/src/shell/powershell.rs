use std::collections::BTreeMap;
use std::fmt::Debug;

use super::shell::TEMPLATE_RE;
use super::{
    build_wrapper_trie, has_template_args, substitute_powershell, substitute_quote_aware,
    ShellAdapter, WrapperNode,
};

/// Substitute `{{N}}` → `$($args[N-1+offset])` and `{{@}}` → `($args | Select-Object -Skip offset)`.
/// Used in subcommand wrappers where PowerShell doesn't shift args.
fn substitute_offset(cmd: &str, offset: usize) -> String {
    substitute_quote_aware(cmd, &TEMPLATE_RE, |n| match n {
        "@" => format!("($args | Select-Object -Skip {offset})"),
        n => {
            let idx: usize = n.parse::<usize>().unwrap() - 1 + offset;
            format!("$($args[{idx}])")
        }
    })
}
use crate::alias::AliasEntry;
use crate::subcommand::SubcommandEntry;

/// Render `line` as the interior of a PowerShell double-quoted string literal.
/// Escapes the two metacharacters that would break the surrounding `"…"`:
/// the backtick (PS's escape character) and the literal `"`. Leaves `$(...)`
/// subexpressions and `$args[N]` references intact so PS interpolates them
/// at call time — that's the whole point of the trace line.
fn ps_trace_escape(line: &str) -> String {
    line.replace('`', "``").replace('"', "`\"")
}

/// Emit a one-line debug gate. When `$env:__AM_DEBUG` is `1`, writes the
/// post-expansion command to stderr (so it doesn't intermix with the real
/// stdout the wrapped command produces). When unset/`0`, the `if` short-
/// circuits — no side effect.
fn ps_trace(line: &str) -> String {
    format!(
        "if ($env:__AM_DEBUG -eq '1') {{ [Console]::Error.WriteLine(\"[am] {}\") }}; ",
        ps_trace_escape(line)
    )
}

#[derive(Debug, Default)]
pub struct PowerShell;

impl ShellAdapter for PowerShell {
    fn unalias(&self, alias_name: &str) -> String {
        format!("if (Test-Path Function:\\{alias_name}) {{ Remove-Item Function:\\{alias_name} }}")
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        if !entry.raw && has_template_args(entry.command) {
            let body = substitute_powershell(entry.command);
            let trace = ps_trace(&body);
            format!("function global:{} {{ {trace}{body} }}", entry.name)
        } else {
            // Non-parameterised: real call uses `@args` (splat); trace shows
            // `$args` (space-joined interpolation) — visually equivalent for
            // simple cases and avoids the splat-vs-interpolate mismatch
            // inside a "..." literal.
            let trace = ps_trace(&format!("{} $args", entry.command));
            format!(
                "function global:{} {{ {trace}{} @args }}",
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
    let outer_default_trace = ps_trace(&format!("{ps_base} $args"));
    lines.push(format!(
        "{indent}  default {{ {outer_default_trace}{ps_base} @args }}"
    ));
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
        // Leaf node: emit the expansion (and its trace).
        let expansion = node.leaf_longs.as_deref().unwrap_or_default().join(" ");
        if has_template_args(&expansion) {
            let resolved = substitute_offset(&expansion, next_depth);
            let trace = ps_trace(&format!("{ps_base} {resolved}"));
            lines.push(format!("{indent}{trace}{ps_base} {resolved}"));
        } else {
            // Trace shows the tail via $(($args | Select-Object -Skip N)) so
            // the printed line reflects the actual args the wrapped command
            // will receive, not the literal pipeline expression.
            let tail_real = format!("($args | Select-Object -Skip {next_depth})");
            let tail_trace = format!("$(($args | Select-Object -Skip {next_depth}))");
            let trace = ps_trace(&format!("{ps_base} {expansion} {tail_trace}"));
            lines.push(format!("{indent}{trace}{ps_base} {expansion} {tail_real}"));
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
                let resolved = substitute_offset(&expansion, next_depth);
                let trace = ps_trace(&format!("{ps_base} {resolved}"));
                lines.push(format!(
                    "{indent}  default {{ {trace}{ps_base} {resolved} }}"
                ));
            } else {
                let tail_real = format!("($args | Select-Object -Skip {next_depth})");
                let tail_trace = format!("$(($args | Select-Object -Skip {next_depth}))");
                let trace = ps_trace(&format!("{ps_base} {expansion} {tail_trace}"));
                lines.push(format!(
                    "{indent}  default {{ {trace}{ps_base} {expansion} {tail_real} }}"
                ));
            }
        } else {
            let trace = ps_trace(&format!("{ps_base} $args"));
            lines.push(format!("{indent}  default {{ {trace}{ps_base} @args }}"));
        }
        lines.push(format!("{indent}}}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::test_helpers::{raw, simple};
    use crate::subcommand::SubcommandEntry;

    /// Inline copy of the debug-gate format used inside emitted PS function
    /// bodies. Tests build the expected output by formatting against the
    /// post-expansion form — same shape `ps_trace` produces in src.
    fn gate(traced: &str) -> String {
        format!(
            "if ($env:__AM_DEBUG -eq '1') {{ [Console]::Error.WriteLine(\"[am] {traced}\") }}; "
        )
    }

    #[test]
    fn test_simple_alias() {
        assert_eq!(
            PowerShell.alias(&simple("gs", "git status")),
            format!(
                "function global:gs {{ {}git status @args }}",
                gate("git status $args")
            )
        );
    }

    #[test]
    fn test_parameterized_alias() {
        assert_eq!(
            PowerShell.alias(&simple("cmf", "cm feat: {{@}}")),
            format!(
                "function global:cmf {{ {}cm feat: $args }}",
                gate("cm feat: $args")
            )
        );
        assert_eq!(
            PowerShell.alias(&simple("x", "echo {{1}} and {{2}}")),
            format!(
                "function global:x {{ {}echo $($args[0]) and $($args[1]) }}",
                gate("echo $($args[0]) and $($args[1])")
            )
        );
    }

    #[test]
    fn test_raw_alias() {
        assert_eq!(
            PowerShell.alias(&raw("my-awk", "awk '{print {{1}}}'")),
            format!(
                "function global:my-awk {{ {}awk '{{print {{{{1}}}}}}' @args }}",
                gate("awk '{print {{1}}}' $args")
            )
        );
    }

    #[test]
    fn test_ps_trace_escapes_inner_quotes() {
        // Body with an embedded "..." (the headline case from issue #143):
        // git cm → commit -m "{{1}}" --signoff
        let out = PowerShell.alias(&simple("cm", "git commit -m \"{{1}}\" --signoff"));
        // Inside the trace literal, the inner " must be backtick-escaped so
        // PS doesn't terminate the WriteLine argument.
        assert!(
            out.contains("[am] git commit -m `\"$($args[0])`\" --signoff"),
            "trace should backtick-escape inner quotes: {out}"
        );
        // The real call still uses unescaped double-quotes.
        assert!(
            out.contains("git commit -m \"$($args[0])\" --signoff }"),
            "real call must keep literal quotes: {out}"
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
            description: None,
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
            description: None,
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
            description: None,
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
            description: None,
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
            description: None,
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
                description: None,
            },
            SubcommandEntry {
                program: "jj".into(),
                short_subcommands: vec!["b".into(), "l".into()],
                long_subcommands: vec!["branch".into(), "list".into()],
                description: None,
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
