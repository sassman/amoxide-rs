use std::collections::BTreeMap;
use std::fmt::Debug;

use super::{
    build_wrapper_trie, has_template_args, quote_cmd, substitute_fish, ShellAdapter, WrapperNode,
    TEMPLATE_RE,
};
use crate::alias::AliasEntry;
use crate::config::FishConfig;
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
pub struct Fish {
    pub use_abbr: bool,
}

impl Fish {
    pub fn from_config(config: Option<&FishConfig>) -> Self {
        Fish {
            use_abbr: config.map(|c| c.use_abbr).unwrap_or(false),
        }
    }
}

impl ShellAdapter for Fish {
    fn unalias(&self, alias_name: &str) -> String {
        if self.use_abbr {
            format!("abbr --erase {alias_name}")
        } else {
            format!("functions -e {alias_name}")
        }
    }

    fn force_unalias(&self, alias_name: &str) -> String {
        format!(
            "functions -e {alias_name}\nabbr --query {alias_name}; and abbr --erase {alias_name}"
        )
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        // Emit a plain `function` instead of going through fish's `alias`
        // builtin. Fish's `alias` records `--wraps` via the completion
        // system, and that entry survives `functions -e`; redefining the
        // same alias stacks `--wraps=` flags. Using `function` directly
        // with no `--wraps` avoids the issue entirely (at the cost of
        // completion inheritance, which was unreliable anyway for aliases
        // whose command is a pipe/chain). `functions -e` still prefixes to
        // keep the redefinition clean, and `complete -e -c NAME` clears any
        // `--wraps` left over from prior amoxide versions that used `alias`.
        let prelude = format!(
            "functions -e {name}\ncomplete -e -c {name}",
            name = entry.name
        );
        if !entry.raw && has_template_args(entry.command) {
            let body = substitute_fish(entry.command);
            format!(
                "{prelude}\nfunction {name}\n    {body}\nend",
                name = entry.name
            )
        } else if self.use_abbr {
            format!("abbr --add {} {}", entry.name, quote_cmd(entry.command))
        } else {
            format!(
                "{prelude}\nfunction {name}\n    {cmd} $argv\nend",
                name = entry.name,
                cmd = entry.command,
            )
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
            Fish::default().alias(&simple("h", "'echo hello'")),
            "functions -e h\ncomplete -e -c h\nfunction h\n    'echo hello' $argv\nend"
        );
        assert_eq!(
            Fish::default().alias(&simple("h", "echo hello")),
            "functions -e h\ncomplete -e -c h\nfunction h\n    echo hello $argv\nend"
        );
    }

    #[test]
    fn test_fish_parameterized() {
        assert_eq!(
            Fish::default().alias(&simple("cmf", "cm feat: {{@}}")),
            "functions -e cmf\ncomplete -e -c cmf\nfunction cmf\n    cm feat: $argv\nend"
        );
        assert_eq!(
            Fish::default().alias(&simple("x", "echo {{1}} and {{2}}")),
            "functions -e x\ncomplete -e -c x\nfunction x\n    echo $argv[1] and $argv[2]\nend"
        );
    }

    #[test]
    fn test_fish_raw_skips_templates() {
        assert_eq!(
            Fish::default().alias(&raw("my-awk", "awk '{print {{1}}}'")),
            "functions -e my-awk\ncomplete -e -c my-awk\nfunction my-awk\n    awk '{print {{1}}}' $argv\nend"
        );
    }

    #[test]
    fn test_fish_unalias() {
        assert_eq!(Fish::default().unalias("h"), "functions -e h");
    }

    #[test]
    fn test_fish_env() {
        assert_eq!(Fish::default().set_env("FOO", "bar"), "set -gx FOO \"bar\"");
        assert_eq!(Fish::default().unset_env("FOO"), "set -e FOO");
    }

    #[test]
    fn test_fish_subcommand_wrapper_single() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["ab".into()],
            long_subcommands: vec!["abandon".into()],
        }];
        let output = Fish::default().subcommand_wrapper("jj", "command jj", &entries);
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
        let output = Fish::default().subcommand_wrapper("jj", "command jj", &entries);
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
        let output = Fish::default().subcommand_wrapper("jj", "command jj", &entries);
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
        let output = Fish::default().subcommand_wrapper("jj", "command jj", &entries);
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
        let output = Fish::default().subcommand_wrapper("jj", "command jj", &entries);
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
        let output = Fish::default().subcommand_wrapper("jj", "command jj", &entries);
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
        let output = Fish::default().subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("switch $argv[2]"), "depth-2 switch missing");
        assert!(output.contains("switch $argv[3]"), "depth-3 switch missing");
        assert!(output.contains("case 'x'"), "depth-3 case missing");
        assert!(
            output.contains("command jj branch list extra $argv[4..]"),
            "depth-3 expansion wrong"
        );
    }

    #[test]
    fn test_fish_abbr_simple_alias() {
        let fish = Fish { use_abbr: true };
        assert_eq!(
            fish.alias(&simple("gs", "git status")),
            "abbr --add gs \"git status\""
        );
        assert_eq!(
            fish.alias(&simple("h", "'echo hello'")),
            "abbr --add h 'echo hello'"
        );
    }

    #[test]
    fn test_fish_abbr_parameterized_still_uses_function() {
        let fish = Fish { use_abbr: true };
        assert_eq!(
            fish.alias(&simple("cmf", "cm feat: {{@}}")),
            "functions -e cmf\ncomplete -e -c cmf\nfunction cmf\n    cm feat: $argv\nend"
        );
    }

    #[test]
    fn test_fish_abbr_unalias() {
        let fish = Fish { use_abbr: true };
        assert_eq!(fish.unalias("gs"), "abbr --erase gs");
    }

    #[test]
    fn test_fish_no_abbr_unalias() {
        let fish = Fish { use_abbr: false };
        assert_eq!(fish.unalias("gs"), "functions -e gs");
    }

    #[test]
    fn test_fish_force_unalias_no_abbr() {
        let fish = Fish { use_abbr: false };
        let out = fish.force_unalias("foo");
        assert!(
            out.contains("functions -e foo"),
            "missing functions -e: {out}"
        );
        assert!(
            out.contains("abbr --query foo; and abbr --erase foo"),
            "missing abbr guard: {out}"
        );
    }

    #[test]
    fn test_fish_force_unalias_with_abbr() {
        let fish = Fish { use_abbr: true };
        let out = fish.force_unalias("foo");
        assert!(
            out.contains("functions -e foo"),
            "missing functions -e: {out}"
        );
        assert!(
            out.contains("abbr --query foo; and abbr --erase foo"),
            "missing abbr guard: {out}"
        );
    }
}
