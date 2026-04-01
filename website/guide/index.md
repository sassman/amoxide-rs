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
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.sh | sh
```

```powershell [PowerShell]
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

**2. Set up your shell:**

```sh
am setup fish          # or: zsh, powershell
```

This detects your profile file, shows exactly what it will add, and asks for confirmation. See [Shell Setup](/guide/setup) for the manual approach.

**3. Add your first alias:**

```sh
am add gs git status
```

That's it — `gs` is now available on your active profile, no restart needed. Use `-g` for a global alias (always active) or `-l` for a project-specific one:

```sh
am add -l t cargo test
```

This writes to a `.aliases` file in the current directory — loaded automatically when you `cd` in, unloaded when you leave. See [Usage](/usage/) for how global, profile, and project aliases work together.

**4. See your aliases:**

```sh
am ls
# or short: am l
```

## Recommended Workflow

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

**Step 2 — Refactor:** After a while I notice `t` and `l` are the same in every rust project. Time to extract them into a reusable **profile** — a named collection of aliases you can activate anywhere:

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

::: tip
`am tui` allows to move the aliases from project to profile level (by select and `m`)
:::

Now `t` and `l` are available everywhere (not just this project). The project `.aliases` keeps only project-specific ones like `i`.

**Step 3 — Use multiple profiles.** I also want git aliases everywhere. Just activate both:

```sh
# git profile with a signing commit alias
am profile add git
am add -p git gm "git commit -S --signoff -m"

# activate both — order matters: the last one activated wins on conflicts
am profile use git
am profile use rust
```

Now I have git aliases and rust aliases loaded at the same time. If both profiles had an alias with the same name, the last-activated one (rust) would take precedence.

::: tip
All verbs have short forms to save typing, e.g. `am a -l t cargo test` or `am p a rust`.
:::

## What's Next?

- [Installation](/guide/installation) — all installation methods in detail
- [Shell Setup](/guide/setup) — how the shell integration works
- [Profiles](/usage/profiles) — organize aliases into reusable groups
- [Project Aliases](/usage/project-aliases) — auto-loading `.aliases` files
