# Update-Prüfung <VersionBadge v="0.9.0" />

Wenn eine neuere amoxide-Version auf crates.io erscheint, sagt dir `am ls` beim nächsten Listing Bescheid. Der Hinweis geht auf stderr, die Prüfung läuft im Hintergrund, und du kannst sie abschalten.

## Was du siehst

Wenn du hinterher bist, geben die Listing-Befehle (`am ls`, `am la`, `am profile list`) eine Zeile auf stderr aus, nach der Liste:

```text
am: 💡 a new version is available: v0.9.0 -> visit https://github.com/sassman/amoxide-rs/releases
```

Wenn du aktuell bist, siehst du nichts.

## So funktioniert es

- Beim ersten Listing-Aufruf startet amoxide einen abgekoppelten Hintergrundprozess, der crates.io anfragt. Die Liste erscheint sofort — du wartest nicht aufs Netzwerk.
- Das Ergebnis landet in `~/.cache/amoxide/update-check.toml`.
- Beim nächsten Listing liest amoxide diese Datei. Ist sie älter als 24 Stunden, läuft im Hintergrund eine neue Prüfung, und das vorherige Ergebnis (falls vorhanden) wird inzwischen angezeigt.
- Netzwerkfehler bleiben stumm. Offline, DNS kaputt, crates.io down — die Liste funktioniert, nur der Hinweis erscheint nicht.

Der Hinweis steht auf stderr, also bleiben Pipes unberührt:

```bash
am ls | grep mein-alias   # der Hinweis fällt nicht in die Pipe
```

## Prüfung deaktivieren

Zwei Wege: ein Konfigurations-Flag (dauerhaft) oder eine Umgebungsvariable (einmalig, für CI).

### Konfiguration: `~/.config/amoxide/config.toml`

```toml
[update]
check = false
```

Mit `check = false` gibt es kein Cache-Lesen, kein Spawn, keinen Netzwerkaufruf. Die Cache-Datei wird gar nicht erst angelegt.

### Umgebung: `AM_NO_UPDATE_CHECK`

```bash
AM_NO_UPDATE_CHECK=1 am ls
```

Jeder nicht-leere Wert überspringt die Prüfung für diesen einen Aufruf. Praktisch in CI, wo du weder Netzwerkverkehr noch eine Konfigurationsdatei pflegen willst.

## Datenschutz

Ein HTTPS-GET an `https://crates.io/api/v1/crates/amoxide`. Mehr nicht — Crate-Name plus der Standard-User-Agent von ureq. Nichts identifiziert dich, nichts wird getrackt.

## Siehe auch

- [Konfigurationsdateien](/de/advanced/config-files) — vollständige TOML-Referenz
