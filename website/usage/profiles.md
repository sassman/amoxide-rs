# Profiles

Profiles are named groups of aliases. Think of them as layers — you can activate multiple simultaneously, and later-activated profiles override earlier ones for conflicting alias names.

## Creating a Profile

```sh
am profile add rust
am p a rust          # short form
```

## Adding Aliases to a Profile

```sh
am add -p rust ct "cargo test"
am add -p rust cb "cargo build"
```

## Activating Profiles

```sh
# Activate a profile (adds on top of the current stack)
am profile use rust
am p u rust          # short form

# Activate at a specific position (1 = base layer)
am profile use git -n 1
```

When you activate multiple profiles, they stack. The last-activated profile wins on conflicts:

```sh
am profile use git    # base layer (active: 1)
am profile use rust   # on top (active: 2)
# If both have alias "t", rust's version wins
```

## Listing Profiles

```sh
am profile           # default action
am profile list      # explicit
am l                 # shortest form
```

Active profiles are shown connected by a tree trunk. Inactive profiles appear below:

```
🌐 global
│
├─● git (active: 1)
│ gm → git commit -S --signoff -m
│
├─● rust (active: 2)
│ ct → cargo test
│
╰─📁 project aliases (.aliases)

○ node
  nr → npm run
```

## Removing a Profile

```sh
am profile remove rust     # asks for confirmation if it has aliases
am p r rust -f             # skip confirmation
```

## Adding and Removing Aliases

```sh
# Add to the currently active profile
am add gs git status

# Add to a specific profile
am add -p rust ct cargo test

# Remove from the active profile
am remove gs
am r gs              # short form

# Remove from a specific profile
am remove -p rust ct
```

::: tip
All verbs have short forms: `am a` for add, `am r` for remove, `am p a` for profile add, `am p u` for profile use.
:::
