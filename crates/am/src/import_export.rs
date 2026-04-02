use std::io::{BufRead, Read, Write};
use std::time::Duration;

use crate::alias::MergeResult;
use crate::cli::{ExportArgs, ImportArgs, ShareArgs};
use crate::effects::Effect;
use crate::exchange::{
    base64_decode, base64_encode, parse_import, render_import_summary, render_suspicious_warning,
    scan_suspicious, ExportAll, ImportPayload, Scope, SanitizedName,
};
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
        String::from(r#"Share your aliases with others via a pastebin service.

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
"#)
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
    // Phase 1: Determine input source
    let input = match &args.url {
        Some(url) if url.starts_with("http://") || url.starts_with("https://") => {
            eprintln!("Fetching {url}...");
            fetch_url(url)?
        }
        Some(value) => {
            anyhow::bail!("expected a http(s) URL, got: {value}");
        }
        None => {
            let mut buf = String::new();
            std::io::stdin().lock().read_to_string(&mut buf)?;
            buf
        }
    };

    if input.trim().is_empty() {
        anyhow::bail!("no input received");
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
            // --yes --trust: user explicitly accepts the risk
            let n = findings.len();
            let label = if n == 1 { "entry" } else { "entries" };
            eprintln!("WARNING: {n} suspicious {label} found, proceeding due to --trust");
        } else if args.yes {
            // --yes without --trust: refuse to auto-accept dangerous input
            eprint!("{}", render_suspicious_warning(&findings));
            anyhow::bail!(
                "refusing to import: suspicious characters detected. \
                 Use --yes --trust to override."
            );
        } else {
            // Interactive: show warning and require explicit "YES" confirmation
            eprint!("{}", render_suspicious_warning(&findings));
            eprint!("Type YES to import anyway, or anything else to abort: ");
            std::io::stderr().flush()?;
            let mut confirmation = String::new();
            std::io::stdin().lock().read_line(&mut confirmation)?;
            if confirmation.trim() != "YES" {
                anyhow::bail!("import aborted by user");
            }
        }
    }

    // Phase 2: Resolve conflicts + Phase 3: Apply
    if args.scope.local || args.scope.global || !args.scope.profile.is_empty() {
        import_with_override(model, args, &parsed)?;
    } else {
        import_auto_route(model, args, &parsed)?;
    }

    Ok(())
}

fn import_auto_route(
    model: &mut AppModel,
    args: &ImportArgs,
    parsed: &ExportAll,
) -> anyhow::Result<()> {
    let mut payload = ImportPayload::default();

    if !parsed.global_aliases.is_empty() {
        let merge = model.config.aliases.merge_check(&parsed.global_aliases);
        if let Some(accepted) = prompt_merge(&Scope::Global, &merge, args.yes)? {
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
        if let Some(accepted) = prompt_merge(&scope, &merge, args.yes)? {
            payload.profiles.push(Profile {
                name: profile.name.clone(),
                aliases: accepted,
            });
        }
    }

    if !parsed.local_aliases.is_empty() {
        let existing = model.project_alias_set();
        let merge = existing.merge_check(&parsed.local_aliases);
        if let Some(accepted) = prompt_merge(&Scope::Local, &merge, args.yes)? {
            payload.local_aliases = Some(accepted);
        }
    }

    apply_import(model, payload)
}

fn import_with_override(
    model: &mut AppModel,
    args: &ImportArgs,
    parsed: &ExportAll,
) -> anyhow::Result<()> {
    let flattened = parsed.flatten();
    let mut payload = ImportPayload::default();

    if args.scope.global {
        let merge = model.config.aliases.merge_check(&flattened);
        if let Some(accepted) = prompt_merge(&Scope::Global, &merge, args.yes)? {
            payload.global_aliases = Some(accepted);
        }
    }

    for name in &args.scope.profile {
        let existing_aliases = model
            .profile_config()
            .get_profile_by_name(name)
            .map(|p| p.aliases.clone())
            .unwrap_or_default();
        let merge = existing_aliases.merge_check(&flattened);
        let scope = Scope::Profile(SanitizedName::new(name));
        if let Some(accepted) = prompt_merge(&scope, &merge, args.yes)? {
            payload.profiles.push(Profile {
                name: name.clone(),
                aliases: accepted,
            });
        }
    }

    if args.scope.local {
        let existing = model.project_alias_set();
        let merge = existing.merge_check(&flattened);
        if let Some(accepted) = prompt_merge(&Scope::Local, &merge, args.yes)? {
            payload.local_aliases = Some(accepted);
        }
    }

    apply_import(model, payload)
}

fn prompt_merge(
    scope: &Scope,
    merge: &MergeResult,
    auto_yes: bool,
) -> anyhow::Result<Option<AliasSet>> {
    if merge.new_aliases.is_empty() && merge.conflicts.is_empty() {
        eprintln!("Nothing new to import into \"{scope}\"");
        return Ok(None);
    }

    eprint!("{}", render_import_summary(&scope.to_string(), merge));

    // Ask to merge
    if !auto_yes {
        eprint!("Merge into \"{scope}\"? [Y/n] ");
        std::io::stderr().flush()?;
        let mut input = String::new();
        std::io::stdin().lock().read_line(&mut input)?;
        if matches!(input.trim().to_lowercase().as_str(), "n" | "no") {
            eprintln!("Skipped \"{scope}\"");
            return Ok(None);
        }
    }

    // Start with new aliases
    let mut accepted = merge.new_aliases.clone();

    // Ask about overwrites
    if !merge.conflicts.is_empty() {
        let apply_overwrites = if auto_yes {
            true
        } else {
            eprint!(
                "Apply {} overwrite{}? [y/N] ",
                merge.conflicts.len(),
                if merge.conflicts.len() == 1 { "" } else { "s" }
            );
            std::io::stderr().flush()?;
            let mut input = String::new();
            std::io::stdin().lock().read_line(&mut input)?;
            matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
        };

        if apply_overwrites {
            for conflict in &merge.conflicts {
                accepted.insert(conflict.name.clone(), conflict.incoming.clone());
            }
        }

        let imported = accepted.len();
        let skipped = if apply_overwrites {
            0
        } else {
            merge.conflicts.len()
        };
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

/// Validate a URL argument for import.
/// Only http:// and https:// URLs are accepted.
pub fn validate_url_arg(value: &str) -> anyhow::Result<()> {
    if value.starts_with("http://") || value.starts_with("https://") {
        Ok(())
    } else {
        anyhow::bail!("expected a http(s) URL, got: {value}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_arg_https() {
        assert!(validate_url_arg("https://example.com/aliases.toml").is_ok());
    }

    #[test]
    fn test_validate_url_arg_http() {
        assert!(validate_url_arg("http://example.com/aliases.toml").is_ok());
    }

    #[test]
    fn test_validate_url_arg_ftp_rejected() {
        let err = validate_url_arg("ftp://example.com/file").unwrap_err();
        assert!(err.to_string().contains("expected a http(s) URL"));
    }

    #[test]
    fn test_validate_url_arg_file_rejected() {
        let err = validate_url_arg("file:///etc/passwd").unwrap_err();
        assert!(err.to_string().contains("expected a http(s) URL"));
    }

    #[test]
    fn test_validate_url_arg_no_scheme_rejected() {
        let err = validate_url_arg("just-a-string").unwrap_err();
        assert!(err.to_string().contains("expected a http(s) URL"));
    }

    #[test]
    fn test_validate_url_arg_path_rejected() {
        let err = validate_url_arg("/tmp/aliases.toml").unwrap_err();
        assert!(err.to_string().contains("expected a http(s) URL"));
    }

    #[test]
    fn test_fetch_url_invalid_host_errors() {
        // This should fail with a connection error, not panic
        let result = fetch_url("http://this-host-does-not-exist.invalid/aliases.toml");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed to fetch"));
    }
}
