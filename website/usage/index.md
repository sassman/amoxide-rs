# Usage

amoxide organizes aliases in three layers, from broadest to most specific:

1. **Global** — always active, available in every shell session
2. **Profiles** — named groups of aliases you can activate/deactivate
3. **Project** — local `.aliases` files that auto-load per directory

Each layer can override the previous one. Project aliases override profile aliases, which override global aliases.

All three layers also support **subcommand aliases** — short forms for programs that use subcommands (like `jj`, `git`, `cargo`, or `kubectl`).

```
🌐 global
│  ╰─ ll → ls -lha
│
├─● rust (active: 1)
│   ├─ i → cargo install --path .
│   ├─ l → cargo clippy --locked --all-targets -- -D warnings
│   ╰─ t → cargo test --all-features
│
├─● git (active: 2)
│   ├─ gm → git commit -S --signoff -m
│   ╰─◆ git (subcommands)
│     ├─ psh → push
│     ╰─ st → status --short
│
╰─📁 project (~/path/to/project/.aliases)
  ├─ b → ./x.py build
  ╰─ t → ./x.py test

○ node
  ╰─ nr → npm run
```

- [Global Aliases](/usage/global) — always-on aliases for every session
- [Profiles](/usage/profiles) — managing named alias groups
- [Project Aliases](/usage/project-aliases) — directory-scoped `.aliases` files
- [Subcommand Aliases](/usage/subcommand-aliases) — short forms for subcommand-based tools
- [Sharing](/usage/sharing) — export, import, and share with others
