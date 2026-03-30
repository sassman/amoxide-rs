# Installation

## Package Managers

::: code-group

```sh [Homebrew (macOS/Linux)]
brew install sassman/tap/amoxide sassman/tap/amoxide-tui
```

```sh [Shell Script (macOS/Linux)]
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.sh | sh
```

```powershell [PowerShell (Windows)]
irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.ps1 | iex
```

```sh [Cargo (pre-built)]
cargo binstall amoxide amoxide-tui
```

```sh [Cargo (source)]
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
| Bash, Nushell | Not yet implemented |

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
| `am tui` | Interactive TUI for managing aliases and profiles |
| `am hook` | Called by the cd hook (internal) |

::: tip
All verbs have short forms: `am a` for add, `am r` for remove, `am p` for profile, `am l` for ls.
:::
