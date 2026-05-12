//! `am context` snapshot renderer.
//!
//! Sibling presentation module to `display.rs`. Drives from
//! `Precedence::resolve()` outcome plus the original layer inputs.
//! Output is markdown intended for AI coding assistants.

use std::path::Path;

use crate::precedence::OriginScope;

/// Describes the active precedence chain for the preamble.
/// Order: highest precedence first.
#[derive(Debug, Clone)]
pub struct PrecedenceChain {
    pub layers: Vec<ChainLayer>,
}

#[derive(Debug, Clone)]
pub struct ChainLayer {
    pub scope: OriginScope,
    /// For profile layers: 1-based priority (higher = higher precedence above lower).
    /// For global/project: None.
    pub priority: Option<usize>,
}

/// Render the preamble. Leads with usage rules; format notes second.
pub fn render_preamble(cwd: &Path, chain: &PrecedenceChain) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# amoxide aliases (active set, cwd: {})\n",
        cwd.display()
    ));
    out.push_str("#\n");
    out.push_str("# ## How to use this snapshot\n");
    out.push_str("#\n");
    out.push_str(
        "# When the user mentions a name from the `Aliases` table below in any context —\n",
    );
    out.push_str(
        "# running a command, suggesting one, asking what it does — treat the `expands to`\n",
    );
    out.push_str("# value as the canonical form.\n");
    out.push_str("#\n");
    out.push_str(
        "# 1. Recognise aliases by name match. If the user's input contains a token that\n",
    );
    out.push_str(
        "#    matches a `name` from the table — including multi-word names with a space,\n",
    );
    out.push_str("#    like `git pl` — it is an alias. Expand it before running.\n");
    out.push_str("#\n");
    out.push_str("# 2. Subcommand aliases are deceptive. A name like `git pl` looks like a real\n");
    out.push_str("#    git subcommand but is not. Running `git pl` verbatim in a subshell fails\n");
    out.push_str("#    with `git: 'pl' is not a git command`. Always run the value from\n");
    out.push_str("#    `expands to` (`git pull --rebase`), never the alias text.\n");
    out.push_str("#\n");
    out.push_str(
        "# 3. Recover from `command not found` failures. If a shell command fails because\n",
    );
    out.push_str("#    the name is unknown, check this table — the user's shell sees the alias\n");
    out.push_str("#    but your subshell does not.\n");
    out.push_str("#\n");
    out.push_str("# 4. In chat, the user's vocabulary is fine. When suggesting commands in\n");
    out.push_str(
        "#    conversation, the short form (`git cm \"msg\"`) matches the user's mental\n",
    );
    out.push_str("#    model. When *running* it in a subshell, use the canonical form.\n");
    out.push_str("#\n");
    out.push_str("# ## Format\n");
    out.push_str("#\n");
    out.push_str(&format!(
        "# Precedence (highest first): {}\n",
        render_chain(chain)
    ));
    out.push_str("#\n");
    out.push_str("# Templates: {{N}} is a positional placeholder (1-indexed; `tag v1.0` expands {{1}} → `v1.0`).\n");
    out.push_str("# Variables: {{name}} tokens are already substituted in the table below.\n");
    out.push_str("#            The full variable table is in the `## Variables` section.\n");
    out
}

fn render_chain(chain: &PrecedenceChain) -> String {
    chain
        .layers
        .iter()
        .map(|l| match (&l.scope, l.priority) {
            (OriginScope::Project, _) => "project".to_string(),
            (OriginScope::Profile(name), Some(p)) => format!("profile({name}, prio {p})"),
            (OriginScope::Profile(name), None) => format!("profile({name})"),
            (OriginScope::Global, _) => "global".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" > ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn chain(layers: Vec<ChainLayer>) -> PrecedenceChain {
        PrecedenceChain { layers }
    }

    #[test]
    fn preamble_starts_with_cwd_header() {
        let cwd = PathBuf::from("/tmp/project");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let out = render_preamble(&cwd, &c);
        assert!(
            out.starts_with("# amoxide aliases (active set, cwd: /tmp/project)\n"),
            "got: {out}"
        );
    }

    #[test]
    fn preamble_contains_all_four_usage_rules() {
        let cwd = PathBuf::from("/x");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let out = render_preamble(&cwd, &c);
        assert!(out.contains("1. Recognise aliases by name match"), "rule 1");
        assert!(
            out.contains("2. Subcommand aliases are deceptive"),
            "rule 2"
        );
        assert!(
            out.contains("3. Recover from `command not found`"),
            "rule 3"
        );
        assert!(
            out.contains("4. In chat, the user's vocabulary is fine"),
            "rule 4"
        );
    }

    #[test]
    fn preamble_renders_precedence_chain_in_order() {
        let cwd = PathBuf::from("/x");
        let c = chain(vec![
            ChainLayer {
                scope: OriginScope::Project,
                priority: None,
            },
            ChainLayer {
                scope: OriginScope::Profile("git".into()),
                priority: Some(2),
            },
            ChainLayer {
                scope: OriginScope::Profile("rust".into()),
                priority: Some(1),
            },
            ChainLayer {
                scope: OriginScope::Global,
                priority: None,
            },
        ]);
        let out = render_preamble(&cwd, &c);
        assert!(
            out.contains("project > profile(git, prio 2) > profile(rust, prio 1) > global"),
            "got: {out}"
        );
    }
}
