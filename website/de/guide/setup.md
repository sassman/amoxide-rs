# Shell-Einrichtung

## Geführtes Setup (Empfohlen)

Der einfachste Weg, amoxide einzurichten:

::: code-group

```sh [Fish]
am setup fish
```

```sh [Zsh]
am setup zsh
```

```powershell [PowerShell]
am setup powershell
```

```sh [Bash (v0.4.0+)]
am setup bash
```

```sh [Brush]
am setup brush
```

:::

Das erkennt deine Profil-Datei, zeigt genau was hinzugefügt wird und fragt nach Bestätigung.

## Manuelles Setup

Füge die Init-Zeile zu deiner Shell-Konfiguration hinzu:

::: code-group

```fish [Fish]
# ~/.config/fish/config.fish
am init fish | source
```

```zsh [Zsh]
# ~/.zshrc
eval "$(am init zsh)"
```

```powershell [PowerShell]
# Zu deinem PowerShell-Profil hinzufügen (echo $PROFILE zeigt den Pfad)
(am init powershell) -join "`n" | Invoke-Expression
```

```bash [Bash (v0.4.0+)]
# ~/.bashrc
eval "$(am init bash)"
```

```bash [Brush]
# ~/.brushrc
eval "$(am init brush)"
```

:::

## Was das Init macht

Der `am init`-Befehl macht zwei Dinge:

1. **Lädt Aliase** aus deinen aktiven Profilen in die aktuelle Shell
2. **Installiert einen cd-Hook**, der automatisch Projekt-Aliase (aus `.aliases`-Dateien) lädt und entlädt, wenn du das Verzeichnis wechselst

## Setup überprüfen

Prüfe, ob alles korrekt konfiguriert ist:

```sh
am status
```
