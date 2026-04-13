# amoxide

<p align="center">
  <a href="https://crates.io/crates/amoxide"><img src="https://img.shields.io/crates/v/amoxide.svg" alt="crates.io"/></a>
  <a href="https://github.com/sassman/amoxide-rs/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-GPLv3-blue" alt="license"/></a>
</p>

Shell alias manager — manage aliases globally, via profiles or per-project. Think [direnv](https://direnv.net), but for aliases.

The crate is called `amoxide`, but the binary it installs is simply `am`.

## Install

```sh
cargo install amoxide
# or pre-built:
cargo binstall amoxide
```

Also available via [Homebrew, Shell Script, and PowerShell](https://amoxide.rs/guide/installation).

## Features

- **[Global aliases](https://amoxide.rs/usage/global)** — always loaded, independent of profile
- **[Profiles](https://amoxide.rs/usage/profiles)** — group aliases by context (rust, git, node), activate several at once
- **[Project aliases](https://amoxide.rs/usage/project-aliases)** — `.aliases` files, auto-loaded on `cd`
- **[Parameterized aliases](https://amoxide.rs/advanced/parameterized-aliases)** — `{{1}}`, `{{@}}` template syntax
- **[Subcommand aliases](https://amoxide.rs/advanced/subcommand-aliases)** — map short tokens to subcommand expansions (`jj:ab` → `jj abandon`, multi-level: `jj:b:l` → `jj branch list`)
- **[Interactive TUI](https://crates.io/crates/amoxide-tui)** — visual alias management (`cargo install amoxide-tui`)

## Documentation

Getting started, shell setup, and full usage guide: **[amoxide.rs](https://amoxide.rs)**
