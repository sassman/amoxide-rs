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

## Removing Project Aliases <VersionBadge v="0.6.0" />

Use the same `-l` flag with `am remove`:

```sh
am remove -l t
am remove -l b
```

This updates the `.aliases` file and refreshes the cryptographic hash automatically, so no trust warning is triggered.

## The `.aliases` File

You can also create or edit the file directly:

```toml
# /path/to/my-project/.aliases
[aliases]
t = "./x.py test"
b = "./x.py build"
```

## Trust Model <VersionBadge v="0.5.0" />

Project `.aliases` files can contain arbitrary shell commands. Since anyone could put a `.aliases` file into a repository, amoxide requires you to explicitly trust each file before its aliases are loaded — similar to how [direnv](https://direnv.net) handles `.envrc` files.

### First encounter

When you `cd` into a directory with an untrusted `.aliases` file, you'll see:

```
am: .aliases found but not trusted. Run 'am trust' to review and allow.
```

No aliases are loaded until you review and approve them.

### Reviewing and trusting

Run `am trust` to review the aliases and decide:

```
❯ am trust
Reviewing .aliases at /home/user/projects/my-app

  b  → make build
  t  → cargo test
  cb → cargo build

Trust these aliases? [Y/n]
```

If the file contains suspicious content (hidden escape sequences or control characters), a warning is shown before the prompt.

Answering **yes** trusts the file — aliases are loaded immediately:

```
am: loaded .aliases
  b  → make build
  t  → cargo test
  cb → cargo build
```

Answering **no** marks the directory as untrusted. Future `cd`s into it will be silent — no warnings, no aliases.

### Revoking trust

```sh
am untrust          # mark as untrusted (silent on cd)
am untrust --forget # remove from tracking entirely (will prompt again)
```

### Tamper detection

amoxide stores a cryptographic hash (BLAKE3) of every trusted `.aliases` file. If the file changes outside of `am`, the hash won't match and you'll see:

```
am: .aliases was modified since last trusted. Run 'am trust' to review and allow.
```

This happens when the file is edited manually, updated by `git pull`, or changed by any tool other than `am`. The warning repeats on every `cd` until you review the changes with `am trust`.

When you use `am` itself to modify the file — via `am add -l` or `am remove -l` — the hash is updated automatically, so those changes never trigger this warning.

### Load and unload messages

When aliases are loaded, you see which commands became available:

```
am: loaded .aliases
  b  → make build
  t  → cargo test
```

When you leave the project:

```
am: unloaded .aliases: b, t
```

These messages only appear when entering or leaving the directory containing the `.aliases` file — not when navigating subdirectories within the same project.

## How It Works

The `am init` shell hook calls `am hook <shell>` on every directory change. The hook:

1. Walks up from the current directory looking for a `.aliases` file (stopping before `$HOME`)
2. Checks whether the file is trusted (path + hash match in `security.toml`)
3. If trusted: unloads any previously active project aliases and loads the new ones
4. If not trusted: shows a warning or stays silent, depending on the trust state

This means aliases automatically follow your context — switch to a Rust project and get Rust aliases, switch to a Node project and get Node aliases — as long as you've trusted the respective `.aliases` files.

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

Use `am tui` to move aliases from project level to a profile visually — select an alias and press `m`:

<video autoplay loop muted playsinline>
  <source src="/am-tui-moving-aliases.webm" type="video/webm">
  <source src="/am-tui-moving-aliases.mp4" type="video/mp4">
</video>
