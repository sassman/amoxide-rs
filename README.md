<h1 align="center">amoxide (am) - alias manager oxidized</h1>

<p align="center">
  <img src="assets/banner.png" width="50%" alt="amoxide banner" />
</p>

<p align="center">
  <a href="https://crates.io/crates/amoxide"><img src="https://img.shields.io/crates/v/amoxide.svg" alt="amoxide on crates.io"/></a>
  <a href="https://crates.io/crates/amoxide-tui"><img src="https://img.shields.io/crates/v/amoxide-tui.svg" alt="amoxide-tui on crates.io"/></a>
  <a href="https://github.com/sassman/amoxide-rs/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-GPLv3-blue" alt="license"/></a>
</p>

> If you only have a handful of shell aliases in your dotfiles, you're missing out. amoxide (`am`) lets you define aliases per project, per toolchain, or globally — and loads the right ones automatically when you `cd` into a directory. Think [direnv](https://direnv.net), but for aliases.

```sh
cd ~/my-rust-project
# cargo test is now: t
# cargo clippy <with-a-lot-of-options> is now: l

cd ~/my-node-project
# same aliases, different commands, loaded automatically
# the rust ones are gone — no pollution
```

## Screenshots

- `am tui` launches the tui to navigate, select, move, add, and delete aliases visually:

<p align="center">
  <img src="assets/am-tui-2.png" alt="am tui" />
</p>

- `am ls` the regular cli

<p align="center">
  <img src="assets/am-ls.png" alt="am ls" />
</p>

## Installation

### Homebrew (macOS and Linux)

```sh
brew install sassman/tap/amoxide sassman/tap/amoxide-tui
```

### Shell Script (macOS and Linux)

```sh
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.sh | sh
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.sh | sh
```

### PowerShell (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.ps1 | iex"
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.ps1 | iex"
```

### Cargo (pre-built)

```sh
cargo binstall amoxide amoxide-tui
```

### Cargo (from source)

```sh
cargo install amoxide amoxide-tui
```

The crate is called `amoxide`, but the binary it installs is simply `am` (short for amoxide).

## Shell Support

| Shell | Status |
|-------|--------|
| Fish | Fully supported and tested |
| PowerShell | Supported and tested (5.1 + 7) |
| Zsh | Supported, not yet tested |
| Bash, Nushell | Not yet implemented |

## Quick Setup

```sh
am setup fish          # or: zsh, powershell
```

Then add your first alias:

```sh
am add -l t cargo test     # project-local alias
am add -p rust t cargo test # profile alias
am add -g ll ls -lha        # global alias
```

## Documentation

Full documentation — usage guides, profiles, project aliases, parameterized aliases, and more:

**[amoxide.rs](https://amoxide.rs)**
