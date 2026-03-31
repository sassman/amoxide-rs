# Aliase verketten

Aliase können andere Aliase referenzieren. Da amoxide-Aliase von der Shell aufgelöst werden, kannst du leistungsstarke Befehlsketten aufbauen, indem du einfache Aliase aufeinander aufbaust.

## Grundlegende Komposition

Definiere einen Basis-Alias und baue darauf auf:

```sh
# Basis: ein signierter Commit
am add -p git cm "git commit -S --signoff -m"

# Darauf aufbauend: Conventional-Commit-Präfixe
am add -p git cmf "cm feat:"
am add -p git cmx "cm fix:"
am add -p git cmd "cm docs:"

cmf Benutzer-Authentifizierung hinzufügen
# → cm feat: Benutzer-Authentifizierung hinzufügen
# → git commit -S --signoff -m feat: Benutzer-Authentifizierung hinzufügen
```

Ändere den Basis-Alias `cm` einmal, und alle Varianten (`cmf`, `cmx`, `cmd`) übernehmen die Änderung.

## Profil-übergreifende Komposition

Aliase aus verschiedenen Profilen können sich gegenseitig referenzieren, solange beide Profile aktiv sind:

```sh
# git-Profil — Basis-Werkzeuge
am add -p git ga "git add"
am add -p git gc "git commit -S --signoff -m"

# workflow-Profil — übergeordnete Abkürzungen
am add -p workflow wip "ga -A && gc wip"
am add -p workflow ship "ga -A && gc"

# beide aktivieren
am profile use git
am profile use workflow

ship bereit zum Mergen
# → ga -A && gc bereit zum Mergen
# → git add -A && git commit -S --signoff -m bereit zum Mergen
```

## Mischung mit Projekt-Aliasen

Projekt-Aliase können auch Profil-Aliase referenzieren. Ein Rust-Projekt könnte definieren:

```sh
# Profil: immer verfügbar
am add -p rust t "cargo test"
am add -p rust l "cargo clippy --locked --all-targets -- -D warnings"

# Projekt-lokal: baut auf dem Profil-Alias auf
am add -l check "l && t"
```

Jetzt führt `check` Clippy und dann Tests aus — und wenn du `l` oder `t` im Profil änderst, übernimmt der Projekt-Alias die Änderung.

## Tipps

- Halte Basis-Aliase einfach und fokussiert — ein Befehl, ein Zweck
- Benenne verkettete Aliase so, dass die Kette erkennbar ist (`cm` → `cmf` für "cm feat")
- Verwende `am ls` um zu sehen, welche Aliase verfügbar sind und aus welcher Ebene
