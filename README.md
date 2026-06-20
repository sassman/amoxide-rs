<h1 align="center">amoxide (am) - alias manager oxidized</h1>

<p align="center">
  <img src="assets/banner.png" width="80%" alt="amoxide banner" />
</p>

<p align="center">
  <a href="https://crates.io/crates/amoxide"><img src="https://img.shields.io/crates/v/amoxide.svg" alt="amoxide on crates.io"/></a>
  <a href="https://crates.io/crates/amoxide-tui"><img src="https://img.shields.io/crates/v/amoxide-tui.svg" alt="amoxide-tui on crates.io"/></a>
  <a href="https://github.com/sassman/amoxide-rs/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-GPLv3-blue" alt="license"/></a>
</p>

> [!TIP]
> **Full documentation lives at [amoxide.rs](https://amoxide.rs).** Guides, usage references, and advanced topics — kept in sync with each release. This README is a pointer map; the docs are the source of truth.

> If you only have a handful of shell aliases in your dotfiles, you're missing out. amoxide (`am`) lets you define aliases per project, per toolchain, or globally — and loads the right ones automatically when you `cd` into a directory. Think [direnv](https://direnv.net), but for aliases.

## Screenshots

`am tui` — navigate, select, move, add, and delete aliases visually:

<p align="center">
  <img src="assets/am-tui-2.png" alt="am tui" />
</p>

`am ls` — the regular cli:

<p align="center">
  <img src="assets/am-ls.png" alt="am ls" />
</p>

## Install

```sh
brew install sassman/tap/amoxide sassman/tap/amoxide-tui
```

or from source via cargo:

```sh
cargo install amoxide amoxide-tui
```

Full options (shell script, PowerShell, `cargo binstall` for pre-built binaries) → **[amoxide.rs/guide/installation](https://amoxide.rs/guide/installation/)**

The crate is `amoxide`; the binary it installs is `am`.

## Docs

Everything lives on **[amoxide.rs](https://amoxide.rs)**. Jump to a section:

- [Getting Started](https://amoxide.rs/guide/) — install, shell setup, first aliases
- [Shell Setup](https://amoxide.rs/guide/setup/) — wire `am` into fish / zsh / bash / brush / powershell
- [Global Aliases](https://amoxide.rs/usage/global/) — always-on aliases across every shell
- [Profiles](https://amoxide.rs/usage/profiles/) — named groups you toggle on and off
- [Project Aliases](https://amoxide.rs/usage/project-aliases/) — auto-load `.aliases` per directory
- [Subcommand Aliases](https://amoxide.rs/usage/subcommand-aliases/) — short forms for `git cm`, `kubectl gp`, etc.
- [Variables](https://amoxide.rs/usage/variables/) — `{{name}}` placeholders, scope-local values
- [Parameterized Aliases](https://amoxide.rs/advanced/parameterized-aliases/) — `{{1}}` positional templates
- [Composing Aliases](https://amoxide.rs/advanced/composing-aliases/) — layer aliases on top of aliases
- [AI Agents](https://amoxide.rs/usage/ai-agents/) — teach Claude Code your active aliases via `am setup claude`
- [Sharing](https://amoxide.rs/usage/sharing/) — export / import / paste-share alias sets
- [Config Files](https://amoxide.rs/advanced/config-files/) — TOML format and file locations
- [FAQ](https://amoxide.rs/faq/)

## License

GPLv3 — see [LICENSE](LICENSE).
