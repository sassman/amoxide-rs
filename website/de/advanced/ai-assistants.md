# KI-Assistenten

Amoxide-Aliase leben in Deiner interaktiven Shell. KI-Coding-Assistenten
(Claude Code, Codex, Cursor, …) führen Kommandos in nicht-interaktiven
Subshells aus, die diese Aliase nicht sehen. `am context` schließt
diese Lücke: Es gibt eine kompakte, modellfreundliche Momentaufnahme
Deines aktiven Alias-Sets aus, die per Session-Start-Hook in den
Assistenten injiziert wird.

Nach der Einrichtung kann Dein Assistent Kurzformen wie `git cm`,
`gst` oder Deine Subcommand-Aliase (`git pl`) in die kanonischen
Befehle expandieren, bevor er sie ausführt.

## Claude Code

Zwei gleichwertige Optionen — wähle nach Präferenz.

### Option 1 — automatische Einrichtung

```sh
am context --setup claude
```

Idempotent. Erkennt vorhandene Einträge und macht in diesem Fall
nichts. Legt `~/.claude/settings.json` an, falls nicht vorhanden;
fügt sich in eine vorhandene Datei ein, ohne andere Schlüssel oder
Hook-Events anzutasten.

### Option 2 — manuelle Einrichtung

Füge in `~/.claude/settings.json` ein:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|clear|compact",
        "hooks": [
          { "type": "command", "command": "am context", "async": false }
        ]
      }
    ]
  }
}
```

Der Matcher `"startup|clear|compact"` sorgt dafür, dass der Snapshot
auch nach `/clear` und `/compact` neu injiziert wird — ohne ihn
verliert der Assistent Deine Alias-Map mitten in der Sitzung.

## Andere Assistenten

`am context` ist generisch — sein Standard-Output funktioniert als
Session-Start-Kontext für jeden Assistenten, dessen Harness das
Ausführen eines Kommandos beim Session-Start unterstützt. Codex CLI,
Cursor und GitHub Copilot CLI haben ähnliche Mechanismen — siehe
deren Dokumentation für das jeweilige Gegenstück zur Hook-Konfiguration
von Claude Code. Native Unterstützung via `--setup <assistant>` für
diese ist geplant.

## Hinweis zur Output-Stabilität

Die Markdown-Form von `am context` ist **keine** stabile API. Das
Format kann sich aus Gründen der Modell-Verständlichkeit ohne
Vorankündigung ändern. Skripte gegen diesen Output zu schreiben ist
nicht empfohlen — für maschinenlesbare Formate bitte ein [Issue
öffnen](https://github.com/sassman/amoxide-rs/issues).
