# Project Aliases

Project aliases live in a `.aliases` file at your project root. They load automatically when you `cd` into the directory and unload when you leave — like [direnv](https://direnv.net), but for aliases.

## Adding Project Aliases

Use the `-l` (local) flag:

```sh
cd ~/my-project
am add -l t "./x.py test"
am add -l b "./x.py build"
```

If no `.aliases` file exists, one is created in the current directory. If a `.aliases` already exists further up the directory tree, you'll be asked whether you meant to add to that one instead.

## The `.aliases` File

You can also create or edit the file directly:

```toml
# /path/to/my-project/.aliases
[aliases]
t = "./x.py test"
b = "./x.py build"
```

## How It Works

The `am init` shell hook calls `am hook <shell>` on every directory change. The hook:

1. Walks up from the current directory looking for a `.aliases` file (stopping before `$HOME`)
2. Unloads any previously active project aliases
3. Loads the new project's aliases

This means aliases automatically follow your context — switch to a Rust project and get Rust aliases, switch to a Node project and get Node aliases.

## Workflow

A natural workflow is to start with project-local aliases, then refactor duplicates into profiles:

**Step 1:** Add project-specific aliases:

```sh
am add -l t cargo test
am add -l l cargo clippy --all-targets -- -D warnings
am add -l i cargo install --path .
```

**Step 2:** Notice `t` and `l` are the same across Rust projects. Extract to a profile:

```sh
am profile add rust
am add -p rust t cargo test
am add -p rust l cargo clippy --all-targets -- -D warnings
am profile use rust
```

Now the project `.aliases` keeps only truly project-specific aliases like `i`.

::: tip
`am tui` lets you move aliases between project and profile levels visually — select an alias and press `m` to move it.
:::

## Moving Aliases with the TUI

<!-- TODO: Screenshot of am-tui in move mode, showing an alias being moved from project level to a profile -->
::: info Screenshot coming soon
The TUI in move mode — selecting a project alias and moving it to a reusable profile with a single keystroke.
:::
