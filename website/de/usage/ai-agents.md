# KI-Agenten

Deine Aliase leben in Deiner interaktiven Shell. KI-Coding-Agenten wie
Claude Code, Codex oder Cursor führen Kommandos in nicht-interaktiven
Subshells aus, die Dein Shell-Init nicht laden. Wenn Du also `ct = cargo
test` in einem `rust`-Profil definiert hast und den Agenten bittest
„lass die Tests laufen", probiert er `ct` und bekommt `command not
found`.

`am context` gibt Dein aktives Alias-Set als Markdown aus, das der Agent
beim Sitzungsstart einlesen kann. Einmal verdrahtet, expandiert der
Agent `ct` zu `cargo test`, bevor er es ausführt.

## Einrichtung

```sh
am context --setup claude
```

Legt `~/.claude/settings.json` an, falls nicht vorhanden, oder fügt sich
in eine vorhandene Datei ein, ohne andere Schlüssel anzutasten.
Idempotent: mehrfach ausführen ist sicher.

Für andere Agenten `am context` manuell aus deren Session-Start-Hook
aufrufen. Siehe die jeweilige Hook-Dokumentation.

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

Der Matcher `"startup|clear|compact"` sorgt dafür, dass die
Momentaufnahme auch nach `/clear` und `/compact` neu injiziert wird.
Ohne ihn verliert der Agent Deine Aliase beim ersten Clear.

Verifiziert mit Claude Code 2.1.126. Siehe [Anthropics
Hooks-Dokumentation](https://code.claude.com/docs/en/hooks) für die
volle Konfigurationsreferenz.

## Was Du in einer Claude-Code-Sitzung siehst

Öffne eine neue Sitzung in Deinem Projektverzeichnis. Der Agent sieht
jetzt Deine aktiven Aliase: `ll`, `gs`, `ct`, alles aus aktiven
Profilen, alles aus einer vertrauten `.aliases`-Datei.

Bitte ihn „lass die Tests laufen". Er führt `cargo test` aus, nicht
`ct`. Genauso bei `git pl` → `git pull --rebase`, `gst` → `git status`.

Subcommand-Aliase funktionieren auch. Der Agent weiß, dass `git pl` wie
ein echter Git-Subcommand aussieht, aber keiner ist, und führt
stattdessen die Expansion aus.

## Überprüfung

In einer frischen Sitzung fragen: **„welche Aliase habe ich?"**

Der Agent sollte sie direkt aus der Momentaufnahme auflisten, ohne ein
Kommando auszuführen. Wenn nicht, ist der Hook nicht gefeuert. Prüf
`~/.claude/settings.json`.

## Hinweise

- Die Markdown-Form von `am context` ist für ein Modell geschrieben.
  Die Form kann sich ändern, wenn die Modelle besser werden — schreib
  also keine Skripte dagegen.
- `am context --verbose` ergänzt die volle Shadow-Kette und etwaige
  Diagnosen zu ungültigen Aliasen.
