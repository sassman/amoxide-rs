---
author: sassman
description: Git aliases for conventional commit workflows
category: git
tags: [git, conventional-commits, workflow]
profiles: [git, git-conventional]
---

# Git Conventional Commits

My daily driver for conventional commit workflows. Two profiles that build on each other:

- **git** — core git shortcuts (`gs`, `cm`, `ga`, `gp`, `gd`)
- **git-conventional** — conventional commit prefixes built on top (`cmf` for feat, `cmx` for fix, `cmd` for docs)

## Usage

```bash
am import https://raw.githubusercontent.com/sassman/amoxide-rs/main/community/sassman-git-conventional/profiles.toml
```

Activate both profiles in order:

```bash
am profile use git
am profile use git-conventional
```

Now `cmf my commit message` expands to `git commit -sm "feat: my commit message"`.
