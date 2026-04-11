# Projekt-Aliase

Projekt-Aliase leben in einer `.aliases`-Datei im Projektstamm. Sie werden automatisch geladen, wenn du in das Verzeichnis wechselst, und entladen, wenn du es verlässt — wie [direnv](https://direnv.net), aber für Aliase.

## Projekt-Aliase hinzufügen

Verwende das `-l` (lokal) Flag:

```sh
cd ~/my-project
am add -l t "./x.py test"
am add -l b "./x.py build"
```

Wenn keine `.aliases`-Datei existiert, wird eine im aktuellen Verzeichnis erstellt. Wenn eine `.aliases`-Datei weiter oben im Verzeichnisbaum existiert, wirst du gefragt, ob du die Aliase dort hinzufügen möchtest.

## Die `.aliases`-Datei

Du kannst die Datei auch direkt erstellen oder bearbeiten:

```toml
# /pfad/zu/meinem-projekt/.aliases
[aliases]
t = "./x.py test"
b = "./x.py build"
```

## Vertrauensmodell

<VersionBadge v="0.5.0" />

Projekt-`.aliases`-Dateien können beliebige Shell-Befehle enthalten. Da jeder eine `.aliases`-Datei in ein Repository legen könnte, verlangt amoxide, dass du jede Datei explizit als vertrauenswürdig markierst, bevor ihre Aliase geladen werden — ähnlich wie [direnv](https://direnv.net) mit `.envrc`-Dateien umgeht.

### Erster Kontakt

Wenn du in ein Verzeichnis mit einer nicht vertrauenswürdigen `.aliases`-Datei wechselst, siehst du:

```
am: .aliases found but not trusted. Run 'am trust' to review and allow.
```

Es werden keine Aliase geladen, bis du sie überprüft und genehmigt hast.

### Überprüfen und vertrauen

Führe `am trust` aus, um die Aliase zu überprüfen:

```
❯ am trust
Reviewing .aliases at /home/user/projects/my-app

  b  → make build
  t  → cargo test
  cb → cargo build

Trust these aliases? [Y/n]
```

Falls die Datei verdächtige Inhalte enthält (versteckte Escape-Sequenzen oder Steuerzeichen), wird vor der Abfrage eine Warnung angezeigt.

Mit **Ja** wird die Datei als vertrauenswürdig markiert — die Aliase werden sofort geladen:

```
am: loaded .aliases
  b  → make build
  t  → cargo test
  cb → cargo build
```

Mit **Nein** wird das Verzeichnis als nicht vertrauenswürdig markiert. Zukünftiges `cd` dorthin bleibt still — keine Warnungen, keine Aliase.

### Vertrauen widerrufen

```sh
am untrust          # als nicht vertrauenswürdig markieren (still bei cd)
am untrust --forget # aus der Verfolgung entfernen (wird erneut nachfragen)
```

### Manipulationserkennung

amoxide speichert einen kryptographischen Hash (BLAKE3) jeder vertrauenswürdigen `.aliases`-Datei. Wenn die Datei außerhalb von `am` geändert wird, stimmt der Hash nicht mehr überein:

```
am: .aliases was modified since last trusted. Run 'am trust' to review and allow.
```

Das passiert, wenn die Datei manuell bearbeitet, durch `git pull` aktualisiert oder von einem anderen Tool als `am` geändert wird. Die Warnung erscheint bei jedem `cd` wieder, bis du die Änderungen mit `am trust` überprüfst.

Wenn du `am` selbst zum Ändern der Datei verwendest — über `am add -l` oder `am remove -l` — wird der Hash automatisch aktualisiert, sodass diese Änderungen die Warnung nie auslösen.

### Lade- und Entlademeldungen

Wenn Aliase geladen werden, siehst du welche Befehle verfügbar wurden:

```
am: loaded .aliases
  b  → make build
  t  → cargo test
```

Wenn du das Projekt verlässt:

```
am: unloaded .aliases: b, t
```

Diese Meldungen erscheinen nur beim Betreten oder Verlassen des Verzeichnisses mit der `.aliases`-Datei — nicht beim Navigieren in Unterverzeichnissen desselben Projekts.

## Wie es funktioniert

Der `am init` Shell-Hook ruft `am hook <shell>` bei jedem Verzeichniswechsel auf. Der Hook:

1. Sucht vom aktuellen Verzeichnis aufwärts nach einer `.aliases`-Datei (stoppt vor `$HOME`)
2. Prüft, ob die Datei vertrauenswürdig ist (Pfad + Hash in `security.toml`)
3. Falls vertrauenswürdig: entlädt vorherige Projekt-Aliase und lädt die neuen
4. Falls nicht vertrauenswürdig: zeigt eine Warnung oder bleibt still, je nach Vertrauensstatus

## Workflow

Ein natürlicher Workflow: mit Projekt-Aliasen starten, dann Duplikate in Profile auslagern:

**Schritt 1:** Projektspezifische Aliase hinzufügen:

```sh
am add -l t cargo test
am add -l l cargo clippy --all-targets -- -D warnings
am add -l i cargo install --path .
```

**Schritt 2:** `t` und `l` sind in jedem Rust-Projekt gleich. In ein Profil extrahieren:

```sh
am profile add rust
am add -p rust t cargo test
am add -p rust l cargo clippy --all-targets -- -D warnings
am profile use rust
```

Jetzt behält die `.aliases`-Datei nur wirklich projektspezifische Aliase wie `i`.

::: tip
`am tui` ermöglicht es, Aliase visuell zwischen Projekt- und Profil-Ebene zu verschieben — Alias auswählen und `m` drücken.
:::

## Aliase mit dem TUI verschieben

Verwende `am tui` um Aliase visuell von der Projekt-Ebene in ein Profil zu verschieben — Alias auswählen und `m` drücken:

<video autoplay loop muted playsinline>
  <source src="/am-tui-moving-aliases.webm" type="video/webm">
  <source src="/am-tui-moving-aliases.mp4" type="video/mp4">
</video>
