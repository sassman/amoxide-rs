# Share Your Profiles

Got a profile collection others might find useful? Here's how to add it to the showcase.

## Prerequisites

- [amoxide](https://github.com/sassman/amoxide-rs) installed
- A [GitHub](https://github.com) account

## Step by step

### 1. Fork the repository

Go to [github.com/sassman/amoxide-rs](https://github.com/sassman/amoxide-rs) and click **Fork** (top right). This creates your own copy at `github.com/<your-handle>/amoxide-rs`.

### 2. Clone your fork

```bash
git clone git@github.com:<your-handle>/amoxide-rs.git
cd amoxide-rs
```

### 3. Create a branch

```bash
git checkout -b community/<your-handle>-<descriptive-name>
```

For example: `community/sassman-git-conventional`

### 4. Copy the template

```bash
cp -r community/TEMPLATE community/<your-handle>-<descriptive-name>
```

This gives you:

```
community/<your-handle>-<descriptive-name>/
├── README.md     ← edit this
└── profiles.toml ← replace with your export
```

### 5. Export your profiles

Replace the template `profiles.toml` with your actual export:

```bash
am export -p <profile-name> > community/<your-handle>-<descriptive-name>/profiles.toml
```

Or export multiple profiles:

```bash
am export -p git -p git-conventional > community/<your-handle>-<descriptive-name>/profiles.toml
```

### 6. Edit the README

Open `community/<your-handle>-<descriptive-name>/README.md` and fill in the frontmatter:

```yaml
---
author: your-github-handle
description: A short one-line description
category: git
tags: [tag1, tag2]
profiles: [profile-name-1, profile-name-2]
---
```

Then write a few sentences about what your aliases do, how you use them, and any tools that need to be installed.

::: details Frontmatter reference
| Field | Required | Description |
|-------|----------|-------------|
| `author` | yes | Your GitHub handle |
| `description` | yes | One-line summary (shown on the tile) |
| `category` | yes | One of: `git`, `docker`, `rust`, `k8s`, `python`, `node`, `misc` |
| `tags` | yes | Array of keywords for filtering |
| `profiles` | yes | Profile names included in your `profiles.toml` |
| `shell` | no | Only set if your aliases use shell-specific syntax (e.g. `fish`) |
:::

### 7. Test it

Make sure the import works:

```bash
cat community/<your-handle>-<descriptive-name>/profiles.toml | am import --yes
```

### 8. Commit and push

```bash
git add community/<your-handle>-<descriptive-name>/
git commit -m "community: add <your-handle>-<descriptive-name>"
git push origin community/<your-handle>-<descriptive-name>
```

### 9. Open a Pull Request

Go to your fork on GitHub — you'll see a banner to create a Pull Request. Click it and select the **Community Profile** PR template.

The checklist will guide you through what's needed:

- [ ] Folder named `community/<handle>-<name>/`
- [ ] `profiles.toml` is a valid `am export` output
- [ ] `README.md` has the required frontmatter
- [ ] Only files in your own folder are modified
- [ ] Import tested locally

Your contribution will appear on the showcase after review.

## Rules

- Only add or modify files in your own `community/<handle>-<name>/` folder
- One folder per alias collection (multiple profiles in one `profiles.toml` is fine)
- For a second collection, create a second folder (e.g. `sassman-docker-compose`)

## What makes a good contribution?

- **Useful to others** — aliases that solve common workflows
- **Well documented** — explain what each alias does
- **Self-contained** — note any dependencies
- **Tested** — verify the import works

::: warning Security
All submissions are reviewed before merging. We check for suspicious content, but you should always inspect aliases before importing — even from this showcase.
:::
