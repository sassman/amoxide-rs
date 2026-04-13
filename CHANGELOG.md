# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-04-13

### 🚀 Features

- *(ci)* Enforce validation before preview for showcase PRs
- *(ci)* Skip release plan for non-code PRs
- *(ci)* Use cargo-binstall instead of building from source
- Brush shell integration (#71)
- Project alias trust model with tamper detection (#79)
- *(am-tui)* Trust-aware project display and UX polish (#82)
- Separate session state into session.toml (#87)
- Subcommand aliases (#81)
- *(ci)* Make sure website deploy happends after release announce

### 🐛 Bug Fixes

- Correct alias count in showcase tiles

### 📚 Documentation

- Improve header title and add current version to the header (#83)

### ⚙️ Miscellaneous Tasks

- Release v0.4.0 (#66)
- Skip build/coverage for community-only PRs
- Release v0.5.0 (#67)

### Community

- Add sassman-rust-essentials (#68)

## [0.4.0] - 2026-04-05

### 🚀 Features

- Add bash shell support (3.2+) (#63)
- Import/export aliases with sharing, URL import, and security scanning (#62)
- Community showcase gallery (#65)

### ⚙️ Miscellaneous Tasks

- Add alternate format for demos in gif
- Release v0.4.0 (#64)

## [0.3.0] - 2026-04-02

### 🚀 Features

- *(am)* Show all platform install options when am-tui is not found
- Add project website (VitePress) (#46)
- *(ci)* Relax the pr-preview condition (#50)
- *(ci)* Enable crates.io trusted publishing on a dedicated environment (#57)

### 🐛 Bug Fixes

- *(am)* Default to global alias when no active profile or local project (#48)
- *(tests)* Make CWD injectable to prevent Windows test race condition (#56)
- *(am)* Setup ignores 'n' answer due to unsafe input handling (#60)
- *(am)* Shell wrappers don't reload after 'profile use' (#61)

### 📚 Documentation

- Align README install sections with project website (#49)

### ⚙️ Miscellaneous Tasks

- *(ci)* Add dependabot for website npm deps (#51)
- Update the GitHub bug template (#59)
- Release v0.3.0 (#47)

## [0.2.1] - 2026-03-31

### 🚀 Features

- *(am-tui)* Add copy-to and edit features (#38)
- Unified release with single tag for all crates

### 🐛 Bug Fixes

- Use GitHub App token for release creation
- Fix the config dir to be .config/amoxide/

### ⚙️ Miscellaneous Tasks

- Cleanup old Makefile used for experiments
- Release v0.2.1 (#45)

## [0.2.0] - 2026-03-29

### 🚀 Features

- Add PowerShell shell support (#36)
- Multiple active profiles, replace inheritance (#35)
- Add cargo-dist for binary distribution (#37)

### 🚜 Refactor

- *(ci)* Extract publish-crate composite action with retry backoff

### ⚙️ Miscellaneous Tasks

- Bump version to v0.2.0
- Release v0.2.0 (#34)

## [0.1.1-beta.1] - 2026-03-26

### 🚀 Features

- Support last command from history for zsh history
- *(ci)* Add a basic ci pipeline
- Add few command line options and refactor
- A major iteration
- Persisting state like active profile
- Implement shell completions as it was in sm previously
- Improve saving and restoring of active profile, profiles and aliases
- Simplify init mechanism, add project aliases, remove dead code
- Profile tree display, subcommand verbs, project alias management
- Add `am ls` command, release pipeline, project aliases file
- Profile remove, init help, banner, parameterized alias spec
- Parameterized aliases, shell functions, profile reload
- Auto-reload project aliases after local add/remove
- Add `am status` command for shell setup diagnostics
- Add global aliases with -g/--global flag
- Add spacer line between aliases and child profiles in tree
- Resolve profile inheritance chain for init and reload
- Add insta snapshot tests, fix zsh wrapper, resolve inheritance
- Add am-tui interactive terminal UI for alias management (#31)
- *(am)* Update profile inheritance via `am profile add --inherits` and `--no-inherits`
- Include shell completions in am init output
- *(am-tui)* `i` key to set/remove profile inheritance
- *(am)* Reload aliases after am-tui exits

### 🐛 Bug Fixes

- Fix clippy lints
- Lint
- Fish wrapper test -a ambiguity with profile -a alias
- Wrapper short-form aliases and false-positive -l detection
- Reload profile aliases after am add/remove (not just -l)
- Reload now handles global + profile aliases together
- Disable sccache for coverage job on Linux
- Reload aliases after profile add/remove, not just set
- Wrapper skips reload on am error, add inheritance removal tests
- Resolve all clippy lints in amoxide-tui
- Backwards-compatible reload with legacy _AM_PROFILE_ALIASES var
- *(am-tui)* Restore own_content_prefix for profiles with children
- *(am-tui)* Match am l tree structure — global/project standalone, profiles as sibling tree
- *(am-tui)* Remove extra 2-col padding from alias arms, align with profile connectors
- *(am-tui)* Global is root, project and profiles branch from it with connectors
- CI failures — remove sccache, fix typos, delete stale sm.fish
- Release binary assets for both am and am-tui
- Remove .cargo/config.toml with macOS-only linker flag
- *(ci)* Fmt + updated completions
- Release pipeline for multi-crate workspace
- Publish dry-run and docs failures
- *(am-tui)* Add 2-space indent to alias arms under project headers
- *(ci)* Satisfy cargo publish with specific version
- Publish dry-run for amoxide-tui uses cargo package instead
- Use --no-verify for amoxide-tui package dry-run
- *(ci)* Skip publish dry-run on tag-triggered builds
- *(ci)* Publish only the crate targeted by the tag

### 🚜 Refactor

- Move into workspace with lib and cli separated crates
- Extract shell wrapper/hook scripts to files with include_str!
- Remove default profile concept

### 📚 Documentation

- Update the readme a bit
- Update the setup code
- Add crate READMEs for crates.io, cross-link amoxide and amoxide-tui
- Add shell support disclaimer to all READMEs
- Add crates.io and GPLv3 license badges to all READMEs
- Center README title
- Some minor adjustments
- Improve README examples — long form commands, ascii annotation
- Rewrite Productivity Tip to follow the actual workflow
- Consolidate README — narrative flow + compact reference
- Refine a bit
- Fix ascii formatting
- Update screenshots
- Fix links in the readme
- Little rewording

### 🧪 Testing

- Add snapshot tests for reload with globals
- *(am-tui)* Add tree prefix continuity tests, fix TestConfigBuilder multi-alias

### ⚙️ Miscellaneous Tasks

- Ignore todo file
- Cleanup ci and stuff
- Add .worktrees/ to gitignore
- Bump sccache-action@v0.0.5 to v0.0.9
- Remove legacy _AM_PROFILE_ALIASES fallback in reload
- Restore sccache (root cause was .cargo/config.toml, not sccache)
- Delete stale sm.* completion files (old crate name)
- Turn publish to true for the crates
- Update lock file
- Release v0.1.1 (#32)
- Change the version to -beta.1
- Release v0.1.1-beta.1 (#33)

### Milestore

- Befor re-pivoting to alias manager only

### Rename

- Shell_scripts → shell_wrappers

<!-- generated by git-cliff -->
