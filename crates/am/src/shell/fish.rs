use std::fmt::Debug;

use super::{has_template_args, quote_cmd, substitute_fish, Shell, TEMPLATE_RE};
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
                        // Convert this single-quoted token to a double-quoted token so
                        // Fish expands `$argv[N]` without any single-quote interference.
                        result.push_str(&rest[..sq]);
                        result.push('"');
                        let inner = sq_content.replace('"', "\\\"");
                        let subbed = TEMPLATE_RE
                            .replace_all(&inner, |caps: &regex::Captures| make_sub(&caps[1]));
                        result.push_str(&subbed);
                        result.push('"');
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
        lines.push("  switch $argv[1]".into());

        let mut groups: std::collections::BTreeMap<String, Vec<&SubcommandEntry>> =
            std::collections::BTreeMap::new();
        for entry in entries {
            groups
                .entry(entry.short_subcommands[0].clone())
                .or_default()
                .push(entry);
        }

        for (first_short, group) in &groups {
            let single: Vec<&&SubcommandEntry> = group
                .iter()
                .filter(|e| e.short_subcommands.len() == 1)
                .collect();
            let deeper: Vec<&&SubcommandEntry> = group
                .iter()
                .filter(|e| e.short_subcommands.len() > 1)
                .collect();

            lines.push(format!("    case {first_short}"));
            if deeper.is_empty() {
                let entry = single[0];
                let long = &entry.long_subcommands[0];
                if has_template_args(long) {
                    lines.push(format!("      {base_cmd} {}", substitute_offset(long, 1)));
                } else {
                    lines.push(format!("      {base_cmd} {long} $argv[2..]"));
                }
            } else {
                lines.push("      switch $argv[2]".into());
                for entry in &deeper {
                    let second_short = &entry.short_subcommands[1];
                    let expansion = entry
                        .long_subcommands
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    lines.push(format!("        case {second_short}"));
                    if has_template_args(&expansion) {
                        lines.push(format!(
                            "          {base_cmd} {}",
                            substitute_offset(&expansion, 2)
                        ));
                    } else {
                        lines.push(format!("          {base_cmd} {expansion} $argv[3..]"));
                    }
                }
                if let Some(entry) = single.first() {
                    let long = &entry.long_subcommands[0];
                    lines.push("        case '*'".into());
                    if has_template_args(long) {
                        lines.push(format!("          {base_cmd} {}", substitute_offset(long, 1)));
                    } else {
                        lines.push(format!("          {base_cmd} {long} $argv[2..]"));
                    }
                } else {
                    lines.push("        case '*'".into());
                    lines.push(format!("          {base_cmd} $argv"));
                }
                lines.push("      end".into());
            }
        }

        lines.push("    case '*'".into());
        lines.push(format!("      {base_cmd} $argv"));
        lines.push("  end".into());
        lines.push("end".into());
        lines.join("\n")
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
        assert!(output.contains("case ab"));
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
        // $argv[2] must NOT be trapped inside single quotes
        assert!(!output.contains("'$argv[2]'"), "variable must not be single-quoted: {output}");
        assert!(!output.contains("'toggle($argv[2])'"), "broken output found: {output}");
        // The expansion should produce a double-quoted token for the affected revset
        assert!(output.contains("\"toggle($argv[2])\""), "expected double-quoted expansion: {output}");
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
        assert!(output.contains("case ab"));
        assert!(output.contains("case b"));
        assert!(output.contains("switch $argv[2]"));
        assert!(output.contains("case l"));
        assert!(output.contains("command jj branch list $argv[3..]"));
    }
}
