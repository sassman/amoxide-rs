# Chocolatey packaging

Windows distribution for [amoxide](https://amoxide.rs) via the [Chocolatey community repository](https://community.chocolatey.org).

Publishing is automated in `.github/workflows/release.yml` (`publish-chocolatey` job); this directory is what that job packs and pushes.

## Layout

- `amoxide/` — `amoxide` package (the `am` CLI).
- `amoxide-tui/` — `amoxide-tui` package (the `am-tui` visual companion).
- `scripts/pack-and-push.ps1` — release-time orchestrator, invoked by CI.
- `scripts/pack-and-push.lib.ps1` — pure helper functions, Pester-tested.
- `scripts/tests/` — Pester 5 test suite.
- `.staging/` — build output, gitignored.

## Local test (dry-run pack)

Requires `pwsh` (macOS/Linux/Windows) and `git`. Doesn't require `choco` for the pack step to fail cleanly; you'll see the substituted nupkg contents in staging even if `choco pack` isn't available.

```fish
# fetch the tag you want to test against
git fetch o refs/tags/v0.11.0:refs/tags/v0.11.0 --no-tags

# dry-run pack (no push)
pwsh ./packaging/choco/scripts/pack-and-push.ps1 -Tag v0.11.0 -DryRun
```

The tag must be a real published release — the script curls `.sha256` sidecars from the corresponding GitHub Release.

**Note:** dry-run against tags older than the first ARM64-shipping release (v0.10.0 and earlier) will fail on the ARM64 sidecar fetch. That's expected.

## Run Pester tests

Requires Pester 5:

```fish
pwsh -Command "Install-Module Pester -MinimumVersion 5.5.0 -Force -Scope CurrentUser"
pwsh -Command "Invoke-Pester packaging/choco/scripts/tests/pack-and-push.tests.ps1"
```

## Playbook

The full first-time setup, moderation, roll-back, and troubleshooting guide is in `Areas/Cheatsheets/chocolatey.md` in Sven's obsidian vault. Not in-repo because it contains account handles + queue-monitoring URLs that don't belong in a public repo.
