# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1](https://github.com/sassman/amoxide-rs/compare/v0.2.0...v0.2.1) - 2026-03-31

### Bug Fixes

- Fix the config dir to be .config/amoxide/

## [0.2.0](https://github.com/sassman/amoxide-rs/compare/amoxide-v0.1.1-beta.1...amoxide-v0.2.0) - 2026-03-29

### Features

- Add cargo-dist for binary distribution ([#37](https://github.com/sassman/amoxide-rs/pull/37))
- Multiple active profiles, replace inheritance ([#35](https://github.com/sassman/amoxide-rs/pull/35))
- Add PowerShell shell support ([#36](https://github.com/sassman/amoxide-rs/pull/36))

## [0.1.1-beta.1](https://github.com/sassman/amoxide-rs/compare/amoxide-v0.1.0...amoxide-v0.1.1-beta.1) - 2026-03-26

### Bug Fixes

- Publish dry-run and docs failures
- Backwards-compatible reload with legacy _AM_PROFILE_ALIASES var
- Wrapper skips reload on am error, add inheritance removal tests
- Reload aliases after profile add/remove, not just set
- Reload now handles global + profile aliases together
- Reload profile aliases after am add/remove (not just -l)
- Wrapper short-form aliases and false-positive -l detection
- Fish wrapper test -a ambiguity with profile -a alias
- Lint
- Fix clippy lints

### Documentation

- Fix links in the readme
- Add crates.io and GPLv3 license badges to all READMEs
- Add shell support disclaimer to all READMEs
- Add crate READMEs for crates.io, cross-link amoxide and amoxide-tui

### Features

- Reload aliases after am-tui exits
- Include shell completions in am init output
- Update profile inheritance via `am profile add --inherits` and `--no-inherits`
- Add am-tui interactive terminal UI for alias management ([#31](https://github.com/sassman/amoxide-rs/pull/31))
- Add insta snapshot tests, fix zsh wrapper, resolve inheritance
- Resolve profile inheritance chain for init and reload
- Add spacer line between aliases and child profiles in tree
- Add global aliases with -g/--global flag
- Add `am status` command for shell setup diagnostics
- Auto-reload project aliases after local add/remove
- Parameterized aliases, shell functions, profile reload
- Profile remove, init help, banner, parameterized alias spec
- Add `am ls` command, release pipeline, project aliases file
- Profile tree display, subcommand verbs, project alias management
- Simplify init mechanism, add project aliases, remove dead code
- Improve saving and restoring of active profile, profiles and aliases
- Implement shell completions as it was in sm previously
- Persisting state like active profile
- A major iteration

### Miscellaneous Tasks

- Release v0.1.1 ([#32](https://github.com/sassman/amoxide-rs/pull/32))
- Turn publish to true for the crates
- Remove legacy _AM_PROFILE_ALIASES fallback in reload

### Refactor

- Remove default profile concept
- Extract shell wrapper/hook scripts to files with include_str!

### Testing

- Add snapshot tests for reload with globals

### Rename

- Shell_scripts → shell_wrappers

## [0.1.1](https://github.com/sassman/amoxide-rs/compare/amoxide-v0.1.0...amoxide-v0.1.1) - 2026-03-25

### Bug Fixes

- Publish dry-run and docs failures
- Backwards-compatible reload with legacy _AM_PROFILE_ALIASES var
- Wrapper skips reload on am error, add inheritance removal tests
- Reload aliases after profile add/remove, not just set
- Reload now handles global + profile aliases together
- Reload profile aliases after am add/remove (not just -l)
- Wrapper short-form aliases and false-positive -l detection
- Fish wrapper test -a ambiguity with profile -a alias
- Lint
- Fix clippy lints

### Documentation

- Fix links in the readme
- Add crates.io and GPLv3 license badges to all READMEs
- Add shell support disclaimer to all READMEs
- Add crate READMEs for crates.io, cross-link amoxide and amoxide-tui

### Features

- Reload aliases after am-tui exits
- Include shell completions in am init output
- Update profile inheritance via `am profile add --inherits` and `--no-inherits`
- Add am-tui interactive terminal UI for alias management ([#31](https://github.com/sassman/amoxide-rs/pull/31))
- Add insta snapshot tests, fix zsh wrapper, resolve inheritance
- Resolve profile inheritance chain for init and reload
- Add spacer line between aliases and child profiles in tree
- Add global aliases with -g/--global flag
- Add `am status` command for shell setup diagnostics
- Auto-reload project aliases after local add/remove
- Parameterized aliases, shell functions, profile reload
- Profile remove, init help, banner, parameterized alias spec
- Add `am ls` command, release pipeline, project aliases file
- Profile tree display, subcommand verbs, project alias management
- Simplify init mechanism, add project aliases, remove dead code
- Improve saving and restoring of active profile, profiles and aliases
- Implement shell completions as it was in sm previously
- Persisting state like active profile
- A major iteration

### Miscellaneous Tasks

- Turn publish to true for the crates
- Remove legacy _AM_PROFILE_ALIASES fallback in reload

### Refactor

- Remove default profile concept
- Extract shell wrapper/hook scripts to files with include_str!

### Testing

- Add snapshot tests for reload with globals

### Rename

- Shell_scripts → shell_wrappers
