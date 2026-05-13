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
jetzt Deine aktiven Aliase, auch die aus aktiven Profilen und aus einer
vertrauten `.aliases`-Datei im Projekt.

Beispiel: Du hast einen Projekt-Alias `t → cargo test --all-features
--verbose`. Bitte Claude „lass die Tests laufen". Claude Code sieht `t`
im Kontext, expandiert ihn zu `cargo test --all-features --verbose` und
führt das aus. Der Agent kennt also die Variante von `cargo test`, die
Du in genau diesem Projekt (oder aus einem aktiven Profil) bevorzugst,
und muss nicht raten, welche Flags Du willst.

Subcommand-Aliase funktionieren auch. Etwa mit einem Git-Profil wie:

```
├─● git (active: 2)
│   ├─ tag → git tag {{1}} && git push o {{1}}
│   ╰─◆ git (subcommands)
│     ├─ cm → commit -S --signoff -m
│     ├─ pl → pull --rebase
│     ├─ psh → push
│     ╰─ st → status --short
```

Bittest Du Claude Code „zieh die neuesten Änderungen", sieht es `pl` im
Kontext und expandiert das zu `git pull --rebase`, bevor es läuft. Bei
„commite die Änderungen" entsprechend zur Expansion von `git cm`. Im
Terminal tippst Du die Kurzform, im Chat redest Du natürlich. Claude
Code übernimmt die Expansion.

## Überprüfung

In einer frischen Sitzung fragen: **„welche Aliase habe ich?"**

Der Agent sollte sie direkt aus der Momentaufnahme auflisten, ohne ein
Kommando auszuführen.

Oder frag „was würdest Du ausführen, um den Code zu testen?". Er sollte
mit der Expansion Deines `t`-Alias antworten.

Wenn nicht, ist der Hook nicht gefeuert. Prüf
`~/.claude/settings.json`.

## Hinweise

- Die Markdown-Form von `am context` ist für ein Modell geschrieben.
  Die Form kann sich ändern, wenn die Modelle besser werden. Schreib
  also keine Skripte dagegen.
- `am context --verbose` ergänzt die volle Shadow-Kette und etwaige
  Diagnosen zu ungültigen Aliasen.
