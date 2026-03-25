# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

### Testing

- Add tree prefix continuity tests, fix TestConfigBuilder multi-alias
