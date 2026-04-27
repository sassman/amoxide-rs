# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


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
