# Subcommand Aliases <VersionBadge v="0.5.0" />

Many tools like `jj`, `git`, `cargo`, and `kubectl` organize their commands as subcommands. Subcommand aliases let you create short forms for those subcommands — without losing shell completions or wrapping behavior.

```sh
jj ab          # → jj abandon
jj b l         # → jj branch list
```

amoxide generates a shell function per program that intercepts the subcommand and dispatches to the full form.

## Adding Subcommand Aliases

Use a colon (`:`) in the name to signal a subcommand alias:

```sh
am add -g jj:ab abandon
# jj ab → jj abandon

am add -g jj:b:l branch list
# jj b l → jj branch list
```

The colon separates the program from the short subcommand token (and further levels for nested subcommands). The expansion is the full subcommand that gets run.

::: tip
Short form: `am a -g jj:ab abandon`
:::

The scope flags work exactly like regular aliases:

| Flag | Scope |
|------|-------|
| `-g` / `--global` | Global — always active |
| `-p <profile>` | Profile — active when profile is enabled |
| `-l` / `--local` | Project — loaded from `.aliases` |

## Nested Subcommand Aliases

Add more colon-separated tokens for nested dispatch:

```sh
am add -g jj:b:l branch list
am add -g jj:b:c branch create
am add -g kubectl:get:po "get pods"
```

When you run `jj b l`, amoxide dispatches through `jj → b → l` and expands to `jj branch list "$@"`.

## Templates in Expansions

Expansions support the same templates as regular aliases:

```sh
am add -g jj:anon "log -r 'anon()'"
# jj anon → jj log -r 'anon()'

am add -g jj:edit "edit {{1}}"
# jj edit abc123 → jj edit abc123

am add -g cargo:test "test --test {{1}} -- {{@}}"
# cargo test integration foo bar → cargo test --test integration -- foo bar
```

See [Parameterized Aliases](/advanced/parameterized-aliases) for the full template reference.

## Removing Subcommand Aliases

Use the same colon notation with `am remove`:

```sh
am remove -g jj:ab
am remove -g jj:b:l
```

Short form: `am r -g jj:ab`

## How It Works

At shell init (and on `am reload`), amoxide generates a wrapper function for each program that has subcommand aliases:

```sh
# generated for jj (bash/zsh)
jj() {
  case "$1" in
    ab) shift; command jj abandon "$@" ;;
    b)
      case "$2" in
        l) shift 2; command jj branch list "$@" ;;
        *) command jj "$@" ;;
      esac
      ;;
    *) command jj "$@" ;;
  esac
}
```

Any subcommand not matching a defined alias passes through to the real `jj` unchanged. Additional arguments are always forwarded after expansion.

## The `.aliases` File

Project-local subcommand aliases use a `[subcommands]` section alongside `[aliases]`:

```toml
# .aliases
[aliases]
t = "cargo test"

[subcommands]
"jj:ab" = ["abandon"]
"jj:b:l" = ["branch", "list"]
"jj:anon" = ["log -r 'anon()'"]
```

The key is the colon-joined path, and the value is an array of expansion tokens. The same file format applies in `config.toml` (global) and `profiles.toml` (per profile).

## Trust Model

Project subcommand aliases are subject to the same trust model as regular project aliases. When you run `am trust`, amoxide shows both the regular aliases and the subcommand aliases before asking for confirmation — so you always see exactly what you're approving.

See [Project Aliases — Trust Model](/usage/project-aliases#trust-model) for details.

## Listing Subcommand Aliases

`am ls` displays subcommand aliases grouped by program in the tree view:

```
🌐 global
│  ◆ jj
│  │  ├─ ab → abandon
│  │  ╰─ b:l → branch list
│
╰─📁 project (.aliases)
  ◆ cargo
     ╰─ test → test --test ...
```

The TUI (`am tui`) lets you view and manage subcommand aliases interactively.
