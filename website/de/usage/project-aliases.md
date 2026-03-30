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

## Wie es funktioniert

Der `am init` Shell-Hook ruft `am hook <shell>` bei jedem Verzeichniswechsel auf. Der Hook:

1. Sucht vom aktuellen Verzeichnis aufwärts nach einer `.aliases`-Datei (stoppt vor `$HOME`)
2. Entlädt alle zuvor aktiven Projekt-Aliase
3. Lädt die Aliase des neuen Projekts

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

<!-- TODO: Screenshot von am-tui im Verschiebe-Modus, ein Alias wird von Projekt-Ebene zu einem Profil verschoben -->
::: info Screenshot kommt bald
Das TUI im Verschiebe-Modus — einen Projekt-Alias auswählen und mit einem Tastendruck in ein wiederverwendbares Profil verschieben.
:::
