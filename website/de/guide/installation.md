# Installation

## Paketmanager

::: code-group

```sh [Homebrew (macOS/Linux)]
brew install sassman/tap/amoxide sassman/tap/amoxide-tui
```

```sh [Shell-Skript (macOS/Linux)]
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.sh | sh
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.sh | sh
```

```powershell [PowerShell (Windows)]
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.ps1 | iex"
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.ps1 | iex"
```

```sh [Cargo (pre-built)]
cargo binstall amoxide amoxide-tui
```

```sh [Cargo (source)]
cargo install amoxide amoxide-tui
```

:::

Das Crate heißt `amoxide`, aber die installierte Binary heißt einfach `am`.

::: tip
Der TUI-Companion (`am-tui`) ist ein separates Paket. Optional, aber empfohlen für visuelle Alias-Verwaltung.
:::

## Shell-Unterstützung

| Shell | Status |
|-------|--------|
| Fish | Vollständig unterstützt und getestet |
| PowerShell | Unterstützt und getestet (5.1 + 7) |
| Zsh | Unterstützt, noch nicht getestet |
| Bash, Nushell | Noch nicht implementiert |

## Befehle

| Befehl | Beschreibung |
|--------|-------------|
| `am add` | Neuen Alias hinzufügen (global, Profil oder lokal) |
| `am remove` | Alias entfernen |
| `am profile` | Profile verwalten (add, use, remove, list) |
| `am ls` | Alle Profile und Projekt-Aliase auflisten |
| `am init` | Shell-Init-Code ausgeben |
| `am status` | Prüfen, ob die Shell korrekt eingerichtet ist |
| `am setup` | Geführte Shell-Einrichtung |
| `am tui` | Interaktives TUI zur Alias-Verwaltung |
| `am hook` | Wird vom cd-Hook aufgerufen (intern) |

::: tip
Alle Verben haben Kurzformen: `am a` für add, `am r` für remove, `am p` für profile, `am l` für ls.
:::
