---
author: sassman
description: Git aliases for conventional commit workflows
category: git
tags: [git, conventional-commits, workflow]
profiles: [git, git-conventional]
---

# Git Conventional Commits

Two profiles that build on each other for a fast conventional commit workflow.

## Core shortcuts

| Alias | Expands to | When I use it |
|-------|-----------|---------------|
| `gs` | `git status` | Quick check before committing |
| `ga` | `git add` | Stage files — `ga .` or `ga src/` |
| `gd` | `git diff` | Review changes before staging |
| `gp` | `git push` | Push after commit |
| `cm` | `git commit -sm` | Signed commit — `cm "my message"` |

## Conventional prefixes

These build on `cm` from above:

| Alias | Expands to | Example |
|-------|-----------|---------|
| `cmf` | `cm feat: ...` | `cmf add user export` |
| `cmx` | `cm fix: ...` | `cmx resolve null pointer on login` |
| `cmd` | `cm docs: ...` | `cmd update API reference` |

## Typical workflow

```bash
gs                          # check what changed
ga .                        # stage everything
cmf add sharing feature     # → git commit -sm "feat: add sharing feature"
gp                          # push
```

## Notes

- `cm` uses `-sm` (sign + message) — if you don't have GPG signing configured, change it to `-m`
- `cmf`, `cmx`, `cmd` use parameterized aliases (`{{@}}`) so the full argument list becomes the commit message
