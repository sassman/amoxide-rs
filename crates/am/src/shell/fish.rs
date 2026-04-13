use std::collections::BTreeMap;
use std::fmt::Debug;

use super::{
    build_wrapper_trie, has_template_args, quote_cmd, substitute_fish, Shell, WrapperNode,
    TEMPLATE_RE,
};
use crate::alias::AliasEntry;
use crate::subcommand::SubcommandEntry;

/// Substitute `{{N}}` → `$argv[N+offset]` and `{{@}}` → `$argv[offset+1..]`.
///
/// For Fish, single-quoted tokens that contain a template are converted to
/// double-quoted tokens rather than using the bash-style quote-breaking pattern
/// (`'before'$var'after'`).  This avoids the ambiguity where Fish could parse
/// `($argv[N])` as a command substitution when surrounding parens are at the
/// boundary of what were originally single-quote regions.
fn substitute_offset(cmd: &str, offset: usize) -> String {
    let make_sub = |n: &str| -> String {
        match n {
            "@" => format!("$argv[{}..]", offset + 1),
            n => {
                let idx: usize = n.parse::<usize>().unwrap() + offset;
                format!("$argv[{idx}]")
            }
        }
    };

    let mut result = String::new();
    let mut rest = cmd;

    while !rest.is_empty() {
        let next_sq = rest.find('\'');
        let next_tmpl = TEMPLATE_RE.find(rest).map(|m| m.start());

        match (next_sq, next_tmpl) {
            (Some(sq), Some(tmpl)) if sq < tmpl => {
                // A single-quote opens before the next template.
                // Peek ahead: does this single-quoted region contain a template?
                let after_open = &rest[sq + 1..];
                if let Some(close_rel) = after_open.find('\'') {
                    let sq_content = &after_open[..close_rel];
                    if has_template_args(sq_content) {
                        // Break the single-quoted token at each `{{N}}` and concatenate
                        // with the unquoted variable reference.  This keeps static content
                        // inside single quotes (no escaping needed) and lets Fish expand
                        // `$argv[N]` in the unquoted segments between them.
                        //
                        // e.g. 'toggle({{1}})' → 'toggle('$argv[2]')'
                        result.push_str(&rest[..sq]);
                        result.push('\'');
                        let mut inner = sq_content;
                        while !inner.is_empty() {
                            match TEMPLATE_RE.find(inner) {
                                Some(m) => {
                                    result.push_str(&inner[..m.start()]);
                                    result.push('\''); // close single-quote
                                    let cap = TEMPLATE_RE.captures(inner).unwrap();
                                    result.push_str(&make_sub(&cap[1]));
                                    result.push('\''); // reopen single-quote
                                    inner = &inner[m.end()..];
                                }
                                None => {
                                    result.push_str(inner);
                                    break;
                                }
                            }
                        }
                        result.push('\'');
                        rest = &after_open[close_rel + 1..];
                    } else {
                        // No template in this region: keep as-is.
                        let token_end = sq + 1 + close_rel + 1;
                        result.push_str(&rest[..token_end]);
                        rest = &rest[token_end..];
                    }
                } else {
                    // Unclosed single quote — emit remainder unchanged.
                    result.push_str(rest);
                    break;
                }
            }
            (_, Some(tmpl)) => {
                // Template outside single quotes: substitute directly.
                result.push_str(&rest[..tmpl]);
                let m = TEMPLATE_RE.find(rest).unwrap();
                let cap = TEMPLATE_RE.captures(rest).unwrap();
                result.push_str(&make_sub(&cap[1]));
                rest = &rest[m.end()..];
            }
            _ => {
                result.push_str(rest);
                break;
            }
        }
    }
    result
}

#[derive(Debug, Default)]
pub struct Fish;

impl Shell for Fish {
    fn unalias(&self, alias_name: &str) -> String {
        format!("functions -e {alias_name}")
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        if !entry.raw && has_template_args(entry.command) {
            let body = substitute_fish(entry.command);
            format!("function {}\n    {}\nend", entry.name, body)
        } else {
            format!("alias {} {}", entry.name, quote_cmd(entry.command))
        }
    }

    fn set_env(&self, var_name: &str, value: &str) -> String {
        format!("set -gx {var_name} {}", quote_cmd(value))
    }

    fn unset_env(&self, var_name: &str) -> String {
        format!("set -e {var_name}")
    }

    fn echo(&self, message: &str) -> String {
        format!("echo '{}'", message.replace('\'', "\\'"))
    }

    fn subcommand_wrapper(
        &self,
        program: &str,
        base_cmd: &str,
        entries: &[SubcommandEntry],
    ) -> String {
        let mut lines = Vec::new();
        lines.push(format!("function {program} --wraps={program}"));
        let roots = build_wrapper_trie(entries);
        emit_fish_switch(&mut lines, &roots, 1, base_cmd, "  ");
        lines.push("end".into());
        lines.join("\n")
    }
}

/// Emit a `switch $argv[{argv_depth}]` block for the given set of trie nodes.
fn emit_fish_switch(
    lines: &mut Vec<String>,
    nodes: &BTreeMap<String, WrapperNode>,
    argv_depth: usize,
    base_cmd: &str,
    indent: &str,
) {
    lines.push(format!("{indent}switch $argv[{argv_depth}]"));
    for (short, node) in nodes {
        lines.push(format!("{indent}  case '{short}'"));
        emit_fish_node_body(lines, node, argv_depth, base_cmd, &format!("{indent}    "));
    }
    lines.push(format!("{indent}  case '*'"));
    lines.push(format!("{indent}    {base_cmd} $argv"));
    lines.push(format!("{indent}end"));
}

/// Emit the body of a matched case at `argv_depth`.
/// If the node has children, a nested switch is emitted; otherwise the leaf expansion.
fn emit_fish_node_body(
    lines: &mut Vec<String>,
    node: &WrapperNode,
    argv_depth: usize,
    base_cmd: &str,
    indent: &str,
) {
    let next_depth = argv_depth + 1;
    if node.children.is_empty() {
        let expansion = node.leaf_longs.as_deref().unwrap_or_default().join(" ");
        if has_template_args(&expansion) {
            lines.push(format!(
                "{indent}{base_cmd} {}",
                substitute_offset(&expansion, argv_depth)
            ));
        } else {
            lines.push(format!(
                "{indent}{base_cmd} {expansion} $argv[{next_depth}..]"
            ));
        }
    } else {
        lines.push(format!("{indent}switch $argv[{next_depth}]"));
        for (short, child) in &node.children {
            lines.push(format!("{indent}  case '{short}'"));
            emit_fish_node_body(lines, child, next_depth, base_cmd, &format!("{indent}    "));
        }
        lines.push(format!("{indent}  case '*'"));
        if let Some(longs) = &node.leaf_longs {
            let expansion = longs.join(" ");
            if has_template_args(&expansion) {
                lines.push(format!(
                    "{indent}    {base_cmd} {}",
                    substitute_offset(&expansion, argv_depth)
                ));
            } else {
                lines.push(format!(
                    "{indent}    {base_cmd} {expansion} $argv[{next_depth}..]"
                ));
            }
        } else {
            lines.push(format!("{indent}    {base_cmd} $argv"));
        }
        lines.push(format!("{indent}end"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::test_helpers::{raw, simple};
    use crate::subcommand::SubcommandEntry;

    #[test]
    fn test_fish_simple_alias() {
        assert_eq!(
            Fish.alias(&simple("h", "'echo hello'")),
            "alias h 'echo hello'"
        );
        assert_eq!(
            Fish.alias(&simple("h", "echo hello")),
            "alias h \"echo hello\""
        );
    }

    #[test]
    fn test_fish_parameterized() {
        assert_eq!(
            Fish.alias(&simple("cmf", "cm feat: {{@}}")),
            "function cmf\n    cm feat: $argv\nend"
        );
        assert_eq!(
            Fish.alias(&simple("x", "echo {{1}} and {{2}}")),
            "function x\n    echo $argv[1] and $argv[2]\nend"
        );
    }

    #[test]
    fn test_fish_raw_skips_templates() {
        assert_eq!(
            Fish.alias(&raw("my-awk", "awk '{print {{1}}}'")),
            "alias my-awk \"awk '{print {{1}}}'\""
        );
    }

    #[test]
    fn test_fish_unalias() {
        assert_eq!(Fish.unalias("h"), "functions -e h");
    }

    #[test]
    fn test_fish_env() {
        assert_eq!(Fish.set_env("FOO", "bar"), "set -gx FOO \"bar\"");
        assert_eq!(Fish.unset_env("FOO"), "set -e FOO");
    }

    #[test]
    fn test_fish_subcommand_wrapper_single() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["ab".into()],
            long_subcommands: vec!["abandon".into()],
        }];
        let output = Fish.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("function jj --wraps=jj"));
        assert!(output.contains("case 'ab'"));
        assert!(output.contains("command jj abandon $argv[2..]"));
        assert!(output.contains("case '*'"));
        assert!(output.contains("command jj $argv"));
    }

    #[test]
    fn test_fish_subcommand_wrapper_parameterized_single() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["ab".into()],
            long_subcommands: vec!["abandon --rev {{1}}".into()],
        }];
        let output = Fish.subcommand_wrapper("jj", "command jj", &entries);
        // offset=1: {{1}} → $argv[2]
        assert!(output.contains("command jj abandon --rev $argv[2]"));
        assert!(!output.contains("$argv[2..]")); // no trailing spread when templates used
    }

    #[test]
    fn test_fish_subcommand_wrapper_single_quoted_template() {
        // Expansion contains single-quoted revset expressions with a template inside.
        // The generated code must NOT leave $argv[N] inside single quotes (Fish won't expand it).
        // Correct output: double-quote the affected token so Fish expands the variable.
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["echo".into()],
            long_subcommands: vec!["rebase -s 'mega()' -d 'toggle({{1}})'".into()],
        }];
        let output = Fish.subcommand_wrapper("jj", "command jj", &entries);
        // The expansion should break the single-quoted token at the template boundary
        // so Fish can expand $argv[2] between the two single-quoted fragments.
        assert!(
            output.contains("'toggle('$argv[2]')'"),
            "expected broken single-quote expansion: {output}"
        );
        // The variable must NOT be trapped entirely inside a single-quoted string.
        assert!(
            !output.contains("'toggle($argv[2])'"),
            "broken output found: {output}"
        );
    }

    #[test]
    fn test_fish_subcommand_wrapper_parameterized_all_args() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["l".into()],
            long_subcommands: vec!["log --template {{@}}".into()],
        }];
        let output = Fish.subcommand_wrapper("jj", "command jj", &entries);
        // offset=1: {{@}} → $argv[2..]
        assert!(output.contains("command jj log --template $argv[2..]"));
    }

    #[test]
    fn test_fish_subcommand_wrapper_parameterized_multi_level() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["b".into(), "l".into()],
            long_subcommands: vec!["branch".into(), "list --limit {{1}}".into()],
        }];
        let output = Fish.subcommand_wrapper("jj", "command jj", &entries);
        // offset=2: {{1}} → $argv[3]
        assert!(output.contains("command jj branch list --limit $argv[3]"));
    }

    #[test]
    fn test_fish_subcommand_wrapper_multi_level() {
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
        let output = Fish.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("case 'ab'"));
        assert!(output.contains("case 'b'"));
        assert!(output.contains("switch $argv[2]"));
        assert!(output.contains("case 'l'"));
        assert!(output.contains("command jj branch list $argv[3..]"));
    }

    #[test]
    fn test_fish_subcommand_wrapper_depth3() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["b".into(), "l".into(), "x".into()],
            long_subcommands: vec!["branch".into(), "list".into(), "extra".into()],
        }];
        let output = Fish.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("switch $argv[2]"), "depth-2 switch missing");
        assert!(output.contains("switch $argv[3]"), "depth-3 switch missing");
        assert!(output.contains("case 'x'"), "depth-3 case missing");
        assert!(
            output.contains("command jj branch list extra $argv[4..]"),
            "depth-3 expansion wrong"
        );
    }
}
