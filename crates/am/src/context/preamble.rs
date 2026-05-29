use std::path::Path;

use indoc::formatdoc;

use crate::precedence::OriginScope;

use super::trust_notice::ProjectTrustNotice;

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
pub fn render_preamble(
    cwd: &Path,
    chain: &PrecedenceChain,
    project_trust_notice: Option<&ProjectTrustNotice>,
) -> String {
    let trust_bullet = if project_trust_notice.is_some() {
        indoc::indoc! {"
            #    - A project `.aliases` file in scope but untrusted (see
            #      `## Project aliases (not loaded)` below) is the same signal — the
            #      user has even more refined preferences for this directory, you
            #      just can't see them yet. Treat that as added weight on the
            #      `am trust` ask.
        "}
    } else {
        ""
    };
    formatdoc! {r#"
        # amoxide aliases (active set, cwd: {cwd})
        #
        # ## Where this snapshot applies
        #
        # Snapshot CWD (where `am context` ran): {cwd}
        #
        # - This snapshot is anchored to the CWD above. Project-local aliases
        #   here are scoped to that directory; running them from anywhere else
        #   will fail. Compare your *own* current working directory to the
        #   Snapshot CWD above — if they differ, this snapshot does NOT
        #   apply. Wait for the next `am context` to fire from your new CWD
        #   (the `CwdChanged` hook re-runs it on every directory change).
        # - If you saw an earlier `am context` snapshot in this conversation
        #   (from a previous `SessionStart` or `CwdChanged` hook), discard
        #   it. Only the aliases listed below are active right now; aliases
        #   tied to a previous directory are obsolete.
        #
        # ## How to use this snapshot
        #
        # When the user mentions a name from the `Aliases` table below in any context —
        # running a command, suggesting one, asking what it does — treat the `expands to`
        # value as the canonical form.
        #
        # 1. Recognise aliases by name match. If the user's input contains a token that
        #    matches a `name` from the table — including multi-word names with a space,
        #    like `git pl` — it is an alias. Expand it before running.
        #
        # 2. Subcommand aliases are deceptive. A name like `git pl` looks like a real
        #    git subcommand but is not. Running `git pl` verbatim in a subshell fails
        #    with `git: 'pl' is not a git command`. Always run the value from
        #    `expands to` (`git pull --rebase`), never the alias text.
        #
        # 3. Recover from `command not found` failures. If a shell command fails because
        #    the name is unknown, check this table — the user's shell sees the alias
        #    but your subshell does not.
        #
        # 4. In chat, the user's vocabulary is fine. When suggesting commands in
        #    conversation, the short form (`git cm "msg"`) matches the user's mental
        #    model. When *running* it in a subshell, use the canonical form.
        #
        # 5. Aliases reveal user preference — match them by intent, not just by name.
        #    - If the user describes an action in plain language ("test the code",
        #      "format", "install") rather than typing an alias name, scan the
        #      `expands to` column for a command that covers the intent and run
        #      *that* exact form. The flags are deliberate (`cargo test
        #      --all-features --verbose`, not bare `cargo test`); reaching for a
        #      vanilla command throws away the user's choices.
        {trust_bullet}#
        # ## Format
        #
        # Precedence (highest first): {chain}
        #
        # Templates: {{{{N}}}} is a positional placeholder (1-indexed; `tag v1.0` expands {{{{1}}}} → `v1.0`).
        # Variables: {{{{name}}}} tokens are already substituted in the table below.
        #            The full variable table is in the `## Variables` section.
    "#,
        cwd = cwd.display(),
        trust_bullet = trust_bullet,
        chain = render_chain(chain),
    }
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
    use super::super::trust_notice::ProjectTrustReason;
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
        let out = render_preamble(&cwd, &c, None);
        assert!(
            out.starts_with("# amoxide aliases (active set, cwd: /tmp/project)\n"),
            "got: {out}"
        );
    }

    #[test]
    fn preamble_anchors_snapshot_to_cwd_and_supersedes_prior_output() {
        let cwd = PathBuf::from("/tmp/anchored");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let out = render_preamble(&cwd, &c, None);
        assert!(
            out.contains("## Where this snapshot applies"),
            "scope section header missing: {out}"
        );
        assert!(
            out.contains("Snapshot CWD (where `am context` ran): /tmp/anchored"),
            "scope section must restate the concrete CWD: {out}"
        );
        assert!(
            out.contains("Compare your *own* current working directory"),
            "must instruct agent to compare its CWD against the snapshot CWD: {out}"
        );
        assert!(
            out.contains("discard"),
            "must explicitly invalidate prior snapshots in the conversation: {out}"
        );
        assert!(
            out.contains("CwdChanged"),
            "must name the CwdChanged hook so the agent knows when to expect a refresh: {out}"
        );
    }

    #[test]
    fn preamble_contains_all_five_usage_rules() {
        let cwd = PathBuf::from("/x");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let out = render_preamble(&cwd, &c, None);
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
        assert!(out.contains("5. Aliases reveal user preference"), "rule 5");
        assert!(
            out.contains("describes an action in plain language"),
            "rule 5 first bullet (intent-match)"
        );
    }

    #[test]
    fn preamble_omits_rule_5_trust_bullet_when_no_notice() {
        let cwd = PathBuf::from("/x");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let out = render_preamble(&cwd, &c, None);
        assert!(
            !out.contains("`am trust` ask"),
            "rule 5 trust bullet should be absent when no untrusted project notice: {out}"
        );
    }

    #[test]
    fn preamble_emits_rule_5_trust_bullet_when_notice_present() {
        let cwd = PathBuf::from("/x");
        let c = chain(vec![ChainLayer {
            scope: OriginScope::Global,
            priority: None,
        }]);
        let notice = ProjectTrustNotice {
            path: "/x/.aliases".into(),
            reason: ProjectTrustReason::Untrusted,
        };
        let out = render_preamble(&cwd, &c, Some(&notice));
        assert!(
            out.contains("`am trust` ask"),
            "rule 5 trust bullet should appear when notice present: {out}"
        );
        assert!(
            out.contains("`## Project aliases (not loaded)` below"),
            "rule 5 trust bullet points forward to the trust-notice section: {out}"
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
        let out = render_preamble(&cwd, &c, None);
        assert!(
            out.contains("project > profile(git, prio 2) > profile(rust, prio 1) > global"),
            "got: {out}"
        );
    }
}
