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
```

```powershell [PowerShell]
irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.ps1 | iex
```

```sh [Cargo]
cargo install amoxide amoxide-tui
# oder ohne Kompilierung:
cargo binstall amoxide amoxide-tui
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

Das war's — `gs` ist jetzt global in jeder Shell-Sitzung verfügbar, kein Neustart nötig.

**4. Aliase anzeigen:**

```sh
am ls
# oder kurz: am l
```

## Nächste Schritte

- [Installation](/de/guide/installation) — alle Installationsmethoden im Detail
- [Shell-Einrichtung](/de/guide/setup) — wie die Shell-Integration funktioniert
- [Profile](/de/config/profiles) — Aliase in wiederverwendbare Gruppen organisieren
- [Projekt-Aliase](/de/config/project-aliases) — automatisch ladende `.aliases`-Dateien
