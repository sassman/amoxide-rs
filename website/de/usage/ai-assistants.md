# KI-Assistenten

Deine Aliase leben in Deiner interaktiven Shell. KI-Coding-Assistenten —
Claude Code, Codex, Cursor — führen Kommandos in nicht-interaktiven
Subshells aus, die Dein Shell-Init nicht laden. Wenn Du also `ct = cargo
test` in einem `rust`-Profil definiert hast und den Assistenten bittest
„lass die Tests laufen", probiert er `ct` und bekommt `command not
found`.

`am context` gibt Dein aktives Alias-Set als Markdown aus, das der
Assistent beim Sitzungsstart einlesen kann. Einmal verdrahtet,
expandiert der Assistent `ct` zu `cargo test`, bevor er es ausführt.

## Installation

```sh
am context --setup claude
```

Idempotent. Legt `~/.claude/settings.json` an, falls nicht vorhanden,
oder fügt sich in eine vorhandene Datei ein, ohne andere Schlüssel
anzutasten. Mehrfach ausführen ist sicher.

Für andere Assistenten `am context` manuell aus deren
Session-Start-Hook aufrufen — siehe die jeweilige Hook-Dokumentation.

## Was Du in einer Claude-Code-Sitzung siehst

Öffne eine neue Claude-Code-Sitzung in Deinem Projektverzeichnis. Der
Assistent hat jetzt Deine aktiven Aliase — `ll`, `gs`, `ct`, alles aus
aktiven Profilen, alles aus einer vertrauten `.aliases`-Datei im
Projekt.

Probier: „lass die Tests laufen". Der Assistent führt `cargo test` aus
(die kanonische Form), nicht `ct`. Genauso bei `git pl` → `git pull
--rebase`, `gst` → `git status`.

Subcommand-Aliase funktionieren auch. Der Assistent weiß, dass `git pl`
wie ein Subcommand aussieht, aber keiner ist, und führt die Expansion
aus.

## Überprüfung

In einer frischen Sitzung fragen: **„welche Aliase habe ich?"**

Der Assistent sollte sie direkt aus der Momentaufnahme auflisten, ohne
ein Kommando auszuführen. Wenn nicht, ist der Hook nicht gefeuert —
prüf `~/.claude/settings.json`.

## Manuelle Einrichtung

Wenn Du das JSON lieber selbst editierst:

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

Der Matcher `"startup|clear|compact"` ist wichtig — ohne ihn wird die
Momentaufnahme nur beim Kaltstart injiziert, und der Assistent verliert
Deine Aliase beim ersten `/clear` oder `/compact`.

## Hinweise

- Die Markdown-Form von `am context` kann sich aus Gründen der
  Modell-Verständlichkeit ändern. Schreib keine Skripte gegen die
  Ausgabe.
- `am context --verbose` ergänzt die volle Shadow-Kette und etwaige
  Diagnosen zu ungültigen Aliasen.
