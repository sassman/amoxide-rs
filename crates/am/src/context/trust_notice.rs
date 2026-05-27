use indoc::formatdoc;

/// A discovered `.aliases` file that is not currently loaded due to its
/// trust state. Carried into the snapshot so the agent can prompt the user
/// to review the file instead of silently working from an incomplete picture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectTrustNotice {
    pub path: std::path::PathBuf,
    pub reason: ProjectTrustReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectTrustReason {
    /// File exists but has never been reviewed by the user.
    Unknown,
    /// User previously declined to trust this file.
    Untrusted,
    /// File contents changed since the user last trusted it.
    Tampered,
}

impl ProjectTrustReason {
    fn label(self) -> &'static str {
        match self {
            Self::Unknown => "not yet reviewed",
            Self::Untrusted => "previously untrusted",
            Self::Tampered => "modified since last trusted",
        }
    }
}

/// Render the `## Project aliases` section. Empty string when there is no
/// such file in scope or the file is trusted (its aliases are already in
/// the main table).
///
/// The body tells the *agent* to ask the *user* once at session start to
/// run `am trust`. `am trust` itself is the interactive review tool: it
/// shows the file contents and asks for confirmation. The agent's only job
/// here is to surface the choice to the user — once — before acting on the
/// alias table, which is partial while the project file is untrusted.
///
/// `alias_names` is every name currently in the effective alias table,
/// listed verbatim in the notice so the agent sees concretely which entries
/// the untrusted project file could override.
pub fn render_project_trust_notice(
    notice: Option<&ProjectTrustNotice>,
    alias_names: &[&str],
) -> String {
    let Some(notice) = notice else {
        return String::new();
    };
    let name_list = if alias_names.is_empty() {
        String::from("(the table is currently empty)")
    } else {
        alias_names
            .iter()
            .map(|n| format!("`{n}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    formatdoc! {r#"
        ## Project aliases (not loaded)

        > [!WARNING]
        > Your view of aliases here is incomplete. The table below is missing the project file's entries, any of which can shadow what you see.

        A project `.aliases` file at `{path}` is in scope but not loaded ({reason}). Its entries shadow every alias of the same name at priority 0 — above all profiles and globals — so any of these table names, plus any the project file may add, could resolve to a different command here:

        {name_list}

        **At session start, ask the user once** — unless they have already declined trust in this session, in which case skip this whole section:

        > "A project `.aliases` file here isn't trusted yet. Run `am trust` in your shell — it'll show you the contents and ask for your confirmation."

        Then wait for their answer before acting on any alias.

        **Hand-off rules**

        - Don't read `.aliases` yourself — `am trust` is the review surface; reading it shortcuts the interactive flow.
        - Don't run `am trust` in a subshell — it's interactive and hangs on the prompt. Run it in the user's terminal.
    "#,
        path = notice.path.display(),
        reason = notice.reason.label(),
        name_list = name_list,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_notice_when_none() {
        let out = render_project_trust_notice(None, &[]);
        assert!(out.is_empty(), "expected empty: {out}");
    }

    #[test]
    fn notice_names_path_reason_session_start_ask_and_alias_list() {
        let notice = ProjectTrustNotice {
            path: "/x/.aliases".into(),
            reason: ProjectTrustReason::Untrusted,
        };
        let out = render_project_trust_notice(Some(&notice), &["i", "t", "git cm"]);
        assert!(
            out.contains("## Project aliases (not loaded)"),
            "header: {out}"
        );
        assert!(
            out.contains("> [!WARNING]"),
            "leads with a markdown-native warning admonition: {out}"
        );
        assert!(
            out.contains("Your view of aliases here is incomplete"),
            "warning body names the failure mode: {out}"
        );
        assert!(out.contains("/x/.aliases"), "path: {out}");
        assert!(out.contains("previously untrusted"), "reason label: {out}");
        assert!(
            out.contains("At session start, ask the user once"),
            "asks once at session start (not action-gated): {out}"
        );
        assert!(
            out.contains("already declined trust in this session"),
            "respects a prior decline inline with the ask: {out}"
        );
        assert!(
            out.contains("Run `am trust` in your shell"),
            "natural prompt hands off to user terminal: {out}"
        );
        assert!(
            out.contains("`i`") && out.contains("`t`") && out.contains("`git cm`"),
            "every passed alias name appears verbatim: {out}"
        );
        assert!(
            out.contains("**Hand-off rules**"),
            "groups the don'ts under a labelled bullet block: {out}"
        );
        assert!(
            out.contains("Don't read `.aliases` yourself"),
            "forbids reading the file ourselves: {out}"
        );
        assert!(
            out.contains("Don't run `am trust` in a subshell"),
            "forbids subshell execution: {out}"
        );
    }

    #[test]
    fn notice_handles_empty_alias_list_gracefully() {
        let notice = ProjectTrustNotice {
            path: "/x/.aliases".into(),
            reason: ProjectTrustReason::Unknown,
        };
        let out = render_project_trust_notice(Some(&notice), &[]);
        assert!(
            out.contains("the table is currently empty"),
            "fallback phrasing when no aliases in scope: {out}"
        );
    }

    #[test]
    fn notice_reason_labels_per_trust_state() {
        for (reason, label) in [
            (ProjectTrustReason::Unknown, "not yet reviewed"),
            (ProjectTrustReason::Untrusted, "previously untrusted"),
            (ProjectTrustReason::Tampered, "modified since last trusted"),
        ] {
            let notice = ProjectTrustNotice {
                path: "/p/.aliases".into(),
                reason,
            };
            let out = render_project_trust_notice(Some(&notice), &[]);
            assert!(
                out.contains(label),
                "label '{label}' missing for {reason:?}: {out}"
            );
        }
    }
}
