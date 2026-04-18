# FAQ

## Welche Shells werden unterstützt?

| Shell | Status |
|-------|--------|
| Fish | Vollständig unterstützt und getestet |
| PowerShell | Unterstützt und getestet (5.1 + 7) |
| Zsh | Unterstützt, noch nicht getestet |
| Bash | Unterstützt (3.2+) |
| Brush | Unterstützt (bash-kompatibel) |
| Nushell | Noch nicht implementiert |

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

Eine separate Binary (`amoxide-tui`), die eine interaktive Terminal-Oberfläche zur visuellen Alias-Verwaltung bietet. Nach der Installation integriert sie sich direkt in `am`:

```sh
am tui
```

Der `am tui`-Befehl startet das TUI — kein separater Binary-Name nötig. Schnelle Installation:

```sh
brew install sassman/tap/amoxide-tui
```

Siehe [alle Installationsoptionen](/de/guide/installation#am-tui-installieren) für Shell-Skript, PowerShell und Cargo.

## Ich habe meine Config-Datei manuell geändert — wie wende ich die Änderungen ohne Neustart der Shell an?

Führe `am init -f <shell>` aus, um die Shell neu zu initialisieren. Das entlädt alle aktuellen Aliase und lädt alles aus der Config neu, einschließlich Einstellungsänderungen wie [`use_abbr = true`](/de/advanced/config-files#fish-shell-fish) in Fish. Weitere Details unter [Neu initialisieren ohne Neustart](/de/guide/setup#neu-initialisieren-ohne-neustart).

::: code-group

```fish [Fish]
am init -f fish | source
```

```zsh [Zsh]
eval "$(am init -f zsh)"
```

```powershell [PowerShell]
(am init -f powershell) -join "`n" | Invoke-Expression
```

```bash [Bash]
eval "$(am init -f bash)"
```

:::
