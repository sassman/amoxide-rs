# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.10.4](https://github.com/sassman/amoxide-rs/compare/v0.10.3...v0.10.4) - 2026-07-15


### Miscellaneous Tasks

- Update Cargo.lock dependencies


## [0.10.3](https://github.com/sassman/amoxide-rs/compare/v0.10.2...v0.10.3) - 2026-07-15


### Miscellaneous Tasks

- Update Cargo.lock dependencies


## [0.10.2](https://github.com/sassman/amoxide-rs/compare/v0.10.1...v0.10.2) - 2026-06-26


### Miscellaneous Tasks

- Update Cargo.lock dependencies


## [0.10.1](https://github.com/sassman/amoxide-rs/compare/v0.10.0...v0.10.1) - 2026-06-23


### Bug Fixes

- Aliases not loaded on powershell session start ([#146](https://github.com/sassman/amoxide-rs/pull/146)) by @sassman in [#146](https://github.com/sassman/amoxide-rs/pull/146)

  ## Why

  Starting a new PowerShell session left it with no project aliases until
  you `cd`'d elsewhere and back. The hook was wired only to the prompt's
  directory-change branch, so the initial directory was never synced.
  Cross-shell, the related env-var diff logic could also leave stale
  tracking names behind after a sync emptied the alias set — silently
  breaking the *next* sync.

  ## What changed

  - PowerShell cd hook now runs an explicit initial sync on session start,
  matching bash/zsh/fish behaviour.
  - Hook also clears any inherited `_AM_ALIASES` / `_AM_PROJECT_PATH`
  before that initial sync, so a stale value from a parent process can't
  make `am sync` think nothing's missing.
  - Precedence diff now unsets `_AM_ALIASES` / `_AM_SUBCOMMANDS` when a
  change empties them, instead of leaving the old names tracked. Same trap
  could have bitten any shell on an inherited session.
  - Closes #144.

  ---------


## [0.10.0](https://github.com/sassman/amoxide-rs/compare/v0.9.1...v0.10.0) - 2026-06-20


### Features

- Alias and subcommand descriptions ([#131](https://github.com/sassman/amoxide-rs/pull/131)) by @sassman in [#131](https://github.com/sassman/amoxide-rs/pull/131)

  ## Why

  Aliases were just `name → command` pairs. No way to remember what an
  alias does without reading the expansion, especially in a profile with
  30+ entries. This adds an optional human-readable description so `am ls
  -d` and the TUI can show what each alias is for. Closes #110.

  ## What changed

  - `am add -d/--description` writes descriptions for aliases and
  subcommand aliases across global, profile, and project scopes.
  - `am ls -d` and `am la` render descriptions in an aligned `# desc`
  column, falling back to inline when the terminal is narrow.
  - TUI: `d` toggles the description column (shown in the help bar).
  Add/edit flows include a description input on a second line; edits
  round-trip cleanly, including description-only changes.
  - Import/export preserves descriptions. Description-only differences are
  flagged as conflicts so they get reviewed instead of silently
  overwritten.
  - Empty or whitespace descriptions normalize to `None` everywhere via a
  single `normalize_description` helper, applied at the CLI, TUI, and
  serde boundaries.
  - Backwards compatible: the existing subcommand-alias array form
  `"jj:ab" = ["abandon"]` keeps working unchanged (extended via an
  untagged `TomlSubcommand` enum).
  - Docs in `website/` (EN + DE) with `<VersionBadge v="0.10.0" />`.

  ---------

- Am context — snapshot active aliases for AI coding agents ([#125](https://github.com/sassman/amoxide-rs/pull/125)) by @sassman in [#125](https://github.com/sassman/amoxide-rs/pull/125)

  ## Why

  AI coding agents run shell commands in subshells that don't see your
  aliases, so they suggest the long form when you've defined a short one.
  `am context` exports the active set so the agent can read it from a
  session-start hook. Closes #122.

  ## What changed

  - `am context` snapshots the effective alias set (global + active
  profiles + trusted project `.aliases`) as model-friendly markdown, with
  usage rules that teach the agent to expand short forms and match user
  intent against alias commands
  - `am setup claude` wires the snapshot into Claude Code's
  `~/.claude/settings.json` — atomic write, idempotent, token-based hook
  detection
  - Snapshot surfaces an untrusted project `.aliases` notice and asks the
  agent to prompt for `am trust` at session start, so project-specific
  flavor isn't silently skipped
  - Docs: `website/usage/ai-agents.md` in EN + DE; README rewritten as a
  docs pointer-map so amoxide.rs is the single source of truth

  ## Test plan

  - [x] `am setup claude` against a real `~/.claude/settings.json` —
  creates, merges into existing keys, idempotent on re-run
  - [x] New Claude Code session, ask "what aliases do I have?" — agent
  lists them straight from the snapshot
  - [x] Drop an untrusted project `.aliases` in scope → agent surfaces the
  trust ask before acting on any alias
  - [x] `cargo test -p amoxide` (517 unit + 7 setup integration + snapshot
  tests, all green) and `cargo clippy --locked --all-targets -- -D
  warnings` clean

  ---------


## [0.9.1](https://github.com/sassman/amoxide-rs/compare/v0.9.0...v0.9.1) - 2026-05-28


### Miscellaneous Tasks

- Update Cargo.lock dependencies


## [0.9.0](https://github.com/sassman/amoxide-rs/compare/v0.8.1...v0.9.0) - 2026-05-13

### Features

- Add update check with cached background refresh ([#126](https://github.com/sassman/amoxide-rs/pull/126))
- Alias variables with {{name}} placeholders ([#121](https://github.com/sassman/amoxide-rs/pull/121))

### Miscellaneous Tasks

- Small reformatting

## [0.8.1](https://github.com/sassman/amoxide-rs/compare/v0.8.0...v0.8.1) - 2026-04-29

### Miscellaneous Tasks

- Update Cargo.lock dependencies

## [0.8.0](https://github.com/sassman/amoxide-rs/compare/v0.7.0...v0.8.0) - 2026-04-27

### Bug Fixes

- Apply_import was overwriting the user's real config on every cargo test ([#111](https://github.com/sassman/amoxide-rs/pull/111))
- Hook reload local aliases when individual aliases changes ([#107](https://github.com/sassman/amoxide-rs/pull/107))

### Features

- Precedence engine with unified am sync, replaces am hook/reload ([#108](https://github.com/sassman/amoxide-rs/pull/108))
- Add explicit enable/disable flags to am use ([#115](https://github.com/sassman/amoxide-rs/pull/115))
- Make shell logging on navigation events configurable ([#113](https://github.com/sassman/amoxide-rs/pull/113))

### Miscellaneous Tasks

- Bump clap from 4.6.0 to 4.6.1 ([#104](https://github.com/sassman/amoxide-rs/pull/104))

## [0.7.0](https://github.com/sassman/amoxide-rs/compare/v0.6.1...v0.7.0) - 2026-04-18

### Bug Fixes

- Resolve function/alias shadowing for zsh and bash ([#98](https://github.com/sassman/amoxide-rs/pull/98))

### Features

- Add --force flag to reinitialise shell aliases ([#101](https://github.com/sassman/amoxide-rs/pull/101))

## [0.6.1](https://github.com/sassman/amoxide-rs/compare/v0.6.0...v0.6.1) - 2026-04-15

### Miscellaneous Tasks

- Update Cargo.lock dependencies
- Bump clap_complete from 4.6.0 to 4.6.2 ([#94](https://github.com/sassman/amoxide-rs/pull/94))

## [0.6.0](https://github.com/sassman/amoxide-rs/compare/v0.5.0...v0.6.0) - 2026-04-14

### Bug Fixes

- Add --local/-l flag to am remove ([#90](https://github.com/sassman/amoxide-rs/pull/90))
- Regenerate subcommand wrapper when new entry added to existing program ([#89](https://github.com/sassman/amoxide-rs/pull/89))

### Features

- Add use_abbr config option for fish abbreviations ([#92](https://github.com/sassman/amoxide-rs/pull/92))

## [0.5.0](https://github.com/sassman/amoxide-rs/compare/v0.4.0...v0.5.0) - 2026-04-09

### Miscellaneous Tasks

- Release v0.4.0 ([#66](https://github.com/sassman/amoxide-rs/pull/66))

## [0.4.0](https://github.com/sassman/amoxide-rs/compare/v0.3.0...v0.4.0) - 2026-04-05

### Miscellaneous Tasks

- Release v0.4.0 ([#64](https://github.com/sassman/amoxide-rs/pull/64))

## [0.3.0](https://github.com/sassman/amoxide-rs/compare/v0.2.1...v0.3.0) - 2026-04-02

### Documentation

- Align README install sections with project website ([#49](https://github.com/sassman/amoxide-rs/pull/49))

## [0.2.1](https://github.com/sassman/amoxide-rs/compare/v0.2.0...v0.2.1) - 2026-03-31

### Features

- Add copy-to and edit features ([#38](https://github.com/sassman/amoxide-rs/pull/38))

## [0.2.0](https://github.com/sassman/amoxide-rs/compare/amoxide-tui-v0.1.1-beta.1...amoxide-tui-v0.2.0) - 2026-03-29

### Features

- Add cargo-dist for binary distribution ([#37](https://github.com/sassman/amoxide-rs/pull/37))
- Multiple active profiles, replace inheritance ([#35](https://github.com/sassman/amoxide-rs/pull/35))
- Add PowerShell shell support ([#36](https://github.com/sassman/amoxide-rs/pull/36))

### Miscellaneous Tasks

- Bump version to v0.2.0

## [0.1.1-beta.1](https://github.com/sassman/amoxide-rs/compare/amoxide-tui-v0.1.0...amoxide-tui-v0.1.1-beta.1) - 2026-03-26

### Bug Fixes

- Satisfy cargo publish with specific version
- Add 2-space indent to alias arms under project headers
- Fmt + updated completions
- Global is root, project and profiles branch from it with connectors
- Remove extra 2-col padding from alias arms, align with profile connectors
- Match am l tree structure — global/project standalone, profiles as sibling tree
- Restore own_content_prefix for profiles with children
- Resolve all clippy lints in amoxide-tui

### Documentation

- Fix links in the readme
- Add crates.io and GPLv3 license badges to all READMEs
- Add shell support disclaimer to all READMEs
- Add crate READMEs for crates.io, cross-link amoxide and amoxide-tui

### Features

- `i` key to set/remove profile inheritance
- Add am-tui interactive terminal UI for alias management ([#31](https://github.com/sassman/amoxide-rs/pull/31))

### Miscellaneous Tasks

- Change the version to -beta.1
- Release v0.1.1 ([#32](https://github.com/sassman/amoxide-rs/pull/32))
- Turn publish to true for the crates

### Testing

- Add tree prefix continuity tests, fix TestConfigBuilder multi-alias

## [0.1.1](https://github.com/sassman/amoxide-rs/compare/amoxide-tui-v0.1.0...amoxide-tui-v0.1.1) - 2026-03-25

### Bug Fixes

- Add 2-space indent to alias arms under project headers
- Fmt + updated completions
- Global is root, project and profiles branch from it with connectors
- Remove extra 2-col padding from alias arms, align with profile connectors
- Match am l tree structure — global/project standalone, profiles as sibling tree
- Restore own_content_prefix for profiles with children
- Resolve all clippy lints in amoxide-tui

### Documentation

- Fix links in the readme
- Add crates.io and GPLv3 license badges to all READMEs
- Add shell support disclaimer to all READMEs
- Add crate READMEs for crates.io, cross-link amoxide and amoxide-tui

### Features

- `i` key to set/remove profile inheritance
- Add am-tui interactive terminal UI for alias management ([#31](https://github.com/sassman/amoxide-rs/pull/31))

### Miscellaneous Tasks

- Turn publish to true for the crates

### Testing

- Add tree prefix continuity tests, fix TestConfigBuilder multi-alias
