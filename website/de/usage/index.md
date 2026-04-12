# Nutzung

amoxide organisiert Aliase in drei Ebenen, von breitester zu spezifischster:

1. **Global** — immer aktiv, in jeder Shell-Sitzung verfügbar
2. **Profile** — benannte Alias-Gruppen, die aktiviert/deaktiviert werden können
3. **Projekt** — lokale `.aliases`-Dateien, die sich automatisch pro Verzeichnis laden

Jede Ebene kann die vorherige überschreiben. Projekt-Aliase überschreiben Profil-Aliase, die wiederum globale Aliase überschreiben.

Alle drei Ebenen unterstützen auch **Subcommand-Aliase** — Kurzformen für Programme, die Subcommands verwenden (wie `jj`, `git`, `cargo` oder `kubectl`).

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

- [Globale Aliase](/de/usage/global) — immer verfügbare Aliase für jede Sitzung
- [Profile](/de/usage/profiles) — benannte Alias-Gruppen verwalten
- [Projekt-Aliase](/de/usage/project-aliases) — verzeichnisbezogene `.aliases`-Dateien
- [Subcommand-Aliase](/de/usage/subcommand-aliases) — Kurzformen für subcommandbasierte Tools
- [Teilen](/de/usage/sharing) — Aliase exportieren, importieren und teilen
