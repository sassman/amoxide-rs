# Installation

## Package Managers

::: code-group

```sh [Homebrew (macOS/Linux)]
brew install sassman/tap/amoxide sassman/tap/amoxide-tui
```

```sh [Shell Script (macOS/Linux)]
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.sh | sh
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.sh | sh
```

```powershell [PowerShell (Windows)]
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.ps1 | iex"
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.ps1 | iex"
```

```sh [Cargo (pre-built)]
cargo binstall amoxide amoxide-tui
```

```sh [Cargo]
cargo install amoxide amoxide-tui
```

:::

The crate is called `amoxide`, but the binary it installs is simply `am`.

::: tip
The TUI companion (`am-tui`) is a separate install. It's optional but recommended for visual alias management.
:::

## Shell Support

| Shell | Status |
|-------|--------|
| Fish | Fully supported and tested |
| PowerShell | Supported and tested (5.1 + 7) |
| Zsh | Supported, not yet tested |
| Bash | Supported (3.2+) |
| Brush | Supported (bash-compatible) |
| Nushell | Not yet implemented |

## Subcommands

| Command | Description |
|---------|-------------|
| `am add` | Add a new alias (global, profile, or local) |
| `am remove` | Remove an alias |
| `am profile` | Manage profiles (add, use, remove, list) |
| `am ls` | List all profiles and project aliases |
| `am init` | Print shell init code |
| `am status` | Check if the shell is set up correctly |
| `am setup` | Guided shell setup |
| `am tui` | Interactive TUI for managing aliases and profiles (*separate install*) |
| `am hook` | Called by the cd hook (internal) |

::: tip
All verbs have short forms: `am a` for add, `am r` for remove, `am p` for profile, `am l` for ls.
:::

## Installing `am-tui`

The `am tui` command requires the TUI companion (`amoxide-tui`) to be installed separately. If it's not installed, `am tui` will show install instructions.

::: code-group

```sh [Homebrew]
brew install sassman/tap/amoxide-tui
```

```sh [Shell Script]
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.sh | sh
```

```powershell [PowerShell]
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.ps1 | iex"
```

```sh [Cargo (pre-built)]
cargo binstall amoxide-tui
```

```sh [Cargo]
cargo install amoxide-tui
```

:::
