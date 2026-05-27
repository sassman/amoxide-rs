//! `am context` snapshot renderer.
//!
//! Sibling presentation module to `display.rs`. Drives from
//! `Precedence::resolve()` outcome plus the original layer inputs.
//! Output is markdown intended for AI coding assistants.
//!
//! Submodules are per-section: each `render_*` function lives next to its
//! tests, and the top-level [`render`] orchestrator stitches them together.

mod aliases;
mod escape;
mod invalid;
mod preamble;
mod shadows;
mod trust_notice;
mod variables;

use std::path::Path;

use crate::alias::AliasSet;
use crate::precedence::{EntryKind, ProfileLayer, ResolveOutcome};
use crate::subcommand::SubcommandSet;
use crate::vars::VarSet;

pub use aliases::render_aliases_table;
pub use invalid::render_invalid;
pub use preamble::{render_preamble, ChainLayer, PrecedenceChain};
pub use shadows::{render_shadow_brief, render_shadow_verbose};
pub use trust_notice::{render_project_trust_notice, ProjectTrustNotice, ProjectTrustReason};
pub use variables::render_variables;

/// References to all the layered inputs that fed `Precedence::resolve()`.
/// Carried as borrows so the renderer can re-lookup origins and (later)
/// reconstruct shadow chains without re-resolving.
#[derive(Debug, Clone, Copy)]
pub struct LayerInputs<'a> {
    pub global_aliases: &'a AliasSet,
    pub global_subcommands: &'a SubcommandSet,
    pub global_vars: &'a VarSet,
    pub profile_layers: &'a [ProfileLayer],
    pub project_aliases: &'a AliasSet,
    pub project_subcommands: &'a SubcommandSet,
    pub project_vars: &'a VarSet,
}

#[derive(Debug, Clone, Copy)]
pub struct RenderOptions {
    pub verbose: bool,
}

/// Top-level orchestrator for the `am context` snapshot.
///
/// Wires every section into one markdown blob:
///   preamble → project-trust-notice → aliases → variables → shadowed → invalid
///
/// All sections except preamble and aliases are conditional. Invalid is
/// verbose-only. The project-trust-notice appears when an `.aliases` file
/// exists in scope but is not loaded, and is emitted *before* the alias
/// table so the agent reads the gating rule before treating the table as
/// authoritative.
pub fn render(
    cwd: &Path,
    chain: &PrecedenceChain,
    outcome: &ResolveOutcome,
    layers: &LayerInputs,
    project_trust_notice: Option<&ProjectTrustNotice>,
    opts: RenderOptions,
) -> String {
    let mut out = String::new();
    out.push_str(&render_preamble(cwd, chain, project_trust_notice));
    out.push('\n');

    // Effective set for `am context` is added ∪ unchanged
    let effective: Vec<_> = outcome
        .diff
        .added
        .iter()
        .chain(outcome.diff.unchanged.iter())
        .cloned()
        .collect();

    let mut alias_names: Vec<String> = Vec::new();
    for e in &effective {
        match &e.kind {
            EntryKind::Alias(_) => alias_names.push(e.name.clone()),
            EntryKind::SubcommandWrapper {
                program, entries, ..
            } => {
                for sub in entries {
                    alias_names.push(format!("{program} {}", sub.short_subcommands.join(" ")));
                }
            }
            EntryKind::SubcommandKey { .. } => {
                // Tracking-only, not user-facing.
            }
        }
    }
    alias_names.sort();
    let name_refs: Vec<&str> = alias_names.iter().map(String::as_str).collect();
    let trust_notice = render_project_trust_notice(project_trust_notice, &name_refs);
    if !trust_notice.is_empty() {
        out.push_str(&trust_notice);
        out.push('\n');
    }

    out.push_str(&render_aliases_table(&effective, layers));
    out.push('\n');

    let vars = render_variables(layers);
    if !vars.is_empty() {
        out.push_str(&vars);
        out.push('\n');
    }

    let shadow = if opts.verbose {
        render_shadow_verbose(layers)
    } else {
        render_shadow_brief(layers)
    };
    if !shadow.is_empty() {
        out.push_str(&shadow);
        out.push('\n');
    }

    if opts.verbose {
        let invalid = render_invalid(&outcome.diagnostics);
        if !invalid.is_empty() {
            out.push_str(&invalid);
            out.push('\n');
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alias::{AliasName, AliasSet, TomlAlias};
    use crate::precedence::{
        Diagnostic, EffectiveEntry, EntryKind, OriginScope, PrecedenceDiff, ResolveOutcome,
    };
    use crate::subcommand::SubcommandSet;
    use crate::vars::VarSet;
    use std::path::PathBuf;

    fn aset(pairs: &[(&str, &str)]) -> AliasSet {
        let mut s = AliasSet::default();
        for (n, c) in pairs {
            s.insert(AliasName::from(*n), TomlAlias::Command((*c).into()));
        }
        s
    }

    #[test]
    fn render_assembles_all_sections_in_order_for_brief() {
        let global = aset(&[("ll", "ls -lha")]);
        let project = aset(&[("f", "cargo fmt")]);
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &[],
            project_aliases: &project,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let chain = PrecedenceChain {
            layers: vec![
                ChainLayer {
                    scope: OriginScope::Project,
                    priority: None,
                },
                ChainLayer {
                    scope: OriginScope::Global,
                    priority: None,
                },
            ],
        };

        let effective = vec![
            EffectiveEntry {
                name: "f".into(),
                kind: EntryKind::Alias(TomlAlias::Command("cargo fmt".into())),
                hash: "x".into(),
            },
            EffectiveEntry {
                name: "ll".into(),
                kind: EntryKind::Alias(TomlAlias::Command("ls -lha".into())),
                hash: "y".into(),
            },
        ];
        let outcome = ResolveOutcome {
            diff: PrecedenceDiff {
                added: effective,
                ..Default::default()
            },
            diagnostics: vec![],
        };

        let out = render(
            &PathBuf::from("/tmp/x"),
            &chain,
            &outcome,
            &layers,
            None,
            RenderOptions { verbose: false },
        );

        // Section presence
        assert!(out.contains("# amoxide aliases"), "preamble present: {out}");
        assert!(out.contains("## Aliases"), "aliases header present: {out}");
        // No variables section (none defined); note the preamble comment references
        // "## Variables" so we check for the standalone section header line.
        assert!(
            !out.contains("\n## Variables\n"),
            "no vars means no section: {out}"
        );
        // No shadowed section (no shadows)
        assert!(!out.contains("## Shadowed"), "no shadows: {out}");
        // No invalid section (brief mode)
        assert!(!out.contains("## Invalid"), "brief omits invalid: {out}");

        // Section order
        let preamble_idx = out.find("# amoxide aliases").unwrap();
        let aliases_idx = out.find("## Aliases").unwrap();
        assert!(preamble_idx < aliases_idx, "preamble before aliases");
    }

    #[test]
    fn render_includes_invalid_only_in_verbose_mode() {
        let global = AliasSet::default();
        let global_subs = SubcommandSet::new();
        let global_vars = VarSet::default();
        let project_aliases = AliasSet::default();
        let project_subs = SubcommandSet::new();
        let project_vars = VarSet::default();
        let layers = LayerInputs {
            global_aliases: &global,
            global_subcommands: &global_subs,
            global_vars: &global_vars,
            profile_layers: &[],
            project_aliases: &project_aliases,
            project_subcommands: &project_subs,
            project_vars: &project_vars,
        };
        let chain = PrecedenceChain {
            layers: vec![ChainLayer {
                scope: OriginScope::Global,
                priority: None,
            }],
        };
        let outcome = ResolveOutcome {
            diff: PrecedenceDiff::default(),
            diagnostics: vec![Diagnostic {
                message: "alias `cc` missing vars: x".into(),
            }],
        };

        let brief = render(
            &PathBuf::from("/x"),
            &chain,
            &outcome,
            &layers,
            None,
            RenderOptions { verbose: false },
        );
        let verbose = render(
            &PathBuf::from("/x"),
            &chain,
            &outcome,
            &layers,
            None,
            RenderOptions { verbose: true },
        );
        assert!(
            !brief.contains("## Invalid"),
            "brief hides invalid: {brief}"
        );
        assert!(
            verbose.contains("## Invalid"),
            "verbose shows invalid: {verbose}"
        );
        assert!(verbose.contains("alias `cc` missing vars: x"));
    }
}
