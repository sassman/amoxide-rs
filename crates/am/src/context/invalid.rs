use crate::precedence::Diagnostic;

/// Render the verbose-only `## Invalid` section. Empty if no diagnostics.
///
/// Drives off `ResolveOutcome.diagnostics` (the rendered, ready-to-print form)
/// rather than re-formatting `InvalidEntry` independently. One bullet per
/// diagnostic.
pub fn render_invalid(diagnostics: &[Diagnostic]) -> String {
    if diagnostics.is_empty() {
        return String::new();
    }
    let mut out = String::from("## Invalid\n");
    for d in diagnostics {
        out.push_str(&format!("- {}\n", d.message));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_section_empty_when_no_diagnostics() {
        let out = render_invalid(&[]);
        assert!(out.is_empty(), "no diagnostics means no section: {out}");
    }

    #[test]
    fn invalid_section_renders_diagnostic_messages() {
        let diags = vec![Diagnostic {
            message: "alias `cc` in profile:compile_help references undefined vars: opt-flags"
                .into(),
        }];
        let out = render_invalid(&diags);
        assert!(out.contains("## Invalid"), "section header: {out}");
        assert!(
            out.contains("alias `cc` in profile:compile_help"),
            "message text: {out}"
        );
        assert!(
            out.contains("undefined vars: opt-flags"),
            "message text: {out}"
        );
    }

    #[test]
    fn invalid_section_emits_one_bullet_per_diagnostic() {
        let diags = vec![
            Diagnostic {
                message: "first message".into(),
            },
            Diagnostic {
                message: "second message".into(),
            },
        ];
        let out = render_invalid(&diags);
        // Count bullet lines (lines starting with "- ")
        let bullets = out.lines().filter(|l| l.starts_with("- ")).count();
        assert_eq!(bullets, 2, "expected 2 bullets, got: {out}");
        assert!(out.contains("- first message"));
        assert!(out.contains("- second message"));
    }
}
