# KI-Assistenten

## Das Problem

Deine Aliase leben in Deiner interaktiven Shell. KI-Coding-Assistenten —
Claude Code, Codex, Cursor, GitHub Copilot CLI, Gemini CLI — führen
Kommandos in nicht-interaktiven Subshells aus, die **Dein Shell-Init
nicht laden**. Aus Sicht des Assistenten existieren Deine Aliase
schlicht nicht.

Das Ergebnis ist eine stille, tägliche Reibung:

- Du tippst `git cm "Fehler behoben"` im Chat. Der Assistent führt es
  wörtlich aus. Es scheitert mit `git: 'cm' is not a git command`.
- Du sagst „lass die Tests laufen". Der Assistent rät `cargo test`
  statt Deines projekteigenen `t` (`cargo test --all-features
  --workspace`).
- Du beobachtest, wie der Assistent die Langform von Befehlen tippt,
  die Du seit Jahren abgekürzt verwendest.

Du hast Deine Aliase gebaut, um weniger zu denken und zu tippen. Dieses
Vokabular sollte nicht verdunsten, sobald eine KI das Terminal betritt.

## Für wen das gedacht ist

**Du nutzt einen terminal-nativen KI-Assistenten neben Deiner Shell.**
Die Reibung zeigt sich, egal ob Du in einem Pane mit Claude Code
pair-programmierst, Cursor um einen Build bittest oder einen
langlaufenden Agenten autonom arbeiten lässt, während Du zusiehst.

**Du hast in amoxide-Aliase investiert.** Profile, projekteigene
`.aliases`-Dateien, Subcommand-Aliase — Dein Tipp-Vokabular ist dicht.
Je mehr Kurzformen Du gebaut hast, desto mehr Mehrwert gibt `am
context` zurück.

**Du willst keinen laufenden Aufwand.** Das ist kein Tool, das Du pro
Kommando aufrufst. Einmal einrichten, vergessen. Der Assistent
bekommt bei jedem Sitzungsstart eine frische Momentaufnahme.

## Was `am context` tut

Es gibt eine kompakte Markdown-Momentaufnahme Deines **aktuell
wirksamen** Alias-Sets aus — derselbe Satz, den Deine Shell gerade
sieht, nachdem die Präzedenz zwischen Global, Profilen und der
projekteigenen `.aliases`-Datei aufgelöst wurde. Diese Ausgabe
verdrahtest Du in den Session-Start-Hook Deines Assistenten. Von da an
hat der Assistent Deine Alias-Map im Kontext und kann Kurzformen vor
der Ausführung expandieren.

Die Momentaufnahme bringt dem Modell bei, sich selbst zu nutzen — vier
nummerierte Regeln am Anfang sagen dem Assistenten, wann er einen
Namen expandieren soll, worauf er achten muss (Subcommand-Aliase, die
wie echte Subcommands aussehen, aber keine sind) und wie er sich von
`command not found`-Fehlern erholt.

## Was Du zu sehen bekommst

Eine echte Momentaufnahme sieht so aus:

````markdown
# amoxide aliases (active set, cwd: /Users/du/projects/dein-app)
#
# ## How to use this snapshot
#
# When the user mentions a name from the `Aliases` table below in any context —
# running a command, suggesting one, asking what it does — treat the `expands to`
# value as the canonical form.
#
# 1. Recognise aliases by name match. If the user's input contains a token that
#    matches a `name` from the table — including multi-word names with a space,
#    like `git pl` — it is an alias. Expand it before running.
#
# 2. Subcommand aliases are deceptive. A name like `git pl` looks like a real
#    git subcommand but is not. Running `git pl` verbatim in a subshell fails
#    with `git: 'pl' is not a git command`. Always run the value from
#    `expands to` (`git pull --rebase`), never the alias text.
#
# 3. Recover from `command not found` failures. If a shell command fails because
#    the name is unknown, check this table — the user's shell sees the alias
#    but your subshell does not.
#
# 4. In chat, the user's vocabulary is fine. When suggesting commands in
#    conversation, the short form (`git cm "msg"`) matches the user's mental
#    model. When *running* it in a subshell, use the canonical form.
#
# Precedence (highest first): project > profile(rust, prio 1) > profile(git, prio 2) > global
#
# Templates: {{N}} is a positional placeholder (1-indexed).
# Variables: {{name}} tokens are already substituted in the table below.

## Aliases

| name    | expands to                                 | from         |
|---------|--------------------------------------------|--------------|
| f       | cargo fmt                                  | project      |
| git pl  | git pull --rebase                          | profile:git  |
| gm      | git commit -S --signoff -m                 | profile:git  |
| t       | cargo test --all-features                  | project      |
| tag     | git tag {{1}} && git push o {{1}}          | profile:git  |
| ll      | ls -lha                                    | global       |
````

Hinweis: Der Inhalt der Momentaufnahme bleibt englisch — er wird vom
Modell konsumiert, nicht direkt vom Menschen gelesen, und die KI-Modelle
sind im Englischen am verlässlichsten.

Ein paar Details, die wichtig sind:

- **Die `from`-Spalte** sagt dem Assistenten, woher jeder Alias kommt,
  damit er „warum macht `f` das?" ohne zweite Rückfrage beantworten
  kann.
- **Subcommand-Aliase sind flach** (`git pl → git pull --rebase`),
  nicht verschachtelt. Der Assistent sieht sie wie gewöhnliche
  Einträge.
- **Präzedenz ist bereits angewendet**: Wenn `f` sowohl in einem Profil
  als auch in der projekteigenen `.aliases` definiert ist, steht nur
  der Gewinner in der Tabelle.
- **Templates bleiben erhalten**: `{{1}}` bleibt wörtlich, weil es ein
  Positionsplatzhalter ist, der durch die tatsächlichen Argumente des
  Nutzers gefüllt wird.

## Einrichtung

### Option 1 — automatisch

```sh
am context --setup claude
```

Idempotent. Legt `~/.claude/settings.json` an, falls nicht vorhanden,
oder fügt sich in eine vorhandene Datei ein, ohne andere Schlüssel oder
Hook-Events anzutasten. Beim erneuten Ausführen wird ein vorhandener
Eintrag erkannt und nichts geändert.

Das ist die gesamte Einrichtung. Starte eine neue Claude-Code-Sitzung
und Dein Assistent hat die Momentaufnahme.

### Option 2 — manuell

Wenn Du genau sehen willst, was sich ändert, füge das selbst in
`~/.claude/settings.json` ein:

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

Der Matcher `"startup|clear|compact"` ist wichtig. Ohne ihn wird die
Momentaufnahme nur beim Kaltstart injiziert — der Assistent verliert
Deine Alias-Map beim ersten `/clear` oder `/compact` mitten in der
Sitzung.

### Überprüfung

Öffne eine neue Claude-Code-Sitzung in Deinem Projektverzeichnis und
frag: „Welche Aliase habe ich?" Wenn die Verdrahtung geklappt hat,
listet der Assistent sie direkt aus der Momentaufnahme auf, ohne ein
Kommando auszuführen.

## Andere Assistenten

`am context` ist generisch — seine Standardausgabe funktioniert als
Session-Start-Kontext für jeden Assistenten, dessen Harness das
Ausführen eines Kommandos beim Sitzungsstart unterstützt. Solange
native `--setup <assistant>`-Unterstützung pro Assistent noch nicht
gelandet ist, schau in deren Dokumentation nach dem Gegenstück zur
Hook-Konfiguration von Claude Code:

- **Codex CLI / Codex App**
- **Cursor**
- **GitHub Copilot CLI**
- **Gemini CLI**

Der Hook-Inhalt ist immer derselbe: `am context` ausführen und seine
Standardausgabe als Sitzungskontext einspeisen.

## Output-Stabilität

Die Markdown-Form von `am context` ist **keine** stabile API. Das
Format kann sich aus Gründen der Modell-Verständlichkeit ohne
Vorankündigung ändern. Skripte gegen diesen Output zu schreiben ist
nicht empfohlen — für maschinenlesbare Formate bitte ein [Issue
öffnen](https://github.com/sassman/amoxide-rs/issues).

## Siehe auch

- `am context --verbose` zeigt die volle Shadow-Kette und etwaige
  Diagnosen zu ungültigen Aliasen — nützlich, wenn Du den Assistenten
  fragen willst, „warum macht `f` die Projektversion?"
- [Projekt-Aliase](/de/usage/project-aliases) — `.aliases`-Dateien
  geben Dir Aliase pro Repo, die der Assistent automatisch
  übernimmt.
- [Variables](/usage/variables) — `{{name}}`-Substitutionen werden in
  die Momentaufnahme eingebacken, sodass der Assistent den aufgelösten
  Befehl sieht.
