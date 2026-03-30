# Getting Started

amoxide (`am`) is a shell alias manager that brings [direnv](https://direnv.net)-like functionality to aliases. Instead of managing shell aliases in static dotfiles, define them per project, per toolchain, or globally — and the right ones load automatically.

## Quick Start

**1. Install amoxide:**

::: code-group

```sh [Homebrew]
brew install sassman/tap/amoxide sassman/tap/amoxide-tui
```

```sh [Shell Script]
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.sh | sh
```

```powershell [PowerShell]
irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.ps1 | iex
```

```sh [Cargo]
cargo install amoxide amoxide-tui
# or, without compiling from source:
cargo binstall amoxide amoxide-tui
```

:::

**2. Set up your shell:**

```sh
am setup fish          # or: zsh, powershell
```

This detects your profile file, shows exactly what it will add, and asks for confirmation. See [Shell Setup](/guide/setup) for the manual approach.

**3. Add your first alias:**

```sh
am add gs git status
```

**4. Verify:**

```sh
am status
```

## What's Next?

- [Installation](/guide/installation) — all installation methods in detail
- [Shell Setup](/guide/setup) — how the shell integration works
- [Profiles](/config/profiles) — organize aliases into reusable groups
- [Project Aliases](/config/project-aliases) — auto-loading `.aliases` files
