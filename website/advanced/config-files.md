# Config File Reference

amoxide stores its configuration in `~/.config/amoxide/` using TOML files. You rarely need to edit these by hand — the CLI manages them — but understanding the format helps when debugging or sharing setups.

## File Overview

| File | Purpose |
|------|---------|
| `config.toml` | Global aliases and shell options |
| `profiles.toml` | All profile definitions and their aliases |
| `session.toml` | Active profile list (which profiles are currently on) |
| `security.toml` | Trust decisions for project `.aliases` files <VersionBadge v="0.5.0" /> |
| `.aliases` | Project-local aliases (lives in project root) |

## `config.toml` — Global Config

```toml
# Global aliases — always available
[aliases]
helo = "echo hello world"
ll = "ls -lha"

# Global subcommand aliases — short forms for subcommand-based tools
[subcommands]
"jj:ab" = ["abandon"]
"jj:b:l" = ["branch", "list"]
```

## `config.toml` — Shell Options

Shell-specific rendering options can be set under the `[shell.<name>]` section.

### Fish: `[shell.fish]` <VersionBadge v="0.6.0" />

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `use_abbr` | bool | `false` | Render simple aliases as [fish abbreviations](https://fishshell.com/docs/current/cmds/abbr.html) (`abbr --add`) instead of `alias` |

```toml
[shell.fish]
use_abbr = true
```

When `use_abbr = true`, every simple alias from every layer (global, profile, and project) is emitted as an abbreviation. Abbreviations expand in-line as you type, which keeps your command history clean.

<span v-pre>Parameterized aliases — those that use `{{1}}` or `{{@}}` placeholders — are always emitted as `function` definitions regardless of this setting, because fish abbreviations do not support arguments.</span>

Example output with `use_abbr = true`:

```fish
abbr --add gs "git status"
abbr --add ll "ls -lha"
```

Example output for a parameterized alias (always a function, regardless of `use_abbr`):

```fish
function cmf
    cm feat: $argv
end
```

To enable this setting, edit `~/.config/amoxide/config.toml` manually and add the `[shell.fish]` block shown above. Then run `am reload fish` (or start a new shell session) to apply the change.

## `profiles.toml` — Profile Definitions

```toml
[[profiles]]
name = "git"

[profiles.aliases]
ga = "git commit --amend"
gcm = "git commit -S --signoff -m"
gst = "git status"

[[profiles]]
name = "rust"

[profiles.aliases]
f = "cargo fmt"
t = "cargo test --all-features"
l = "cargo clippy --locked --all-targets -- -D warnings"

[profiles.subcommands]
"cargo:t" = ["test", "--all-features"]

[[profiles]]
name = "node"

[profiles.aliases]
t = "npm run test"
b = "npm run build"
```

Each `[[profiles]]` block defines a named profile with its aliases and optional subcommand aliases. Note that different profiles can use the same alias name (e.g., `t` in both `rust` and `node`) — whichever profile has higher priority in `active_profiles` wins.

## `session.toml` — Active Profiles <VersionBadge v="0.5.0" />

Tracks which profiles are currently active and in what order. Managed automatically by `am profile use` and `am use` — you rarely need to edit this directly.

```toml
active_profiles = ["git", "rust"]
```

The order determines precedence: the **last** entry has the highest priority. If both `git` and `rust` define an alias with the same name, `rust` wins.

## `security.toml` — Trust Decisions <VersionBadge v="0.5.0" />

Tracks which project `.aliases` files you have reviewed and trusted. Managed automatically by `am trust` and `am untrust` — you shouldn't need to edit this file.

```toml
[[trusted]]
path = "/home/user/projects/my-app/.aliases"
hash = "a1b2c3d4e5f6..."

[[untrusted]]
path = "/home/user/projects/declined-repo/.aliases"
```

Each trusted entry stores the file path and a BLAKE3 hash of its contents. If the file changes, the hash won't match and amoxide will ask you to re-review. See [Trust Model](/usage/project-aliases#trust-model) for details.

A third section, `[[tampered]]`, appears automatically when a trusted file is modified outside of `am`. It clears when you run `am trust` to review the changes.

## `.aliases` — Project Aliases

This file lives in your project root and is loaded automatically when you `cd` into the directory.

```toml
[aliases]
i = "cargo install --path crates/am && cargo install --path crates/am-tui"
l = "cargo clippy --locked --all-targets -- -D warnings"
t = "cargo test --all-features"

[subcommands]
"jj:ab" = ["abandon"]
"jj:b:l" = ["branch", "list"]
```

Project aliases override profile aliases with the same name. This lets you customize shortcuts per project without changing your global setup.

## Priority Order

When multiple layers define the same alias name, the most specific one wins:

```
Project aliases (.aliases)    ← highest priority
  ↑ overrides
Active profiles (last wins)
  ↑ overrides
Global aliases (config.toml)  ← lowest priority
```

The same priority order applies to subcommand aliases. A `[subcommands]` entry in `.aliases` overrides the same key from an active profile, which overrides the same key in `config.toml`.

See [Subcommand Aliases](/usage/subcommand-aliases) for usage examples and how the shell wrappers are generated.
