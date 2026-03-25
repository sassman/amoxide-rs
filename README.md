<h1 align="center">amoxide (am) - alias manager oxidized</h1>

<p align="center">
  <img src="assets/banner.png" width="50%" alt="amoxide banner" />
</p>

<p align="center">
  <a href="https://crates.io/crates/amoxide"><img src="https://img.shields.io/crates/v/amoxide.svg" alt="amoxide on crates.io"/></a>
  <a href="https://crates.io/crates/amoxide-tui"><img src="https://img.shields.io/crates/v/amoxide-tui.svg" alt="amoxide-tui on crates.io"/></a>
  <a href="https://github.com/sassman/amoxide-rs/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-GPLv3-blue" alt="license"/></a>
</p>

> Manage shell aliases — globally, by profile, or per-project. Like direnv, but for aliases.

<p align="center">
  <img src="assets/am-tui-2.png" alt="am tui" />
</p>

<p align="center">
  <img src="assets/am-ls.png" alt="am ls" />
</p>

## Installation

```shell
cargo install amoxide          # the `am` CLI
cargo install amoxide-tui      # interactive TUI (optional)
```

| Shell | Status |
|-------|--------|
| Fish | Fully supported and tested |
| Zsh | Supported, not yet tested |
| Bash, Nushell, PowerShell | Not yet implemented |

## Setup

```fish
# ~/.config/fish/config.fish
am init fish | source
```

```zsh
# ~/.zshrc
eval "$(am init zsh)"
```

Verify with `am status`.

## How I Use It

**1. Start local.** Add project aliases for things I type often:

```sh
am add -l t cargo test
#  ^^^ ^^ ^ ^--------^
#   |   | |       |
#   |   | |       +---- the command
#   |   | +---- alias name
#   |   +---- local (.aliases file)
#   +---- add

am add -l l cargo clippy --all-targets --all-features -- -D warning
am add -l i cargo install --path crates/am
```

These live in `.aliases` and load/unload on `cd`.

**2. Extract a profile.** When `t` and `l` repeat across projects:

```sh
am profile add rust
am add -p rust t cargo test
am add -p rust l cargo clippy --all-targets --all-features -- -D warning
am profile set rust
```

Now available everywhere. Project `.aliases` keeps only `i`.

**3. Compose with inheritance.**

```sh
am profile add git
am add -p git gm "git commit -S --signoff -m"

am profile add git-conventional --inherits git
am add -p git-conventional gmf "gm feat: {{@}}"

gmf "my feature"
# → git commit -S --signoff -m feat: my feature
```

> Tip: short forms save typing — `am a -l t cargo test`, `am p a rust`, `am p s rust`.

> `am tui` makes all of this visual — `cargo install amoxide-tui`.

## Reference

### Aliases

```sh
am add gs git status              # active profile
am add -g ll "ls -lha"            # global (always loaded)
am add -p rust t "cargo test"     # specific profile
am add -l b "make build"          # project-local (.aliases)
am remove gs                      # remove
am remove -g ll                   # remove global
```

### Parameterized aliases

`{{@}}` = all args, `{{1}}`/`{{2}}` = positional. Use `--raw` to disable detection.

```sh
am add gm "git commit -S --signoff -m {{@}}"
am add greet "echo Hello {{1}}, welcome to {{2}}"
am add --raw my-awk "awk '{print {{1}}}'"
```

### Profiles

```sh
am profile add rust                    # create
am profile add rust --inherits git     # with inheritance
am profile set rust                    # activate
am profile remove rust                 # remove
am profile                             # list (default)
am ls                                  # shortcut
```

### Project aliases

```sh
am add -l t "cargo test"               # add to .aliases
```

Or edit `.aliases` directly:

```toml
[aliases]
t = "cargo test"
b = "cargo build"
```

Auto-loaded on `cd`, unloaded on leave.

### Listing

```
🌐 global
  ll → ls -lha

○ git
│ gm → git commit -S --signoff -m
│
╰─● rust (active)
    t → cargo test
    l → cargo clippy ...

📁 project aliases (.aliases)
  i → cargo install --path crates/am
```
