---
author: sassman
description: Rust development shortcuts for fmt, lint, and test
category: rust
tags: [rust, cargo, testing, linting]
profiles: [rust]
---

# Rust Essentials

Three aliases I use on every Rust project — format, lint, test. Nothing fancy, just less typing.

## Aliases

| Alias | Expands to | When I use it |
|-------|-----------|---------------|
| `f` | `cargo fmt` | Format before committing |
| `l` | `cargo clippy --locked --all-targets -- -D warnings` | Lint with strict warnings |
| `t` | `cargo test --all-features` | Run the full test suite |

## Typical workflow

```bash
f           # format
l           # lint — fix anything clippy complains about
t           # test — make sure nothing broke
```

## Notes

- `l` uses `--locked` to ensure the lockfile is respected and `-D warnings` to treat warnings as errors
- `t` uses `--all-features` to test everything, not just the default feature set
