use std::path::Path;

use unicode_width::UnicodeWidthStr;

use crate::described::Described;
use crate::trust::ProjectTrust;
use crate::{AliasDisplayFilter, AliasSet, Profile, ProfileConfig};

// ── alignment types ───────────────────────────────────────────────────────────

#[derive(Debug)]
struct Row {
    prefix: String,
    body: String,
    description: Option<String>,
}

impl Row {
    fn width(&self) -> usize {
        self.prefix.width() + self.body.width()
    }
}

const HASH_GAP_PADDED: &str = "  # ";
const HASH_GAP_INLINE: &str = " # ";

fn compute_target_column(rows: &[OutputItem]) -> usize {
    rows.iter()
        .filter_map(|item| match item {
            OutputItem::Row(r) if r.description.is_some() => Some(r.width()),
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

fn render_row(row: &Row, target_col: usize, term_width: Option<usize>) -> String {
    let mut out = format!("{}{}", row.prefix, row.body);
    let Some(desc) = row.description.as_deref() else {
        return out;
    };
    let line_width = row.width();
    let padded_width = target_col + HASH_GAP_PADDED.width() + desc.width();
    let inline_width = line_width + HASH_GAP_INLINE.width() + desc.width();

    let use_inline = match term_width {
        Some(w) => padded_width > w && inline_width <= w,
        None => false, // no terminal → always pad
    };

    if use_inline {
        out.push_str(HASH_GAP_INLINE);
        out.push_str(desc);
    } else {
        let pad = target_col.saturating_sub(line_width);
        out.push_str(&" ".repeat(pad));
        out.push_str(HASH_GAP_PADDED);
        out.push_str(desc);
    }
    out
}

// ── output item enum ──────────────────────────────────────────────────────────

enum OutputItem {
    /// Raw text written as-is (headers, trunk lines, blank separators).
    Header(String),
    /// An alias / subcommand entry that participates in column alignment.
    Row(Row),
}

// ── display_path helper ───────────────────────────────────────────────────────

/// Display a path as `~/…` when it falls under the user's home directory.
fn display_path(path: &Path) -> String {
    if let Some(home) = crate::dirs::home_dir() {
        if let Ok(rel) = path.strip_prefix(&home) {
            return format!("~{}{}", std::path::MAIN_SEPARATOR, rel.display());
        }
    }
    path.display().to_string()
}

// ── render_items ──────────────────────────────────────────────────────────────

/// Append one section's aliases and subcommand groups into `items`.
///
/// `prefix` carries the outer trunk context (e.g. `"│  "` under global).
/// Headers and trunk lines are pushed as `OutputItem::Header`; alias /
/// subcommand entries are pushed as `OutputItem::Row`.
fn render_items(
    items: &mut Vec<OutputItem>,
    prefix: &str,
    aliases: &AliasSet,
    subcommands: &crate::subcommand::SubcommandSet,
) {
    let subcmd_groups = subcommands.group_by_program();

    let alias_items: Vec<(String, String, Option<String>)> = aliases
        .iter()
        .map(|(k, v)| {
            (
                k.as_ref().to_string(),
                v.command().to_string(),
                v.description().map(str::to_owned),
            )
        })
        .collect();
    let group_items: Vec<(String, Vec<crate::subcommand::SubcommandEntry>)> =
        subcmd_groups.into_iter().collect();

    let alias_count = alias_items.len();
    let total = alias_count + group_items.len();

    for (i, (name, cmd, desc)) in alias_items.iter().enumerate() {
        let is_last = i + 1 == total;
        let conn = if is_last {
            "\u{2570}\u{2500}"
        } else {
            "\u{251c}\u{2500}"
        };
        // Keep the leading newline separate so Row.width() is purely visual.
        items.push(OutputItem::Header("\n".to_string()));
        let prefix_str = format!("{prefix}{conn} ");
        let body = format!("{name} \u{2192} {cmd}");
        items.push(OutputItem::Row(Row {
            prefix: prefix_str,
            body,
            description: desc.clone(),
        }));
    }

    for (gi, (program, entries)) in group_items.iter().enumerate() {
        let is_last = alias_count + gi + 1 == total;
        let conn = if is_last {
            "\u{2570}\u{2500}"
        } else {
            "\u{251c}\u{2500}"
        };
        items.push(OutputItem::Header(format!(
            "\n{prefix}{conn}\u{25c6} {program} (subcommands)"
        )));

        let inner: String = if is_last {
            format!("{}  ", prefix)
        } else {
            format!("{}\u{2502} ", prefix)
        };

        let mut entry_iter = entries.iter().peekable();
        while let Some(entry) = entry_iter.next() {
            let entry_conn = if entry_iter.peek().is_none() {
                "\u{2570}\u{2500}"
            } else {
                "\u{251c}\u{2500}"
            };
            let shorts = entry.short_subcommands.join(" ");
            let longs = entry.long_subcommands.join(" ");
            items.push(OutputItem::Header("\n".to_string()));
            items.push(OutputItem::Row(Row {
                prefix: format!("{inner}{entry_conn} "),
                body: format!("{shorts} \u{2192} {longs}"),
                description: entry.description.clone(),
            }));
        }
    }
}

// ── render_listing ────────────────────────────────────────────────────────────

/// Render profiles + project aliases as a complete two-zone listing.
///
/// **Active zone** (connected by tree trunk):
///   global → active profiles (by activation order) → project
///
/// **Inactive zone** (flat, alphabetical):
///   remaining profiles
#[allow(clippy::too_many_arguments)]
pub fn render_listing(
    global_aliases: &AliasSet,
    global_subcommands: &crate::subcommand::SubcommandSet,
    config: &ProfileConfig,
    active_profiles: &[String],
    project: Option<&ProjectTrust>,
    filter: Option<AliasDisplayFilter>,
    descriptions: bool,
    term_width: Option<usize>,
) -> String {
    let mut items: Vec<OutputItem> = Vec::new();

    // Collect active profiles in activation order
    let active_ordered: Vec<&Profile> = active_profiles
        .iter()
        .filter_map(|name| config.get_profile_by_name(name))
        .collect();

    // Collect inactive profiles (alphabetical, already sorted in ProfileConfig)
    let inactive: Vec<&Profile> = config
        .iter()
        .filter(|p| !active_profiles.contains(&p.name))
        .collect();

    let has_active_items = !active_ordered.is_empty() || project.is_some();

    // ── Active zone ──────────────────────────────────────────────

    // Global header
    items.push(OutputItem::Header("\u{1f310} global".to_string()));
    let global_prefix = if has_active_items {
        "\u{2502}  "
    } else {
        "   "
    };
    render_items(
        &mut items,
        global_prefix,
        global_aliases,
        global_subcommands,
    );

    if has_active_items {
        items.push(OutputItem::Header("\n\u{2502}".to_string()));
    }

    // Active profiles
    for (i, profile) in active_ordered.iter().enumerate() {
        let order = active_profiles
            .iter()
            .position(|n| n == &profile.name)
            .map(|idx| idx + 1)
            .unwrap_or(0);

        let is_last_active_item = i == active_ordered.len() - 1 && project.is_none();

        let connector = if is_last_active_item {
            "\u{2570}\u{2500}"
        } else {
            "\u{251c}\u{2500}"
        };
        let trunk = if is_last_active_item { " " } else { "\u{2502}" };

        items.push(OutputItem::Header(format!(
            "\n{connector}\u{25cf} {} (active: {order})",
            profile.name
        )));

        let profile_prefix = format!("{trunk}   ");
        render_items(
            &mut items,
            &profile_prefix,
            &profile.aliases,
            &profile.subcommands,
        );

        if !is_last_active_item {
            items.push(OutputItem::Header(format!("\n{trunk}")));
        }
    }

    // Project aliases (last in active zone)
    if let Some(trust) = &project {
        let path = trust.path();
        match trust {
            ProjectTrust::Trusted(proj, _) => {
                items.push(OutputItem::Header(format!(
                    "\n\u{2570}\u{2500}\u{1f4c1} project ({})",
                    display_path(path)
                )));
                render_items(&mut items, "  ", &proj.aliases, &proj.subcommands);
            }
            ProjectTrust::Unknown(_) => {
                items.push(OutputItem::Header(format!(
                    "\n\u{2570}\u{2500}\u{1f4c1} project ({})",
                    display_path(path)
                )));
                items.push(OutputItem::Header(
                    "\n       \u{26A0} untrusted \u{2014} run 'am trust' to review and allow"
                        .to_string(),
                ));
            }
            ProjectTrust::Tampered(_) => {
                items.push(OutputItem::Header(format!(
                    "\n\u{2570}\u{2500}\u{1f4c1} project ({})",
                    display_path(path)
                )));
                items.push(OutputItem::Header(
                    "\n       \u{26A0} modified since last trust \u{2014} run 'am trust' to review and allow"
                        .to_string(),
                ));
            }
            ProjectTrust::Untrusted(_) => {
                items.push(OutputItem::Header(format!(
                    "\n\u{2570}\u{2500}\u{1f4c1} project ({})",
                    display_path(path)
                )));
                items.push(OutputItem::Header(
                    "\n       \u{26A0} blocked \u{2014} run 'am untrust --forget' to reset"
                        .to_string(),
                ));
            }
        }
    }

    // ── Inactive zone ────────────────────────────────────────────

    if !matches!(filter, Some(AliasDisplayFilter::Used)) && !inactive.is_empty() {
        items.push(OutputItem::Header("\n".to_string()));
        for profile in &inactive {
            items.push(OutputItem::Header(format!("\n\u{25cb} {}", profile.name)));
            render_items(&mut items, "  ", &profile.aliases, &profile.subcommands);
            items.push(OutputItem::Header("\n".to_string()));
        }
    }

    // ── Two-pass rendering ────────────────────────────────────────

    // If descriptions are disabled, clear them before computing column width.
    if !descriptions {
        for item in &mut items {
            if let OutputItem::Row(row) = item {
                row.description = None;
            }
        }
    }

    let target_col = compute_target_column(&items);

    let mut output = String::new();
    for item in &items {
        match item {
            OutputItem::Header(s) => output.push_str(s),
            OutputItem::Row(row) => output.push_str(&render_row(row, target_col, term_width)),
        }
    }

    output
}

// ── render_profiles ───────────────────────────────────────────────────────────

/// Render profiles as a two-zone display (active zone + inactive zone).
///
/// Active profiles show `● name (active: N)` with activation order.
/// Inactive profiles show `○ name`.
pub fn render_profiles(config: &ProfileConfig, active_profiles: &[String]) -> String {
    // Collect active profiles in activation order
    let active_ordered: Vec<&Profile> = active_profiles
        .iter()
        .filter_map(|name| config.get_profile_by_name(name))
        .collect();

    // Collect inactive profiles (alphabetical, already sorted in ProfileConfig)
    let inactive: Vec<&Profile> = config
        .iter()
        .filter(|p| !active_profiles.contains(&p.name))
        .collect();

    let mut lines: Vec<String> = Vec::new();

    // Active profiles
    for profile in &active_ordered {
        let order = active_profiles
            .iter()
            .position(|n| n == &profile.name)
            .map(|idx| idx + 1)
            .unwrap_or(0);

        lines.push(format!("\u{25cf} {} (active: {order})", profile.name));

        if profile.aliases.is_empty() {
            lines.push("  (no aliases)".to_string());
        } else {
            for (alias_name, alias_value) in profile.aliases.iter() {
                let name = alias_name.as_ref();
                let cmd = alias_value.command();
                lines.push(format!("  {name} \u{2192} {cmd}"));
            }
        }

        lines.push(String::new());
    }

    // Inactive profiles
    for (i, profile) in inactive.iter().enumerate() {
        lines.push(format!("\u{25cb} {}", profile.name));

        if profile.aliases.is_empty() {
            lines.push("  (no aliases)".to_string());
        } else {
            for (alias_name, alias_value) in profile.aliases.iter() {
                let name = alias_name.as_ref();
                let cmd = alias_value.command();
                lines.push(format!("  {name} \u{2192} {cmd}"));
            }
        }

        // Blank line between profiles (but not after the last)
        if i < inactive.len() - 1 {
            lines.push(String::new());
        }
    }

    lines.join("\n")
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::project::ProjectAliases;
    use crate::ProfileConfig;
    use indoc::indoc;

    fn make_config(toml_str: &str) -> ProfileConfig {
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_active_profile_shows_order() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"
        "#});

        let output = render_profiles(&config, &["git".to_string()]);
        assert!(output.contains("● git (active: 1)"));
        assert!(output.contains("  gs → git status"));
    }

    #[test]
    fn test_multiple_active_profiles_ordered() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_profiles(&config, &["git".to_string(), "rust".to_string()]);
        assert!(output.contains("● git (active: 1)"));
        assert!(output.contains("● rust (active: 2)"));
    }

    #[test]
    fn test_inactive_profiles_shown_after_active() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "node"
            [profiles.aliases]
            b = "npm run build"

            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_profiles(&config, &["rust".to_string()]);
        // Active first
        assert!(output.contains("● rust (active: 1)"));
        // Inactive alphabetical
        assert!(output.contains("○ git"));
        assert!(output.contains("○ node"));
        // Active should appear before inactive
        let active_pos = output.find("● rust").unwrap();
        let inactive_git_pos = output.find("○ git").unwrap();
        assert!(active_pos < inactive_git_pos);
    }

    #[test]
    fn test_empty_profile_shows_no_aliases() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "empty"
        "#});

        let output = render_profiles(&config, &["empty".to_string()]);
        assert!(output.contains("● empty (active: 1)"));
        assert!(output.contains("(no aliases)"));
    }

    #[test]
    fn test_listing_global_with_trunk() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let mut globals = AliasSet::default();
        globals.insert(
            "ll".into(),
            crate::TomlAlias::Command("ls -lha".to_string()),
        );

        let output = render_listing(
            &globals,
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["rust".to_string()],
            None,
            None,
            false,
            None,
        );
        // Global with trunk
        assert!(output.contains("🌐 global"));
        assert!(output.contains("│  ╰─ ll → ls -lha"));
        // Active profile with connector
        assert!(output.contains("╰─● rust (active: 1)"));
    }

    #[test]
    fn test_listing_active_profiles_with_project() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "git"
            [profiles.aliases]
            gs = "git status"

            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".aliases");
        std::fs::write(&path, "[aliases]\nt = \"cargo test\"\n").unwrap();
        let project = ProjectAliases::load(&path).unwrap();
        let trust = ProjectTrust::Trusted(project, std::path::PathBuf::from(".aliases"));

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["git".to_string(), "rust".to_string()],
            Some(&trust),
            None,
            false,
            None,
        );
        assert!(output.contains("├─● git (active: 1)"));
        assert!(output.contains("├─● rust (active: 2)"));
        assert!(output.contains("╰─📁 project"));
        assert!(output.contains("t → cargo test"));
    }

    #[test]
    fn test_listing_last_active_gets_corner_when_no_project() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["rust".to_string()],
            None,
            None,
            false,
            None,
        );
        assert!(output.contains("╰─● rust (active: 1)"));
    }

    #[test]
    fn test_listing_inactive_profiles_below() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "foo"
            [profiles.aliases]
            sayt = "echo say it"

            [[profiles]]
            name = "rust"
            [profiles.aliases]
            ct = "cargo test"
        "#});

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["rust".to_string()],
            None,
            None,
            false,
            None,
        );
        assert!(output.contains("╰─● rust (active: 1)"));
        assert!(output.contains("○ foo"));
        assert!(output.contains("  ╰─ sayt → echo say it"));
    }

    #[test]
    fn test_listing_global_alone_no_trunk() {
        let config: ProfileConfig = ProfileConfig::default();

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &[],
            None,
            None,
            false,
            None,
        );
        assert!(output.contains("🌐 global"));
        // No trunk when global stands alone
        assert!(!output.contains("│"));
    }

    #[test]
    fn test_listing_with_project_aliases() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "default"
            [profiles.aliases]
            ll = "ls -lha"
        "#});

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".aliases");
        std::fs::write(&path, "[aliases]\nt = \"cargo test\"\n").unwrap();
        let project = ProjectAliases::load(&path).unwrap();
        let trust = ProjectTrust::Trusted(project, std::path::PathBuf::from(".aliases"));

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["default".to_string()],
            Some(&trust),
            None,
            false,
            None,
        );
        assert!(output.contains("● default (active: 1)"));
        assert!(output.contains("📁 project"));
        assert!(output.contains("t → cargo test"));
    }

    #[test]
    fn test_listing_without_project_aliases() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "default"
        "#});

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["default".to_string()],
            None,
            None,
            false,
            None,
        );
        assert!(output.contains("● default (active: 1)"));
        assert!(!output.contains("📁"));
    }

    #[test]
    fn test_listing_used_filter_hides_inactive_profiles() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "active"
            [profiles.aliases]
            t = "cargo test"

            [[profiles]]
            name = "inactive"
            [profiles.aliases]
            b = "cargo build"
        "#});

        let output = render_listing(
            &AliasSet::default(),
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["active".to_string()],
            None,
            Some(AliasDisplayFilter::Used),
            false,
            None,
        );

        assert!(
            output.contains("active"),
            "active profile should be visible"
        );
        assert!(
            !output.contains("inactive"),
            "inactive profile should be hidden"
        );
        assert!(
            !output.contains("cargo build"),
            "inactive alias should be hidden"
        );
    }

    #[test]
    fn test_listing_global_subcommands() {
        use crate::subcommand::{SubcommandSet, TomlSubcommand};

        let config: ProfileConfig = ProfileConfig::default();
        let mut subs = SubcommandSet::new();
        subs.as_mut()
            .insert("jj:ab".into(), TomlSubcommand::Expansion(vec!["abandon".into()]));
        subs.as_mut().insert(
            "jj:b:l".into(),
            TomlSubcommand::Expansion(vec!["branch".into(), "list".into()]),
        );

        let output = render_listing(
            &AliasSet::default(),
            &subs,
            &config,
            &[],
            None,
            None,
            false,
            None,
        );
        assert!(output.contains("jj (subcommands)"));
        assert!(output.contains("ab → abandon"));
        assert!(output.contains("b l → branch list"));
    }

    // ── new alignment tests ───────────────────────────────────────────────────

    #[test]
    fn listing_with_descriptions_pads_to_global_column() {
        let config = make_config(indoc! {r#"
            [[profiles]]
            name = "rust"
            [profiles.aliases]
            b = { command = "cargo b --release", description = "release build" }
            t = { command = "cargo test --all-features", description = "run all tests" }
        "#});

        let mut globals = AliasSet::default();
        globals.insert(
            "ll".into(),
            crate::TomlAlias::Detailed(crate::AliasDetail {
                command: "ls -lha".into(),
                description: Some("long listing".into()),
                raw: false,
            }),
        );

        let output = render_listing(
            &globals,
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &["rust".to_string()],
            None,
            None,
            /* descriptions */ true,
            /* term_width */ Some(120),
        );

        // All three described rows align at the same `#` column.
        // Use char-count (not byte offset) so multi-byte unicode box-drawing
        // chars are counted as single columns, matching terminal display width.
        let lines: Vec<&str> = output.lines().filter(|l| l.contains('#')).collect();
        let hash_cols: Vec<usize> = lines
            .iter()
            .map(|l| l.chars().take_while(|&c| c != '#').count())
            .collect();
        assert!(
            hash_cols.windows(2).all(|w| w[0] == w[1]),
            "`#` columns not aligned: {hash_cols:?}\n{output}"
        );
    }

    #[test]
    fn listing_with_descriptions_falls_back_to_inline_when_narrow() {
        let config: ProfileConfig = ProfileConfig::default();
        let mut globals = AliasSet::default();
        globals.insert(
            "verylongname".into(),
            crate::TomlAlias::Detailed(crate::AliasDetail {
                command: "this is a fairly long command line".into(),
                description: Some("desc".into()),
                raw: false,
            }),
        );
        globals.insert(
            "x".into(),
            crate::TomlAlias::Detailed(crate::AliasDetail {
                command: "y".into(),
                description: Some("d".into()),
                raw: false,
            }),
        );

        let output = render_listing(
            &globals,
            &crate::subcommand::SubcommandSet::new(),
            &config,
            &[],
            None,
            None,
            /* descriptions */ true,
            /* term_width */ Some(40), // too narrow to pad
        );

        // The short row gets inline form, not padded out to the long row's column.
        let short_line = output.lines().find(|l| l.contains("x \u{2192} y")).unwrap();
        let single_space_before_hash = short_line.contains("y # d");
        assert!(
            single_space_before_hash,
            "expected inline form on narrow terminal:\n{output}"
        );
    }

    #[test]
    fn listing_subcommand_descriptions_align_with_alias_descriptions() {
        use crate::subcommand::{SubcommandDetail, SubcommandSet, TomlSubcommand};
        let mut globals = AliasSet::default();
        globals.insert(
            "ll".into(),
            crate::TomlAlias::Detailed(crate::AliasDetail {
                command: "ls -lha".into(),
                description: Some("long listing".into()),
                raw: false,
            }),
        );

        let mut subs = SubcommandSet::new();
        subs.as_mut().insert(
            "jj:ab".into(),
            TomlSubcommand::Detailed(SubcommandDetail {
                expansions: vec!["abandon".into()],
                description: Some("toss the change".into()),
            }),
        );

        let output = render_listing(
            &globals,
            &subs,
            &ProfileConfig::default(),
            &[],
            None,
            None,
            /* descriptions */ true,
            /* term_width */ Some(120),
        );

        // Find the two lines that carry `#` (the alias and the subcommand entry)
        let described_lines: Vec<&str> = output.lines().filter(|l| l.contains('#')).collect();
        assert_eq!(
            described_lines.len(),
            2,
            "expected 2 described lines, got {}:\n{output}",
            described_lines.len()
        );
        let hash_cols: Vec<usize> =
            described_lines.iter().map(|l| l.find('#').unwrap()).collect();
        assert_eq!(
            hash_cols[0], hash_cols[1],
            "subcommand and alias descriptions not aligned: {hash_cols:?}\n{output}"
        );
        assert!(output.contains("toss the change"));
        assert!(output.contains("long listing"));
    }

    #[test]
    fn listing_without_descriptions_flag_omits_them_entirely() {
        let mut globals = AliasSet::default();
        globals.insert(
            "ll".into(),
            crate::TomlAlias::Detailed(crate::AliasDetail {
                command: "ls -lha".into(),
                description: Some("long listing".into()),
                raw: false,
            }),
        );

        let output = render_listing(
            &globals,
            &crate::subcommand::SubcommandSet::new(),
            &ProfileConfig::default(),
            &[],
            None,
            None,
            /* descriptions */ false,
            Some(120),
        );
        assert!(!output.contains('#'));
        assert!(!output.contains("long listing"));
    }
}
