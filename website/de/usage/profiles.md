# Profile

Profile sind benannte Gruppen von Aliasen. Stelle sie dir als Ebenen vor — du kannst mehrere gleichzeitig aktivieren, und später aktivierte Profile überschreiben frühere bei Namenskonflikten.

## Profil erstellen

```sh
am profile add rust
am p a rust          # Kurzform
```

## Aliase zu einem Profil hinzufügen

```sh
am add -p rust ct "cargo test"
am add -p rust cb "cargo build"
```

## Profile aktivieren

```sh
# Profil aktivieren (wird oben auf den Stack gelegt)
am profile use rust
am p u rust          # Kurzform

# An bestimmter Position aktivieren (1 = Basis-Ebene)
am profile use git -n 1
```

Bei mehreren aktiven Profilen werden sie gestapelt. Das zuletzt aktivierte Profil gewinnt bei Konflikten:

```sh
am profile use git    # Basis-Ebene (active: 1)
am profile use rust   # darüber (active: 2)
# Wenn beide den Alias "t" haben, gewinnt die rust-Version
```

## Profile auflisten

```sh
am profile           # Standardaktion
am profile list      # explizit
am l                 # kürzeste Form
```

## Profil entfernen

```sh
am profile remove rust     # fragt nach Bestätigung, wenn Aliase vorhanden
am p r rust -f             # Bestätigung überspringen
```

## Aliase hinzufügen und entfernen

```sh
# Zum aktiven Profil hinzufügen
am add gs git status

# Zu einem bestimmten Profil hinzufügen
am add -p rust ct cargo test

# Vom aktiven Profil entfernen
am remove gs
am r gs              # Kurzform

# Von einem bestimmten Profil entfernen
am remove -p rust ct
```

::: tip
Alle Verben haben Kurzformen: `am a` für add, `am r` für remove, `am p a` für profile add, `am p u` für profile use.
:::

## Profile visuell verwalten

<!-- TODO: Screenshot von am-tui mit Profil-Liste, aktiven Indikatoren und dem Verschiebe-Modus -->
::: info Screenshot kommt bald
Das TUI zeigt Profil-Verwaltung — Profile aktivieren, Aliase zwischen ihnen verschieben und die geschichtete Hierarchie auf einen Blick sehen.
:::
