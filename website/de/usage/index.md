# Nutzung

amoxide organisiert Aliase in drei Ebenen, von breitester zu spezifischster:

1. **Global** — immer aktiv, in jeder Shell-Sitzung verfügbar
2. **Profile** — benannte Alias-Gruppen, die aktiviert/deaktiviert werden können
3. **Projekt** — lokale `.aliases`-Dateien, die sich automatisch pro Verzeichnis laden

Jede Ebene kann die vorherige überschreiben. Projekt-Aliase überschreiben Profil-Aliase, die wiederum globale Aliase überschreiben.

Alle drei Ebenen unterstützen auch **Subcommand-Aliase** — Kurzformen für Programme, die Subcommands verwenden (wie `jj`, `git`, `cargo` oder `kubectl`).

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

- [Globale Aliase](/de/usage/global) — immer verfügbare Aliase für jede Sitzung
- [Profile](/de/usage/profiles) — benannte Alias-Gruppen verwalten
- [Projekt-Aliase](/de/usage/project-aliases) — verzeichnisbezogene `.aliases`-Dateien
- [Subcommand-Aliase](/de/usage/subcommand-aliases) — Kurzformen für subcommandbasierte Tools
- [Teilen](/de/usage/sharing) — Aliase exportieren, importieren und teilen
