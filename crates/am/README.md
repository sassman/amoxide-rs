# amoxide

Shell alias manager — manage aliases globally via profiles or per-project via `.aliases` files.

The crate is called `amoxide`, but the binary it installs is simply `am`.

## Install

```shell
cargo install amoxide
```

## Quick Start

```fish
# Add to your shell config (~/.config/fish/config.fish)
am init fish | source
```

```zsh
# Or for zsh (~/.zshrc)
eval "$(am init zsh)"
```

Then:

```shell
am add -g ll "ls -lha"              # global alias (always loaded)
am profile add rust                  # create a profile
am add -p rust t "cargo test"        # profile alias
am add -l b "make build"             # project-local alias (.aliases file)
```

## Features

- **Global aliases** (`-g`) — always loaded, independent of profile
- **Profiles** with inheritance — group aliases by context (rust, git, node)
- **Project aliases** (`.aliases` files) — auto-loaded on `cd`, like direnv for aliases
- **Parameterized aliases** — `{{1}}`, `{{@}}` template syntax for composable commands
- **Shell completions** — included in `am init` output
- **Fish and Zsh** support

## Interactive TUI

For a visual interface to manage aliases and profiles, install the companion crate:

```shell
cargo install amoxide-tui
```

Or launch it directly via `am tui` (if installed).

See [amoxide-tui on crates.io](https://crates.io/crates/amoxide-tui) for details.

## Documentation

Full documentation with examples: [GitHub](https://github.com/d34dl0ck/alias-manager-rs)
