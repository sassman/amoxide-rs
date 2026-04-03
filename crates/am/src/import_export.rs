use std::io::BufRead;
use std::time::Duration;

use crate::alias::MergeResult;
use crate::cli::{ExportArgs, ImportArgs, ShareArgs};
use crate::effects::Effect;
use crate::exchange::{
    base64_decode, base64_encode, parse_import, render_import_summary, render_suspicious_warning,
    scan_suspicious, ExportAll, ImportPayload, SanitizedName, Scope,
};
use crate::prompt::{ask_user, Answer};
use crate::update::{update, AppModel};
use crate::{AliasSet, Message, Profile};

// ═══════════════════════════════════════════════════════════════════════
// Export
// ═══════════════════════════════════════════════════════════════════════

pub fn handle_export(model: &AppModel, args: &ExportArgs) -> anyhow::Result<String> {
    let toml_output = export_toml(model, args)?;

    if args.base64 {
        Ok(base64_encode(&toml_output))
    } else {
        Ok(toml_output)
    }
}

fn export_toml(model: &AppModel, args: &ExportArgs) -> anyhow::Result<String> {
    if args.scope.all {
        // --all: everything
        let project_aliases = model.project_alias_set();
        let export = ExportAll {
            global_aliases: model.config.aliases.clone(),
            profiles: model.profile_config().to_vec(),
            local_aliases: project_aliases,
        };
        return Ok(toml::to_string(&export)?);
    }

    let has_scope = args.scope.local || args.scope.global || !args.scope.profile.is_empty();
    if !has_scope {
        // No flags: active scope (global + active profiles + local if present)
        let active_profiles: Vec<_> = model
            .config
            .active_profiles
            .iter()
            .filter_map(|name| model.profile_config().get_profile_by_name(name))
            .cloned()
            .collect();
        let project_aliases = model.project_alias_set();
        let export = ExportAll {
            global_aliases: model.config.aliases.clone(),
            profiles: active_profiles,
            local_aliases: project_aliases,
        };
        return Ok(toml::to_string(&export)?);
    }

    // Combinable scope flags: collect from each selected scope
    let mut export = ExportAll::default();

    if args.scope.global {
        export.global_aliases = model.config.aliases.clone();
    }

    for name in &args.scope.profile {
        let profile = model
            .profile_config()
            .get_profile_by_name(name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{name}' not found"))?;
        export.profiles.push(profile.clone());
    }

    if args.scope.local {
        let project = model
            .project_aliases()
            .ok_or_else(|| anyhow::anyhow!("No .aliases file found in directory tree"))?;
        export.local_aliases = project.aliases.clone();
    }

    Ok(toml::to_string(&export)?)
}

// ═══════════════════════════════════════════════════════════════════════
// Share
// ═══════════════════════════════════════════════════════════════════════

pub fn handle_share(args: &ShareArgs) -> String {
    let scope_flags = build_scope_flags(&args.scope);

    if args.termbin {
        format!("am export{scope_flags} -b64 | nc termbin.com 9999")
    } else if args.paste_rs {
        format!("am export{scope_flags} -b64 | curl -d @- https://paste.rs/")
    } else {
        // No target — show help
        String::from(
            r#"Share your aliases with others via a pastebin service.

Available targets:

  --termbin    Post via netcat to termbin.com
               Example: am share -p git --termbin
               Output:  am export -p git -b64 | nc termbin.com 9999

  --paste-rs   Post via curl to paste.rs
               Example: am share -p git --paste-rs
               Output:  am export -p git -b64 | curl -d @- https://paste.rs/

On PowerShell, replace the pipe with:
               am export -p git -b64 | ForEach-Object { curl -d $_ https://paste.rs/ }

Run the generated command to upload. Share the returned URL.
The receiver imports with: am import <url> -b64
"#,
        )
    }
}

fn build_scope_flags(scope: &crate::cli::ScopeArgs) -> String {
    let mut flags = String::new();

    if scope.local {
        flags.push_str(" -l");
    }
    if scope.global {
        flags.push_str(" -g");
    }
    for name in &scope.profile {
        flags.push_str(&format!(" -p {name}"));
    }
    if scope.all {
        flags.push_str(" --all");
    }

    flags
}

// ═══════════════════════════════════════════════════════════════════════
// Import
// ═══════════════════════════════════════════════════════════════════════

pub fn handle_import(model: &mut AppModel, args: &ImportArgs) -> anyhow::Result<()> {
    // Phase 1: Read from URL or file
    let source = &args.source;
    let input = if source.starts_with("http://") || source.starts_with("https://") {
        eprintln!("Fetching {source}...");
        fetch_url(source)?
    } else {
        std::fs::read_to_string(source)
            .map_err(|e| anyhow::anyhow!("failed to read '{}': {}", source, e))?
    };

    if input.trim().is_empty() {
        anyhow::bail!("no aliases found in '{source}'");
    }

    let toml_input = if args.base64 {
        base64_decode(&input)?
    } else {
        input
    };

    let parsed = parse_import(&toml_input)?;

    // Security scan: check for suspicious control characters
    let findings = scan_suspicious(&parsed);
    if !findings.is_empty() {
        if args.trust {
            let n = findings.len();
            let label = if n == 1 { "entry" } else { "entries" };
            eprintln!("WARNING: {n} suspicious {label} found, proceeding due to --trust");
        } else {
            eprint!("{}", render_suspicious_warning(&findings));
            anyhow::bail!(
                "refusing to import: suspicious characters detected. \
                 Use --yes --trust to override."
            );
        }
    }

    // Phase 2: Resolve conflicts + Phase 3: Apply
    // stdin is never consumed — prompts always work
    let mut stdin = std::io::stdin().lock();
    if args.scope.local || args.scope.global || !args.scope.profile.is_empty() {
        import_with_override(model, args.yes, &args.scope, &parsed, &mut stdin)?;
    } else {
        import_auto_route(model, args.yes, &parsed, &mut stdin)?;
    }

    Ok(())
}

fn import_auto_route(
    model: &mut AppModel,
    auto_yes: bool,
    parsed: &ExportAll,
    reader: &mut dyn BufRead,
) -> anyhow::Result<()> {
    let mut payload = ImportPayload::default();

    if !parsed.global_aliases.is_empty() {
        let merge = model.config.aliases.merge_check(&parsed.global_aliases);
        if let Some(accepted) = prompt_merge(&Scope::Global, &merge, auto_yes, reader)? {
            payload.global_aliases = Some(accepted);
        }
    }

    for profile in &parsed.profiles {
        let existing_aliases = model
            .profile_config()
            .get_profile_by_name(&profile.name)
            .map(|p| p.aliases.clone())
            .unwrap_or_default();
        let merge = existing_aliases.merge_check(&profile.aliases);
        let scope = Scope::Profile(SanitizedName::new(&profile.name));
        if let Some(accepted) = prompt_merge(&scope, &merge, auto_yes, reader)? {
            payload.profiles.push(Profile {
                name: profile.name.clone(),
                aliases: accepted,
            });
        }
    }

    if !parsed.local_aliases.is_empty() {
        let existing = model.project_alias_set();
        let merge = existing.merge_check(&parsed.local_aliases);
        if let Some(accepted) = prompt_merge(&Scope::Local, &merge, auto_yes, reader)? {
            payload.local_aliases = Some(accepted);
        }
    }

    apply_import(model, payload)
}

fn import_with_override(
    model: &mut AppModel,
    auto_yes: bool,
    scope_args: &crate::cli::ScopeArgs,
    parsed: &ExportAll,
    reader: &mut dyn BufRead,
) -> anyhow::Result<()> {
    let flattened = parsed.flatten();
    let mut payload = ImportPayload::default();

    if scope_args.global {
        let merge = model.config.aliases.merge_check(&flattened);
        if let Some(accepted) = prompt_merge(&Scope::Global, &merge, auto_yes, reader)? {
            payload.global_aliases = Some(accepted);
        }
    }

    for name in &scope_args.profile {
        let existing_aliases = model
            .profile_config()
            .get_profile_by_name(name)
            .map(|p| p.aliases.clone())
            .unwrap_or_default();
        let merge = existing_aliases.merge_check(&flattened);
        let scope = Scope::Profile(SanitizedName::new(name));
        if let Some(accepted) = prompt_merge(&scope, &merge, auto_yes, reader)? {
            payload.profiles.push(Profile {
                name: name.clone(),
                aliases: accepted,
            });
        }
    }

    if scope_args.local {
        let existing = model.project_alias_set();
        let merge = existing.merge_check(&flattened);
        if let Some(accepted) = prompt_merge(&Scope::Local, &merge, auto_yes, reader)? {
            payload.local_aliases = Some(accepted);
        }
    }

    apply_import(model, payload)
}

pub fn prompt_merge(
    scope: &Scope,
    merge: &MergeResult,
    auto_yes: bool,
    reader: &mut dyn BufRead,
) -> anyhow::Result<Option<AliasSet>> {
    if merge.new_aliases.is_empty() && merge.conflicts.is_empty() {
        eprintln!("Nothing new to import into \"{scope}\"");
        return Ok(None);
    }

    eprint!("{}", render_import_summary(&scope.to_string(), merge));
    eprintln!();

    // Ask to merge
    if !auto_yes {
        let answer = ask_user(
            &format!("Merge into \"{scope}\"?"),
            Answer::Yes,
            false,
            reader,
        )?;
        if answer != Answer::Yes {
            eprintln!("Skipped \"{scope}\"");
            return Ok(None);
        }
    }

    // Start with new aliases
    let mut accepted = merge.new_aliases.clone();

    // Ask about overwrites
    if !merge.conflicts.is_empty() {
        let n = merge.conflicts.len();
        let label = if n == 1 { "overwrite" } else { "overwrites" };
        let apply_overwrites = if auto_yes {
            true
        } else {
            let answer = ask_user(&format!("Apply {n} {label}?"), Answer::No, false, reader)?;
            answer == Answer::Yes
        };

        if apply_overwrites {
            for conflict in &merge.conflicts {
                accepted.insert(conflict.name.clone(), conflict.incoming.clone());
            }
        }

        let imported = accepted.len();
        let skipped = if apply_overwrites { 0 } else { n };
        eprintln!("\u{2713} Imported {imported} aliases into \"{scope}\" ({skipped} skipped)");
    } else {
        eprintln!(
            "\u{2713} Imported {} aliases into \"{scope}\"",
            accepted.len()
        );
    }

    Ok(Some(accepted))
}

fn apply_import(model: &mut AppModel, payload: ImportPayload) -> anyhow::Result<()> {
    let local_aliases = payload.local_aliases.clone();

    // Dispatch through message pipeline — returns effects
    let result = update(model, Message::Import(payload))?;

    // Execute effects (SaveConfig, SaveProfiles)
    for effect in &result.effects {
        match effect {
            Effect::SaveConfig => model.config.save()?,
            Effect::SaveProfiles => model.profile_config().save()?,
            _ => {}
        }
    }

    // Save local aliases directly (needs file path, not handled by effects)
    if let Some(aliases) = local_aliases {
        model.save_project_aliases(aliases)?;
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// URL fetching
// ═══════════════════════════════════════════════════════════════════════

/// Maximum response body size: 1 MB
const MAX_BODY_BYTES: u64 = 1_048_576;

/// Fetch TOML content from a URL with safety limits.
///
/// - 10 second connect timeout
/// - 30 second global timeout
/// - 1 MB maximum response body
/// - SSL verification enforced via native-tls
pub fn fetch_url(url: &str) -> anyhow::Result<String> {
    let config = ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(10)))
        .timeout_global(Some(Duration::from_secs(30)))
        .https_only(url.starts_with("https://"))
        .build();

    let agent = config.new_agent();

    let mut response = agent
        .get(url)
        .call()
        .map_err(|e| anyhow::anyhow!("failed to fetch URL: {e}"))?;

    let body = response
        .body_mut()
        .with_config()
        .limit(MAX_BODY_BYTES)
        .read_to_string()
        .map_err(|e| anyhow::anyhow!("failed to read response body: {e}"))?;

    Ok(body)
}

/// Check if a source string is a URL (vs a file path).
pub fn is_url(source: &str) -> bool {
    source.starts_with("http://") || source.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_url_https() {
        assert!(is_url("https://example.com/aliases.toml"));
    }

    #[test]
    fn test_is_url_http() {
        assert!(is_url("http://example.com/aliases.toml"));
    }

    #[test]
    fn test_is_url_ftp_is_not() {
        assert!(!is_url("ftp://example.com/file"));
    }

    #[test]
    fn test_is_url_file_path_is_not() {
        assert!(!is_url("/tmp/aliases.toml"));
    }

    #[test]
    fn test_is_url_relative_path_is_not() {
        assert!(!is_url("./profiles.toml"));
    }

    #[test]
    fn test_fetch_url_invalid_host_errors() {
        // This should fail with a connection error, not panic
        let result = fetch_url("http://this-host-does-not-exist.invalid/aliases.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed to fetch"));
    }

    // ─── handle_share tests ─────────────────────────────────────────

    #[test]
    fn test_share_termbin_with_profile() {
        let args = crate::cli::ShareArgs {
            scope: crate::cli::ScopeArgs {
                local: false,
                global: false,
                profile: vec!["git".into()],
                all: false,
            },
            termbin: true,
            paste_rs: false,
        };
        let output = handle_share(&args);
        assert_eq!(output, "am export -p git -b64 | nc termbin.com 9999");
    }

    #[test]
    fn test_share_paste_rs_all() {
        let args = crate::cli::ShareArgs {
            scope: crate::cli::ScopeArgs {
                local: false,
                global: false,
                profile: vec![],
                all: true,
            },
            termbin: false,
            paste_rs: true,
        };
        let output = handle_share(&args);
        assert_eq!(
            output,
            "am export --all -b64 | curl -d @- https://paste.rs/"
        );
    }

    #[test]
    fn test_share_no_target_shows_help() {
        let args = crate::cli::ShareArgs {
            scope: crate::cli::ScopeArgs {
                local: false,
                global: false,
                profile: vec![],
                all: false,
            },
            termbin: false,
            paste_rs: false,
        };
        let output = handle_share(&args);
        assert!(output.contains("Available targets:"));
        assert!(output.contains("--termbin"));
        assert!(output.contains("--paste-rs"));
    }

    #[test]
    fn test_share_combined_scope_flags() {
        let args = crate::cli::ShareArgs {
            scope: crate::cli::ScopeArgs {
                local: true,
                global: true,
                profile: vec!["git".into()],
                all: false,
            },
            termbin: true,
            paste_rs: false,
        };
        let output = handle_share(&args);
        assert!(output.contains("-l"));
        assert!(output.contains("-g"));
        assert!(output.contains("-p git"));
    }

    #[test]
    fn test_build_scope_flags_default() {
        let scope = crate::cli::ScopeArgs {
            local: false,
            global: false,
            profile: vec![],
            all: false,
        };
        assert_eq!(build_scope_flags(&scope), "");
    }

    #[test]
    fn test_build_scope_flags_multiple_profiles() {
        let scope = crate::cli::ScopeArgs {
            local: false,
            global: false,
            profile: vec!["git".into(), "rust".into()],
            all: false,
        };
        assert_eq!(build_scope_flags(&scope), " -p git -p rust");
    }

    // ─── prompt_merge tests ─────────────────────────────────────────

    use crate::alias::{AliasConflict, MergeResult};
    use std::io::Cursor;

    #[test]
    fn test_prompt_merge_auto_yes_all_new() {
        let mut new_aliases = AliasSet::default();
        new_aliases.insert("gs".into(), crate::TomlAlias::Command("git status".into()));

        let merge = MergeResult {
            new_aliases,
            conflicts: vec![],
        };

        let mut reader = Cursor::new(b"");
        let result = prompt_merge(&Scope::Global, &merge, true, &mut reader).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_prompt_merge_auto_yes_with_conflicts_accepts_all() {
        let new_aliases = AliasSet::default();
        let conflicts = vec![AliasConflict {
            name: "gs".into(),
            current: crate::TomlAlias::Command("git status --short".into()),
            incoming: crate::TomlAlias::Command("git status".into()),
        }];

        let merge = MergeResult {
            new_aliases,
            conflicts,
        };

        let mut reader = Cursor::new(b"");
        let result = prompt_merge(&Scope::Global, &merge, true, &mut reader).unwrap();
        assert!(result.is_some());
        let accepted = result.unwrap();
        assert_eq!(accepted.len(), 1);
        assert_eq!(accepted.get(&"gs".into()).unwrap().command(), "git status");
    }

    #[test]
    fn test_prompt_merge_empty_returns_none() {
        let merge = MergeResult {
            new_aliases: AliasSet::default(),
            conflicts: vec![],
        };

        let mut reader = Cursor::new(b"");
        let result = prompt_merge(&Scope::Global, &merge, true, &mut reader).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_prompt_merge_interactive_yes_accepts() {
        let mut new_aliases = AliasSet::default();
        new_aliases.insert("gs".into(), crate::TomlAlias::Command("git status".into()));

        let merge = MergeResult {
            new_aliases,
            conflicts: vec![],
        };

        // User presses Enter (default Yes for merge)
        let mut reader = Cursor::new(b"\n");
        let result = prompt_merge(&Scope::Global, &merge, false, &mut reader).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn test_prompt_merge_interactive_no_skips() {
        let mut new_aliases = AliasSet::default();
        new_aliases.insert("gs".into(), crate::TomlAlias::Command("git status".into()));

        let merge = MergeResult {
            new_aliases,
            conflicts: vec![],
        };

        let mut reader = Cursor::new(b"n\n");
        let result = prompt_merge(&Scope::Global, &merge, false, &mut reader).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_prompt_merge_interactive_conflicts_declined() {
        let conflicts = vec![AliasConflict {
            name: "gs".into(),
            current: crate::TomlAlias::Command("git status --short".into()),
            incoming: crate::TomlAlias::Command("git status".into()),
        }];

        let merge = MergeResult {
            new_aliases: AliasSet::default(),
            conflicts,
        };

        // First prompt: yes to merge, second: no to overwrites (default)
        let mut reader = Cursor::new(b"y\n\n");
        let result = prompt_merge(&Scope::Global, &merge, false, &mut reader).unwrap();
        assert!(result.is_some());
        // Overwrites declined — should be empty (no new aliases, conflicts skipped)
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_prompt_merge_interactive_conflicts_accepted() {
        let conflicts = vec![AliasConflict {
            name: "gs".into(),
            current: crate::TomlAlias::Command("git status --short".into()),
            incoming: crate::TomlAlias::Command("git status".into()),
        }];

        let merge = MergeResult {
            new_aliases: AliasSet::default(),
            conflicts,
        };

        // First prompt: yes to merge, second: yes to overwrites
        let mut reader = Cursor::new(b"y\ny\n");
        let result = prompt_merge(&Scope::Global, &merge, false, &mut reader).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }

    // ─── handle_export tests ─────────────────────────────────────────

    #[test]
    fn test_export_global_aliases() {
        let mut config = crate::config::Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let profile_config = crate::profile::ProfileConfig::default();
        let model = AppModel::new(config, profile_config);

        let args = ExportArgs {
            scope: crate::cli::ScopeArgs {
                local: false,
                global: true,
                profile: vec![],
                all: false,
            },
            base64: false,
        };

        let output = handle_export(&model, &args).unwrap();
        assert!(output.contains("[global_aliases]"));
        assert!(output.contains("ll = \"ls -lha\""));
    }

    #[test]
    fn test_export_base64() {
        let mut config = crate::config::Config::default();
        config.add_alias("ll".into(), "ls -lha".into(), false);
        let profile_config = crate::profile::ProfileConfig::default();
        let model = AppModel::new(config, profile_config);

        let args = ExportArgs {
            scope: crate::cli::ScopeArgs {
                local: false,
                global: true,
                profile: vec![],
                all: false,
            },
            base64: true,
        };

        let output = handle_export(&model, &args).unwrap();
        // Base64 output should not contain TOML markers
        assert!(!output.contains("[global_aliases]"));
        // But should decode back to TOML
        let decoded = base64_decode(&output).unwrap();
        assert!(decoded.contains("[global_aliases]"));
    }

    #[test]
    fn test_export_profile() {
        let config = crate::config::Config::default();
        let profile_config: crate::profile::ProfileConfig =
            toml::from_str("[[profiles]]\nname = \"git\"\n[profiles.aliases]\ngs = \"git status\"")
                .unwrap();
        let model = AppModel::new(config, profile_config);

        let args = ExportArgs {
            scope: crate::cli::ScopeArgs {
                local: false,
                global: false,
                profile: vec!["git".into()],
                all: false,
            },
            base64: false,
        };

        let output = handle_export(&model, &args).unwrap();
        assert!(output.contains("[[profiles]]"));
        assert!(output.contains("git"));
        assert!(output.contains("gs"));
    }

    #[test]
    fn test_export_missing_profile_errors() {
        let config = crate::config::Config::default();
        let profile_config = crate::profile::ProfileConfig::default();
        let model = AppModel::new(config, profile_config);

        let args = ExportArgs {
            scope: crate::cli::ScopeArgs {
                local: false,
                global: false,
                profile: vec!["nonexistent".into()],
                all: false,
            },
            base64: false,
        };

        let err = handle_export(&model, &args).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    // ─── apply_import tests ─────────────────────────────────────────

    #[test]
    fn test_apply_import_global() {
        let config = crate::config::Config::default();
        let profile_config = crate::profile::ProfileConfig::default();
        let mut model = AppModel::new(config, profile_config);

        let mut global = AliasSet::default();
        global.insert("ll".into(), crate::TomlAlias::Command("ls -lha".into()));

        let payload = ImportPayload {
            global_aliases: Some(global),
            profiles: vec![],
            local_aliases: None,
        };

        // apply_import calls update() + saves — config save will fail
        // because there's no config dir, but the model mutation should work
        let _ = apply_import(&mut model, payload);
        assert_eq!(model.config.aliases.len(), 1);
    }
}
