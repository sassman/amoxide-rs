# Composing Aliases

Aliases can reference other aliases. Since amoxide aliases are expanded by the shell, you can build powerful command chains by layering simple aliases on top of each other.

## Basic Composition

Define a base alias, then build on it:

```sh
# Base: a signing commit
am add -p git cm "git commit -S --signoff -m"

# Build on top: conventional commit prefixes
am add -p git cmf "cm feat:"
am add -p git cmx "cm fix:"
am add -p git cmd "cm docs:"

cmf add user authentication
# → cm feat: add user authentication
# → git commit -S --signoff -m feat: add user authentication
```

Change the base `cm` alias once, and all variants (`cmf`, `cmx`, `cmd`) inherit the change.

## Cross-Profile Composition

Aliases from different profiles can reference each other, as long as both profiles are active:

```sh
# git profile — base tools
am add -p git ga "git add"
am add -p git gc "git commit -S --signoff -m"

# workflow profile — higher-level shortcuts
am add -p workflow wip "ga -A && gc wip"
am add -p workflow ship "ga -A && gc"

# activate both
am profile use git
am profile use workflow

ship ready to merge
# → ga -A && gc ready to merge
# → git add -A && git commit -S --signoff -m ready to merge
```

## Mixing with Project Aliases

Project aliases can reference profile aliases too. A Rust project might define:

```sh
# profile: always available
am add -p rust t "cargo test"
am add -p rust l "cargo clippy --locked --all-targets -- -D warnings"

# project-local: builds on the profile alias
am add -l check "l && t"
```

Now `check` runs clippy then tests — and if you change `l` or `t` in the profile, the project alias picks up the change.

## Tips

- Keep base aliases simple and focused — one command, one purpose
- Name composed aliases so the chain is guessable (`cm` → `cmf` for "cm feat")
- Use `am ls` to see which aliases are available and from which layer
