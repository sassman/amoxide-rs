# Usage

amoxide organizes aliases in three layers, from broadest to most specific:

1. **Global** — always active, available in every shell session
2. **Profiles** — named groups of aliases you can activate/deactivate
3. **Project** — local `.aliases` files that auto-load per directory

Each layer can override the previous one. Project aliases override profile aliases, which override global aliases.

All three layers also support **subcommand aliases** — short forms for programs that use subcommands (like `jj`, `git`, `cargo`, or `kubectl`).

```
🌐 global
│ helo → echo hello world global
│
├─● git (active: 1)
│ gm → git commit -S --signoff -m
│
├─● rust (active: 2)
│ ct → cargo test
│ cb → cargo build
│
╰─📁 project aliases (.aliases)
  t → ./x.py test
  b → ./x.py build

○ node
  nr → npm run
```

- [Global Aliases](/usage/global) — always-on aliases for every session
- [Profiles](/usage/profiles) — managing named alias groups
- [Project Aliases](/usage/project-aliases) — directory-scoped `.aliases` files
- [Subcommand Aliases](/usage/subcommand-aliases) — short forms for subcommand-based tools
- [Sharing](/usage/sharing) — export, import, and share with others
