# FAQ

## Welche Shells werden unterstützt?

| Shell | Status |
|-------|--------|
| Fish | Vollständig unterstützt und getestet |
| PowerShell | Unterstützt und getestet (5.1 + 7) |
| Zsh | Unterstützt, noch nicht getestet |
| Bash, Nushell | Noch nicht implementiert |

## Wie unterscheidet sich das von Shell-Aliasen in meinen Dotfiles?

Traditionelle Aliase sind statisch — einmal in `.bashrc` oder `.zshrc` definiert. amoxide bietet zusätzlich:

- **Profile** — benannte Gruppen, die ohne Dateien zu bearbeiten aktiviert/deaktiviert werden können
- **Projekt-Aliase** — automatisch pro Verzeichnis geladen, wie direnv für Aliase
- **Parametrisierte Aliase** — Template-Syntax für zusammensetzbare Befehle
- **Mehrere aktive Profile** — Aliase mit klarer Priorität schichten

## Wo werden meine Aliase gespeichert?

- **Globale und Profil-Aliase:** `~/.config/amoxide/config.toml`
- **Projekt-Aliase:** `.aliases`-Datei im Projektverzeichnis

## Kann ich amoxide neben meinen bestehenden Shell-Aliasen verwenden?

Ja. amoxide-Aliase koexistieren mit den nativen Shell-Aliasen. Bei Namenskonflikten hat der amoxide-Alias Vorrang, solange er aktiv ist.

## Was ist `am-tui`?

Eine separate Binary (`amoxide-tui`), die eine interaktive Terminal-Oberfläche zur visuellen Alias-Verwaltung bietet. Installiere sie neben `am`:

::: code-group

```sh [Homebrew]
brew install sassman/tap/amoxide-tui
```

```sh [Cargo]
cargo install amoxide-tui
```

:::
