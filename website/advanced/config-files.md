# Config File Reference

amoxide stores its configuration in `~/.config/amoxide/` using TOML files. You rarely need to edit these by hand — the CLI manages them — but understanding the format helps when debugging or sharing setups.

## File Overview

| File | Purpose |
|------|---------|
| `config.toml` | Global aliases and active profile list |
| `profiles.toml` | All profile definitions and their aliases |
| `.aliases` | Project-local aliases (lives in project root) |

## `config.toml` — Global Config

```toml
# Which profiles are currently active, in priority order
active_profiles = ["git", "rust"]

# Global aliases — always available
[aliases]
helo = "echo hello world"
ll = "ls -lha"
```

The `active_profiles` array determines which profiles are loaded and their precedence. The last entry has the highest priority — if both `git` and `rust` define an alias with the same name, `rust` wins.

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

[[profiles]]
name = "node"

[profiles.aliases]
t = "npm run test"
b = "npm run build"
```

Each `[[profiles]]` block defines a named profile with its aliases. Note that different profiles can use the same alias name (e.g., `t` in both `rust` and `node`) — whichever profile has higher priority in `active_profiles` wins.

## `.aliases` — Project Aliases

This file lives in your project root and is loaded automatically when you `cd` into the directory.

```toml
[aliases]
i = "cargo install --path crates/am && cargo install --path crates/am-tui"
l = "cargo clippy --locked --all-targets -- -D warnings"
t = "cargo test --all-features"
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
