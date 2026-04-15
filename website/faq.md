---
head:
  - - script
    - type: application/ld+json
    - >-
      {"@context":"https://schema.org","@type":"FAQPage","mainEntity":[{"@type":"Question","name":"What shells are supported?","acceptedAnswer":{"@type":"Answer","text":"amoxide supports Fish (fully), PowerShell 5.1+7, Zsh, Bash 3.2+, and Brush (bash-compatible). Nushell support is planned."}},{"@type":"Question","name":"How is amoxide different from shell aliases in dotfiles?","acceptedAnswer":{"@type":"Answer","text":"Traditional aliases are static and defined once in .bashrc or .zshrc. amoxide adds: Profiles (named groups you can activate/deactivate), Project aliases (auto-loaded per directory like direnv), Parameterized aliases (template syntax for composable commands), and multiple active profiles with layered precedence."}},{"@type":"Question","name":"Where are my aliases stored?","acceptedAnswer":{"@type":"Answer","text":"Global aliases are stored in ~/.config/amoxide/config.toml. Profile aliases are stored in ~/.config/amoxide/profiles.toml. Project aliases are stored in a .aliases file in the project directory."}},{"@type":"Question","name":"Can I use amoxide alongside my existing shell aliases?","acceptedAnswer":{"@type":"Answer","text":"Yes. amoxide aliases coexist with your shell's native aliases. If there is a name conflict, amoxide's alias takes precedence while it is active."}},{"@type":"Question","name":"What is am-tui?","acceptedAnswer":{"@type":"Answer","text":"am-tui is a separate binary (amoxide-tui) that provides an interactive terminal UI for managing aliases visually. Run it with: am tui. Install via: brew install sassman/tap/amoxide-tui"}}]}
---

# FAQ

## What shells are supported?

| Shell | Status |
|-------|--------|
| Fish | Fully supported and tested |
| PowerShell | Supported and tested (5.1 + 7) |
| Zsh | Supported, not yet tested |
| Bash | Supported (3.2+) |
| Brush | Supported (bash-compatible) |
| Nushell | Not yet implemented |

## How is this different from shell aliases in my dotfiles?

Traditional aliases are static — defined once in your `.bashrc` or `.zshrc`. amoxide adds:

- **Profiles** — named groups you can activate/deactivate without editing files
- **Project aliases** — auto-loaded per directory, like direnv for aliases
- **Parameterized aliases** — template syntax for composable commands
- **Multiple active profiles** — layer aliases with clear precedence

## Where are my aliases stored?

- **Global aliases:** `~/.config/amoxide/config.toml`
- **Profile aliases:** `~/.config/amoxide/profiles.toml`
- **Project aliases:** `.aliases` file in the project directory

## Can I use amoxide alongside my existing shell aliases?

Yes. amoxide aliases coexist with your shell's native aliases. If there's a name conflict, amoxide's alias takes precedence while it's active.

## What is `am-tui`?

A separate binary (`amoxide-tui`) that provides an interactive terminal UI for managing aliases visually. Once installed, it integrates directly into `am`:

```sh
am tui
```

The `am tui` command launches the TUI — no need to remember a separate binary name. Quick install:

```sh
brew install sassman/tap/amoxide-tui
```

See [all installation options](/guide/installation#installing-am-tui) for Shell Script, PowerShell, and Cargo.

:::
