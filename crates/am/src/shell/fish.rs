use std::fmt::Debug;

use super::{has_template_args, quote_cmd, substitute_fish, substitute_quote_aware, Shell};

/// Substitute `{{N}}` → `$argv[N+offset]` and `{{@}}` → `$argv[offset+1..]`.
/// Used in subcommand wrappers where fish doesn't shift args, so the subcommand
/// tokens at positions 1..=offset must be skipped.
fn substitute_offset(cmd: &str, offset: usize) -> String {
    substitute_quote_aware(cmd, |n| match n {
        "@" => format!("$argv[{}..]", offset + 1),
        n => {
            let idx: usize = n.parse::<usize>().unwrap() + offset;
            format!("$argv[{idx}]")
        }
    })
}
use crate::alias::AliasEntry;
use crate::subcommand::SubcommandEntry;

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
