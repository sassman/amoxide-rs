use std::collections::BTreeMap;
use std::fmt::Debug;

use super::{build_wrapper_trie, has_template_args, substitute_nix, Shell, WrapperNode};
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
        let roots = build_wrapper_trie(entries);
        emit_nix_switch(&mut lines, &roots, 1, base_cmd, "  ", None);
        lines.push("}".into());
        lines.join("\n")
    }
}

/// Emit a `case "$N" in` block for the given trie nodes.
/// `fallback` carries an optional (long_subcommands, shift_depth) for the `*)` default,
/// used when the parent node is itself a leaf (e.g. both `jj:b` and `jj:b:l` exist).
fn emit_nix_switch(
    lines: &mut Vec<String>,
    nodes: &BTreeMap<String, WrapperNode>,
    argv_depth: usize,
    base_cmd: &str,
    indent: &str,
    fallback: Option<(&[String], usize)>,
) {
    lines.push(format!("{indent}case \"${argv_depth}\" in"));
    for (short, node) in nodes {
        let shift_str = if argv_depth == 1 {
            "shift".to_string()
        } else {
            format!("shift {argv_depth}")
        };
        if node.children.is_empty() {
            let expansion = node.leaf_longs.as_deref().unwrap_or_default().join(" ");
            if has_template_args(&expansion) {
                lines.push(format!(
                    "{indent}  {short}) {shift_str}; {base_cmd} {} ;;",
                    substitute_nix(&expansion)
                ));
            } else {
                lines.push(format!(
                    "{indent}  {short}) {shift_str}; {base_cmd} {expansion} \"$@\" ;;"
                ));
            }
        } else {
            lines.push(format!("{indent}  {short})"));
            let child_fallback = node.leaf_longs.as_deref().map(|longs| (longs, argv_depth));
            emit_nix_switch(
                lines,
                &node.children,
                argv_depth + 1,
                base_cmd,
                &format!("{indent}    "),
                child_fallback,
            );
            lines.push(format!("{indent}    ;;"));
        }
    }
    match fallback {
        Some((longs, shift_depth)) => {
            let expansion = longs.join(" ");
            let shift_str = if shift_depth == 1 {
                "shift".to_string()
            } else {
                format!("shift {shift_depth}")
            };
            if has_template_args(&expansion) {
                lines.push(format!(
                    "{indent}  *) {shift_str}; {base_cmd} {} ;;",
                    substitute_nix(&expansion)
                ));
            } else {
                lines.push(format!(
                    "{indent}  *) {shift_str}; {base_cmd} {expansion} \"$@\" ;;"
                ));
            }
        }
        None => lines.push(format!("{indent}  *) {base_cmd} \"$@\" ;;")),
    }
    lines.push(format!("{indent}esac"));
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

    #[test]
    fn test_nix_subcommand_wrapper_depth3() {
        let entries = vec![SubcommandEntry {
            program: "jj".into(),
            short_subcommands: vec!["b".into(), "l".into(), "x".into()],
            long_subcommands: vec!["branch".into(), "list".into(), "extra".into()],
        }];
        let output = NixShell.subcommand_wrapper("jj", "command jj", &entries);
        assert!(output.contains("case \"$2\" in"), "depth-2 switch missing");
        assert!(output.contains("case \"$3\" in"), "depth-3 switch missing");
        assert!(
            output.contains("x) shift 3; command jj branch list extra \"$@\" ;;"),
            "depth-3 expansion wrong"
        );
    }
}
