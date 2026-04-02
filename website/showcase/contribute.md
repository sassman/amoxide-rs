# Share Your Profiles

Got a profile collection others might find useful? Here's how to add it to the showcase.

## Step by step

### 1. Export your profiles

```bash
am export -p <profile-name> > profiles.toml
```

Or export multiple profiles at once:

```bash
am export -p git -p git-conventional > profiles.toml
```

### 2. Create your folder

```
community/<your-github-handle>-<descriptive-name>/
├── README.md
└── profiles.toml
```

For example: `community/sassman-git-conventional/`

### 3. Write the README

Copy from [`TEMPLATE.md`](https://github.com/sassman/amoxide-rs/blob/main/community/TEMPLATE.md) and fill in the frontmatter:

```yaml
---
author: your-github-handle
description: A short one-line description
category: git          # git, docker, rust, k8s, python, node, misc
tags: [tag1, tag2]
shell: fish            # fish, zsh, bash, powershell
profiles: [profile-name-1, profile-name-2]
---
```

Then explain what your aliases do, how you use them, and any dependencies.

### 4. Test it

Make sure your export imports cleanly:

```bash
cat profiles.toml | am import --yes
```

### 5. Open a PR

Use the **Community Profile** PR template. The checklist will guide you through what's needed.

**Rules:**
- Only add/modify files in your own `community/<handle>-<name>/` folder
- One folder per alias collection (you can have multiple profiles in one `profiles.toml`)
- If you want to share a second collection, create a second folder

## Naming conventions

| Part | Convention | Example |
|------|-----------|---------|
| Folder | `<github-handle>-<descriptive-name>` | `sassman-git-conventional` |
| TOML file | Always `profiles.toml` | `profiles.toml` |
| Category | Lowercase, one of the established categories | `git`, `docker`, `rust` |

## What makes a good contribution?

- **Useful to others** — aliases that solve common workflows, not personal one-offs
- **Well documented** — explain what each alias does and when to use it
- **Self-contained** — note any dependencies (tools that must be installed)
- **Tested** — verify the import works before submitting

::: warning Security
All submissions are reviewed before merging. We check for suspicious content, but you should always inspect aliases before importing — even from this showcase.
:::
