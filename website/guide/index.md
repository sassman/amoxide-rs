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

::: code-group

```fish [Fish]
# ~/.config/fish/config.fish
am init fish | source
```

```zsh [Zsh]
# ~/.zshrc
eval "$(am init zsh)"
```

```powershell [PowerShell]
# Add to your PowerShell profile (echo $PROFILE to find it)
(am init powershell) -join "`n" | Invoke-Expression
```

:::

Or use the guided setup: `am setup fish` (or `zsh`, `powershell`).

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
