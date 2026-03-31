# Erste Schritte

amoxide (`am`) ist ein Shell-Alias-Manager, der [direnv](https://direnv.net)-ähnliche Funktionalität für Aliase bietet. Statt Aliase in statischen Dotfiles zu verwalten, definiere sie pro Projekt, pro Toolchain oder global — die richtigen werden automatisch geladen.

## Schnellstart

**1. amoxide installieren:**

::: code-group

```sh [Homebrew]
brew install sassman/tap/amoxide sassman/tap/amoxide-tui
```

```sh [Shell-Skript]
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.sh | sh
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.sh | sh
```

```powershell [PowerShell]
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.ps1 | iex"
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.ps1 | iex"
```

```sh [Cargo (pre-built)]
cargo binstall amoxide amoxide-tui
```

```sh [Cargo]
cargo install amoxide amoxide-tui
```

:::

**2. Shell einrichten:**

```sh
am setup fish          # oder: zsh, powershell
```

Das erkennt deine Profil-Datei, zeigt genau was hinzugefügt wird und fragt nach Bestätigung. Siehe [Shell-Einrichtung](/de/guide/setup) für den manuellen Weg.

**3. Ersten Alias hinzufügen:**

```sh
am add gs git status
```

Das war's — `gs` ist jetzt in deinem aktiven Profil verfügbar, kein Neustart nötig. Verwende `-g` für einen globalen Alias (immer aktiv) oder `-l` für einen projektspezifischen:

```sh
am add -l t cargo test
```

Das schreibt in eine `.aliases`-Datei im aktuellen Verzeichnis — automatisch geladen beim `cd` hinein, entladen beim Verlassen. Siehe [Nutzung](/de/usage/) für das Zusammenspiel von globalen, Profil- und Projekt-Aliasen.

**4. Aliase anzeigen:**

```sh
am ls
# oder kurz: am l
```

## Nächste Schritte

- [Installation](/de/guide/installation) — alle Installationsmethoden im Detail
- [Shell-Einrichtung](/de/guide/setup) — wie die Shell-Integration funktioniert
- [Profile](/de/usage/profiles) — Aliase in wiederverwendbare Gruppen organisieren
- [Projekt-Aliase](/de/usage/project-aliases) — automatisch ladende `.aliases`-Dateien
