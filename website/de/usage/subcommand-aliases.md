# Subcommand-Aliase <VersionBadge v="0.5.0" />

Viele Tools wie `jj`, `git`, `cargo` und `kubectl` organisieren ihre Befehle als Subcommands. Subcommand-Aliase ermöglichen es, Kurzformen für diese Subcommands zu erstellen — ohne Shell-Completions oder das Weiterleitungsverhalten zu verlieren.

```sh
jj ab          # → jj abandon
jj b l         # → jj branch list
```

amoxide generiert pro Programm eine Shell-Funktion, die den Subcommand abfängt und auf die vollständige Form weiterleitet.

## Subcommand-Aliase hinzufügen

Verwende `--sub`, um jedes kurze Token mit seiner langen Expansion zu verknüpfen:

```sh
am add -g jj --sub ab abandon
# jj ab → jj abandon

am add -g jj --sub b branch --sub l list
# jj b l → jj branch list
```

Die Doppelpunkt-(`:`)-Notation ist eine Kurzform dafür — Programm und alle kurzen Token durch Doppelpunkte verbunden, gefolgt von der Expansion:

```sh
am add -g jj:ab abandon
am add -g jj:b:l branch list
```

::: tip
Kurzform: `am a -g jj:ab abandon`
:::

Die Scope-Flags funktionieren genauso wie bei regulären Aliasen:

| Flag | Scope |
|------|-------|
| `-g` / `--global` | Global — immer aktiv |
| `-p <profil>` | Profil — aktiv wenn das Profil aktiviert ist |
| `-l` / `--local` | Projekt — aus `.aliases` geladen |

## Verschachtelte Subcommand-Aliase

Füge weitere durch Doppelpunkte getrennte Token für verschachtelten Dispatch hinzu:

```sh
am add -g jj:b:l branch list
am add -g jj:b:c branch create
am add -g kubectl:get:po "get pods"
```

Wenn du `jj b l` ausführst, leitet amoxide durch `jj → b → l` weiter und expandiert zu `jj branch list "$@"`.

## Templates in Expansionen

Expansionen unterstützen dieselben Templates wie reguläre Aliase:

```sh
am add -g jj:anon "log -r 'anon()'"
# jj anon → jj log -r 'anon()'

am add -g jj:edit "edit {{1}}"
# jj edit abc123 → jj edit abc123
```

Siehe [Parametrisierte Aliase](/de/advanced/parameterized-aliases) für die vollständige Template-Referenz.

## Subcommand-Aliase entfernen

Verwende dieselbe Doppelpunkt-Notation mit `am remove`:

```sh
am remove -g jj:ab
am remove -g jj:b:l
```

Kurzform: `am r -g jj:ab`

## Wie es funktioniert

Beim Shell-Init (und bei jedem `am sync`, ausgelöst durch `cd` oder eine `am`-Änderung) generiert amoxide eine Wrapper-Funktion für jedes Programm mit Subcommand-Aliasen:

```sh
# generiert für jj (bash/zsh)
jj() {
  case "$1" in
    ab) shift; command jj abandon "$@" ;;
    b)
      case "$2" in
        l) shift 2; command jj branch list "$@" ;;
        *) command jj "$@" ;;
      esac
      ;;
    *) command jj "$@" ;;
  esac
}
```

Jeder Subcommand, der keinem definierten Alias entspricht, wird unverändert an das echte `jj` weitergeleitet. Zusätzliche Argumente werden nach der Expansion immer weitergeleitet.

## Die `.aliases`-Datei

Projektlokale Subcommand-Aliase verwenden einen `[subcommands]`-Abschnitt neben `[aliases]`:

```toml
# .aliases
[aliases]
t = "cargo test"

[subcommands]
"jj:ab" = ["abandon"]
"jj:b:l" = ["branch", "list"]
"jj:anon" = ["log -r 'anon()'"]
```

Der Schlüssel ist der durch Doppelpunkte verbundene Pfad, und der Wert ist ein Array von Expansions-Token. Das gleiche Dateiformat gilt für `config.toml` (global) und `profiles.toml` (pro Profil).

## Vertrauensmodell

Projektbezogene Subcommand-Aliase unterliegen demselben Vertrauensmodell wie reguläre Projekt-Aliase. Wenn du `am trust` ausführst, zeigt amoxide sowohl die regulären Aliase als auch die Subcommand-Aliase an — damit siehst du immer genau, was du genehmigst.

Siehe [Projekt-Aliase — Vertrauensmodell](/de/usage/project-aliases#vertrauensmodell) für Details.

## Subcommand-Aliase auflisten

`am ls` zeigt Subcommand-Aliase gruppiert nach Programm in der Baumansicht:

```
🌐 global
│  ├─ ll → ls -lha
│  ╰─◆ jj (subcommands)
│    ├─ ab → abandon
│    ╰─ b l → branch list
│
╰─📁 project (/path/to/project/.aliases)
  ├─ t → cargo test
  ╰─◆ cargo (subcommands)
    ╰─ test → test --test {{1}} -- {{@}}
```

Die TUI (`am tui`) ermöglicht es, Subcommand-Aliase interaktiv zu verwalten.
