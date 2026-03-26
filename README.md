<h1 align="center">amoxide (am) - alias manager oxidized</h1>

<p align="center">
  <img src="assets/banner.png" width="50%" alt="amoxide banner" />
</p>

<p align="center">
  <a href="https://crates.io/crates/amoxide"><img src="https://img.shields.io/crates/v/amoxide.svg" alt="amoxide on crates.io"/></a>
  <a href="https://crates.io/crates/amoxide-tui"><img src="https://img.shields.io/crates/v/amoxide-tui.svg" alt="amoxide-tui on crates.io"/></a>
  <a href="https://github.com/sassman/amoxide-rs/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-GPLv3-blue" alt="license"/></a>
</p>

> If you only have a handful of shell aliases in your dotfiles, you're missing out. amoxide (`am`) lets you define aliases per project, per toolchain, or globally — and loads the right ones automatically when you `cd` into a directory. Think [direnv](https://direnv.net), but for aliases.

```sh
cd ~/my-rust-project
# cargo test is now: t
# cargo clippy <with-a-lot-of-options> is now: l

cd ~/my-node-project
# same aliases, different commands, loaded automatically
# the rust ones are gone — no pollution
```

## Subcommands

| Command | Description | |
|---------|-------------|---|
| [`am add`](#adding-and-removing-aliases) | Add a new alias (global, profile, or local) | |
| [`am remove`](#adding-and-removing-aliases) | Remove an alias | |
| [`am profile`](#profiles) | Manage profiles (add, use, remove, list) | |
| `am ls` | List all profiles and project aliases | |
| [`am init`](#setup) | Print shell init code | |
| [`am status`](#setup) | Check if the shell is set up correctly | |
| `am tui` | Interactive TUI for managing aliases and profiles | *separate install* |
| `am hook` | Called by the cd hook (internal) | |

## Productivity Tip / TL;DR (Opinionated)

I personally start with **project-local aliases**, so I can be super lazy in the context I'm working in.

For example in this project I have:

- installing the binary by `i` does a `cargo install --path crates/am`
- running tests by `t` does a `cargo test`
- running lint checks by `l` does a `cargo clippy --all-targets --all-features -- -D warning`

**Step 1** Add local / project aliases with `-l`:

```sh
am add -l t cargo test
#  ^^^ ^^ ^ ^--------^
#   |   | |       |
#   |   | |       +---- the command to alias
#   |   | +---- the alias name
#   |   +---- local, writes to .aliases in the current directory
#   +---- adding an alias

am add -l l cargo clippy --all-targets --all-features -- -D warning
am add -l i cargo install --path crates/am
```

These *local* aliases live in a `.aliases` file in the project root and are loaded/unloaded automatically on `cd`.

**Step 2 - Refactor:** After a while I notice `t` and `l` are the same in every rust project. Time to extract them into a reusable **profile** — a named collection of aliases you can activate anywhere:

```sh
am profile add rust
am add -p rust t cargo test
#      ^-----^
#         |
#         + ---- we add the alias to the profile "rust"

am add -p rust l cargo clippy --all-targets --all-features -- -D warning

# activate it
am profile use rust
```

> :bulb: Tip: `am tui` allows to move the aliases from project to profile level (by select and `m`)

Now `t` and `l` are available everywhere (not just this project). The project `.aliases` keeps only project-specific ones like `i`.

**Step 3 — use multiple profiles.** I also want git aliases everywhere. Just activate both:

```sh
# git profile with a signing commit alias
am profile add git
am add -p git gm "git commit -S --signoff -m"

# activate both — order matters: the last one activated wins on conflicts
am profile use git
am profile use rust
```

Now I have git aliases and rust aliases loaded at the same time. If both profiles had an alias with the same name, the last-activated one (rust) would take precedence.

> :bulb: Tip: all verbs have short forms to save typing, e.g. `am a -l t cargo test` or `am p a rust`.

> :bulb: Tip: the subcommand `am tui` (`cargo install amoxide-tui`) simplifies this a lot.

## Screenshots

- `am tui` launches the tui to navigate, select, move, add, and delete aliases visually:

<p align="center">
  <img src="assets/am-tui-2.png" alt="am tui" />
</p>

- `am ls` the regular cli

<p align="center">
  <img src="assets/am-ls.png" alt="am ls" />
</p>

## Installation

```shell
cargo install amoxide          # installs the `am` binary
cargo install amoxide-tui      # installs the `am-tui` interactive interface (optional)
```

The crate is called `amoxide`, but the binary it installs is simply `am` (short for amoxide).

## Shell Support

| Shell | Status |
|-------|--------|
| Fish | Fully supported and tested |
| PowerShell | Supported and tested (5.1 + 7) |
| Zsh | Supported, not yet tested |
| Bash, Nushell | Not yet implemented |

## Setup

The easiest way — guided setup:

```shell
am setup fish          # or: zsh, powershell
```

This detects your profile file, shows exactly what it will add, and asks for confirmation.

Or add manually to your shell config:

```fish
# ~/.config/fish/config.fish
am init fish | source
```

```zsh
# ~/.zshrc
eval "$(am init zsh)"
```

```powershell
# add to your PowerShell profile (echo $PROFILE to find it)
(am init powershell) -join "`n" | Invoke-Expression
```

This does two things:
1. Loads aliases from your active profiles into the current shell
2. Installs a cd hook that automatically loads/unloads project aliases (from `.aliases` files) when you change directories

To verify the setup is correct, run:

```shell
am status
```

## Usage by Example

### Adding and removing aliases

```shell
$ am add gs git status         # add to the active profile
$ am add -p rust ct cargo test # add to a specific profile
$ am add -l t ./x.py test     # add to the current project (.aliases)
$ am remove gs                 # remove from active profile
$ am r gs                      # short form
$ am remove -p rust ct         # remove from a specific profile
```

Short form works too:

```shell
$ am a l ls -lha
#    ^ ^ ^-----^
#    | |       |
#    | |       +---- this is alias command `ls -lha`
#    | +---- this is the alias name `l`
#    +---- this is the verb `add`
```

### Parameterized aliases

Aliases can use `{{1}}`, `{{2}}`, ... for positional arguments and `{{@}}` for all arguments:

```shell
# Compose aliases with argument templates
am add -p git cm "git commit -S --signoff -m {{@}}"
am add -p git-conventional cmf "cm feat: {{@}}"

cmf my feature description
# → cm feat: my feature description
# → git commit -S --signoff -m feat: my feature description

# Positional arguments
am add greet "echo Hello {{1}}, welcome to {{2}}"
greet Alice Wonderland
# → echo Hello Alice, welcome to Wonderland
```

If your command literally contains `{{N}}` (e.g., in awk), use `--raw` to disable template detection:

```shell
am add --raw my-awk "awk '{print {{1}}}'"
```

### Profiles

Profiles let you group aliases by context (e.g., `rust`, `node`, `git`). You can activate multiple profiles simultaneously — think of it as layers where later-activated profiles override earlier ones for conflicting alias names:

```shell
# Add a profile
$ am profile add rust
$ am p a rust                  # short form

# Activate a profile (adds it on top of the current stack)
$ am profile use rust
$ am p u rust                  # short form

# Activate at a specific position (1 = base layer, higher = overrides lower)
$ am profile use git -n 1

# Remove a profile (asks for confirmation if it has aliases)
$ am profile remove rust
$ am p r rust -f               # skip confirmation

# Add aliases to a specific profile
$ am add -p rust ct "cargo test"
$ am add -p rust cb "cargo build"

# List all profiles, aliases, and active project aliases
$ am profile                   # default action
$ am profile list              # explicit
$ am l                         # shortest form
```

Active profiles' aliases are loaded on every shell start via `am init`.

Listing shows active profiles connected by a tree trunk, inactive profiles below:

```
🌐 global
│ helo → echo hello world global
│
├─● git (active: 1)
│ gm → git commit -S --signoff -m
│
├─● rust (active: 2)
│ ct → cargo test
│ cb → cargo build
│
╰─📁 project aliases (.aliases)
  t → ./x.py test
  b → ./x.py build

○ node
  nr → npm run
```

In this example, `git` is the base layer (active: 1) and `rust` sits on top (active: 2). If both had an alias named `t`, rust's version would win.

### Project aliases (`.aliases` file)

You can add project-local aliases with the `-l`/`--local` flag:

```shell
$ cd ~/my-project
$ am add -l t "./x.py test"   # writes to .aliases in current directory
$ am add -l b "./x.py build"
```

If no `.aliases` file exists, one is created in the current directory. If a `.aliases` already exists further up the directory tree, you'll be asked whether you meant to add to that one instead.

You can also create or edit the `.aliases` file directly:

```toml
# /path/to/my-project/.aliases
[aliases]
t = "./x.py test"
b = "./x.py build"
```

These aliases are automatically loaded when you `cd` into the project (or any subdirectory) and unloaded when you leave.

Under the hood, `am init` installs a cd hook that calls `am hook <shell>` on every directory change. The hook walks up from the current directory looking for a `.aliases` file (stopping before `$HOME`), unloads any previously active project aliases, and loads the new ones.
