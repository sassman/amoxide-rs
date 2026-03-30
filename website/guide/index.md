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

That's it — `gs` is now available globally in every shell session, no restart needed. Use `-l` to add a project-specific alias instead:

```sh
am add -l t cargo test
```

This writes to a `.aliases` file in the current directory — loaded automatically when you `cd` in, unloaded when you leave. See [Usage](/usage/) for how global, profile, and project aliases work together.

**4. See your aliases:**

```sh
am ls
# or short: am l
```

## What's Next?

- [Installation](/guide/installation) — all installation methods in detail
- [Shell Setup](/guide/setup) — how the shell integration works
- [Profiles](/usage/profiles) — organize aliases into reusable groups
- [Project Aliases](/usage/project-aliases) — auto-loading `.aliases` files
