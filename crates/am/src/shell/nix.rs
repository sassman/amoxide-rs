use std::fmt::Debug;

use super::{has_template_args, substitute_nix, Shell};
use crate::alias::AliasEntry;
use crate::subcommand::SubcommandEntry;

#[derive(Debug, Default)]
pub struct NixShell;

impl Shell for NixShell {
    fn unalias(&self, alias_name: &str) -> String {
        format!("unset -f {alias_name}")
    }

    fn alias(&self, entry: &AliasEntry) -> String {
        if !entry.raw && has_template_args(entry.command) {
            let body = substitute_nix(entry.command);
            format!("{}() {{ {}; }}", entry.name, body)
        } else {
            format!("{}() {{ {} \"$@\"; }}", entry.name, entry.command)
        }
    }

    fn set_env(&self, var_name: &str, value: &str) -> String {
        format!("export {var_name}=\"{value}\"")
    }

    fn unset_env(&self, var_name: &str) -> String {
        format!("unset {var_name}")
    }

    fn echo(&self, message: &str) -> String {
        format!("printf '%s\\n' '{}'", message.replace('\'', "'\\''"))
    }

    fn subcommand_wrapper(
        &self,
        program: &str,
        base_cmd: &str,
        entries: &[SubcommandEntry],
    ) -> String {
        let mut lines = Vec::new();
        lines.push(format!("{program}() {{"));
        lines.push("  case \"$1\" in".into());

        // Group entries by first short subcommand
        let mut groups: std::collections::BTreeMap<String, Vec<&SubcommandEntry>> =
            std::collections::BTreeMap::new();
        for entry in entries {
            groups
                .entry(entry.short_subcommands[0].clone())
                .or_default()
                .push(entry);
        }

        for (first_short, group) in &groups {
            // Separate single-level (depth=1) from deeper entries
            let single: Vec<&&SubcommandEntry> = group
                .iter()
                .filter(|e| e.short_subcommands.len() == 1)
                .collect();
            let deeper: Vec<&&SubcommandEntry> = group
                .iter()
                .filter(|e| e.short_subcommands.len() > 1)
                .collect();

            if deeper.is_empty() {
                // Simple single-level case
                let entry = single[0];
                let long = &entry.long_subcommands[0];
                if has_template_args(long) {
                    lines.push(format!(
                        "    {first_short}) shift; {base_cmd} {} ;;",
                        substitute_nix(long)
                    ));
                } else {
                    lines.push(format!(
                        "    {first_short}) shift; {base_cmd} {long} \"$@\" ;;"
                    ));
                }
            } else {
                // Nested case — first_short opens a sub-case
                lines.push(format!("    {first_short})"));
                lines.push("      case \"$2\" in".into());
                for entry in &deeper {
                    let second_short = &entry.short_subcommands[1];
                    let expansion = entry
                        .long_subcommands
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    if has_template_args(&expansion) {
                        lines.push(format!(
                            "        {second_short}) shift 2; {base_cmd} {} ;;",
                            substitute_nix(&expansion)
                        ));
                    } else {
                        lines.push(format!(
                            "        {second_short}) shift 2; {base_cmd} {expansion} \"$@\" ;;"
                        ));
                    }
                }
                // If there's also a single-level entry for this first_short, handle the default
                if let Some(entry) = single.first() {
                    let long = &entry.long_subcommands[0];
                    if has_template_args(long) {
                        lines.push(format!(
                            "        *) shift; {base_cmd} {} ;;",
                            substitute_nix(long)
                        ));
                    } else {
                        lines.push(format!(
                            "        *) shift; {base_cmd} {long} \"$@\" ;;"
                        ));
                    }
                } else {
                    lines.push(format!("        *) {base_cmd} \"$@\" ;;"));
                }
                lines.push("      esac".into());
                lines.push("      ;;".into());
            }
        }

        lines.push(format!("    *) {base_cmd} \"$@\" ;;"));
        lines.push("  esac".into());
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
    fn test_nix_simple() {
        assert_eq!(
            NixShell.alias(&simple("h", "echo hello")),
            "h() { echo hello \"$@\"; }"
        );
    }

    #[test]
    fn test_nix_parameterized() {
        assert_eq!(
            NixShell.alias(&simple("cmf", "cm feat: {{@}}")),
            "cmf() { cm feat: \"$@\"; }"
        );
        assert_eq!(
            NixShell.alias(&simple("x", "echo {{1}} and {{2}}")),
            "x() { echo \"$1\" and \"$2\"; }"
        );
    }

    #[test]
    fn test_nix_raw_skips_templates() {
        assert_eq!(
            NixShell.alias(&raw("my-awk", "awk '{print {{1}}}'")),
            "my-awk() { awk '{print {{1}}}' \"$@\"; }"
        );
    }

    #[test]
    fn test_nix_unalias() {
        assert_eq!(NixShell.unalias("h"), "unset -f h");
    }

    #[test]
    fn test_nix_env() {
        assert_eq!(NixShell.set_env("FOO", "bar"), "export FOO=\"bar\"");
        assert_eq!(NixShell.unset_env("FOO"), "unset FOO");
    }

    #[test]
    fn test_nix_subcommand_wrapper_single() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["ab".into()],
            long_subcommands: vec!["abandon".into()],
        }];
        let output = NixShell.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("jj() {"));
        assert!(output.contains("ab) shift; command jj abandon \"$@\" ;;"));
        assert!(output.contains("*) command jj \"$@\" ;;"));
    }

    #[test]
    fn test_nix_subcommand_wrapper_with_alias_base() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["ab".into()],
            long_subcommands: vec!["abandon".into()],
        }];
        let output = NixShell.subcommand_wrapper("jj", "just-a-joke", &entries);
        assert!(output.contains("ab) shift; just-a-joke abandon \"$@\" ;;"));
        assert!(output.contains("*) just-a-joke \"$@\" ;;"));
    }

    #[test]
    fn test_nix_subcommand_wrapper_parameterized_single() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["ab".into()],
            long_subcommands: vec!["abandon --rev {{1}}".into()],
        }];
        let output = NixShell.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("ab) shift; command jj abandon --rev \"$1\" ;;"));
        // No trailing "$@" when templates are used
        assert!(!output.contains("\"$1\" \"$@\""));
    }

    #[test]
    fn test_nix_subcommand_wrapper_parameterized_all_args() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["l".into()],
            long_subcommands: vec!["log --template {{@}}".into()],
        }];
        let output = NixShell.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("l) shift; command jj log --template \"$@\" ;;"));
    }

    #[test]
    fn test_nix_subcommand_wrapper_parameterized_multi_level() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["b".into(), "l".into()],
            long_subcommands: vec!["branch".into(), "list --limit {{1}}".into()],
        }];
        let output = NixShell.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("l) shift 2; command jj branch list --limit \"$1\" ;;"));
    }

    #[test]
    fn test_nix_subcommand_wrapper_complex_expansion() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["anon".into()],
            long_subcommands: vec!["log -r 'anon()'".into()],
        }];
        let output = NixShell.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("anon) shift; command jj log -r 'anon()' \"$@\" ;;"));
    }

    #[test]
    fn test_nix_subcommand_wrapper_multi_level() {
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
        let output = NixShell.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("ab) shift; command jj abandon \"$@\" ;;"));
        assert!(output.contains("b)"));
        assert!(output.contains("case \"$2\" in"));
        assert!(output.contains("l) shift 2; command jj branch list \"$@\" ;;"));
    }
}
