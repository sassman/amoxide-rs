# Globale Aliase

Globale Aliase sind immer aktiv — verfügbar in jeder Shell-Sitzung, unabhängig davon, welche Profile aktiviert sind oder in welchem Verzeichnis du dich befindest.

## Globale Aliase hinzufügen

```sh
am add gs git status
am add ll "ls -lha"
```

Wenn du `am add` ohne `-p` (Profil) oder `-l` (lokal) ausführst, wird der Alias global hinzugefügt.

::: tip
Kurzform: `am a gs git status`
:::

## Globale Aliase entfernen

```sh
am remove gs
am r gs              # Kurzform
```

## Wann globale Aliase verwenden

Globale Aliase eignen sich am besten für Befehle, die du überall verwendest, unabhängig vom Projektkontext:

- Git-Abkürzungen (`gs`, `gp`, `gl`)
- System-Utilities (`ll`, `la`)
- Editor-Abkürzungen

Für Aliase, die nur in bestimmten Kontexten Sinn machen, verwende [Profile](/de/usage/profiles) (z.B. Rust-Toolchain-Aliase) oder [Projekt-Aliase](/de/usage/project-aliases) (z.B. projektspezifische Build-Befehle).

## Wie es funktioniert

Globale Aliase werden in `~/.config/amoxide/config.toml` gespeichert und über `am init` in jede Shell-Sitzung geladen. Sie bilden die Basis der Alias-Hierarchie — Profile und Projekt-Aliase können sie überschreiben, wenn sie einen Alias mit dem gleichen Namen definieren.
