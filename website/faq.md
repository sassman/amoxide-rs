# FAQ

## What shells are supported?

| Shell | Status |
|-------|--------|
| Fish | Fully supported and tested |
| PowerShell | Supported and tested (5.1 + 7) |
| Zsh | Supported, not yet tested |
| Bash, Nushell | Not yet implemented |

## How is this different from shell aliases in my dotfiles?

Traditional aliases are static — defined once in your `.bashrc` or `.zshrc`. amoxide adds:

- **Profiles** — named groups you can activate/deactivate without editing files
- **Project aliases** — auto-loaded per directory, like direnv for aliases
- **Parameterized aliases** — template syntax for composable commands
- **Multiple active profiles** — layer aliases with clear precedence

## Where are my aliases stored?

- **Global and profile aliases:** `~/.config/amoxide/config.toml`
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
