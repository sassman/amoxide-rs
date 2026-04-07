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
am setup fish          # oder: zsh, bash, brush, powershell
```

Das erkennt deine Profil-Datei, zeigt genau was hinzugefügt wird und fragt nach Bestätigung. Siehe [Shell-Einrichtung](/de/guide/setup) für den manuellen Weg.

**3. Ersten Alias hinzufügen:**

```sh
am add gs git status
```

Das war's — `gs` ist jetzt in deiner Shell verfügbar, kein Neustart nötig.

## Empfohlener Workflow

Ich persönlich beginne mit **projekt-lokalen Aliasen**, um im aktuellen Kontext möglichst faul sein zu können.

Zum Beispiel habe ich in diesem Projekt:

- Binary installieren mit `i` macht `cargo install --path crates/am`
- Tests ausführen mit `t` macht `cargo test`
- Lint-Checks mit `l` macht `cargo clippy --all-targets --all-features -- -D warning`

**Schritt 1** Lokale / Projekt-Aliase mit `-l` hinzufügen:

```sh
am add -l t cargo test
#  ^^^ ^^ ^ ^--------^
#   |   | |       |
#   |   | |       +---- der Alias-Befehl
#   |   | +---- der Alias-Name
#   |   +---- lokal, schreibt in .aliases im aktuellen Verzeichnis
#   +---- einen Alias hinzufügen

am add -l l cargo clippy --all-targets --all-features -- -D warning
am add -l i cargo install --path crates/am
```

Diese *lokalen* Aliase leben in einer `.aliases`-Datei im Projekt-Root und werden automatisch beim `cd` geladen/entladen.

**Schritt 2 — Refactoring:** Nach einer Weile fällt mir auf, dass `t` und `l` in jedem Rust-Projekt gleich sind. Zeit, sie in ein wiederverwendbares **Profil** zu extrahieren — eine benannte Sammlung von Aliasen, die überall aktiviert werden kann:

```sh
am profile add rust
am add -p rust t cargo test
#      ^-----^
#         |
#         + ---- wir fügen den Alias zum Profil "rust" hinzu

am add -p rust l cargo clippy --all-targets --all-features -- -D warning

# aktivieren
am profile use rust
```

::: tip
`am tui` ermöglicht es, Aliase per Auswahl und `m` vom Projekt- auf Profil-Ebene zu verschieben
:::

Jetzt sind `t` und `l` überall verfügbar (nicht nur in diesem Projekt). Die `.aliases`-Datei behält nur projektspezifische wie `i`.

**Schritt 3 — Mehrere Profile nutzen.** Ich möchte auch Git-Aliase überall. Einfach beide aktivieren:

```sh
# Git-Profil mit einem Signing-Commit-Alias
am profile add git
am add -p git gm "git commit -S --signoff -m"

# beide aktivieren — Reihenfolge zählt: das zuletzt aktivierte gewinnt bei Konflikten
am profile use git
am profile use rust
```

Jetzt habe ich Git-Aliase und Rust-Aliase gleichzeitig geladen. Wenn beide Profile einen Alias mit dem gleichen Namen hätten, würde die zuletzt aktivierte Version (rust) gewinnen.

::: tip
Alle Verben haben Kurzformen: z.B. `am a -l t cargo test` oder `am p a rust`.
:::

## Nächste Schritte

- [Installation](/de/guide/installation) — alle Installationsmethoden im Detail
- [Shell-Einrichtung](/de/guide/setup) — wie die Shell-Integration funktioniert
- [Profile](/de/usage/profiles) — Aliase in wiederverwendbare Gruppen organisieren
- [Projekt-Aliase](/de/usage/project-aliases) — automatisch ladende `.aliases`-Dateien
