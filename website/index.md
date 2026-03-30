---
layout: home

hero:
  name: "amoxide"
  text: "Shell aliases that follow your context"
  tagline: Like direnv, but for aliases. Define aliases per project, per toolchain, or globally — and load the right ones automatically.
  image:
    src: /logo.svg
    alt: amoxide logo
  actions:
    - theme: brand
      text: Get Started
      link: /guide/
    - theme: alt
      text: GitHub
      link: https://github.com/sassman/amoxide-rs

features:
  - title: Context-Aware
    details: Project aliases load automatically when you cd into a directory and unload when you leave. No manual switching.
    link: /usage/project-aliases
    linkText: Learn more
  - title: Profiles
    details: Group aliases by context — rust, git, node. Activate multiple profiles simultaneously with layered precedence.
    link: /usage/profiles
    linkText: Learn more
  - title: Parameterized Aliases
    details: Use positional and catch-all argument templates to compose powerful, reusable aliases with argument forwarding.
    link: /advanced/parameterized-aliases
    linkText: Learn more
  - title: Interactive TUI
    details: Browse, add, move, and manage aliases visually with the am-tui companion. Works alongside the CLI.
    link: /guide/installation
    linkText: Install am-tui
---

## Install

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
```

:::

## See It in Action

### Interactive TUI

Browse, add, move, and delete aliases visually with `am tui`:

![am tui screenshot](/am-tui.png)

### CLI Listing

See your layered alias hierarchy at a glance with `am ls`:

![am ls screenshot](/am-ls.png)
