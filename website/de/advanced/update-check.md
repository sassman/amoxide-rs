# Update-Prüfung

amoxide prüft crates.io regelmäßig auf eine neuere Version und gibt einen einzeiligen Hinweis aus, wenn eine verfügbar ist. Die Prüfung ist **nicht blockierend**, **24 Stunden im Cache** und **vollständig abschaltbar**.

## Was du siehst

Wenn eine neuere Version verfügbar ist, geben die Listing-Befehle (`am ls`, `am la`, `am profile list`) eine einzelne Zeile auf **stderr** nach der Liste aus:

```text
am: 💡 a new version is available: v0.9.0 -> visit https://github.com/sassman/amoxide-rs/releases
```

Wenn du bereits auf der neuesten Version bist, wird nichts ausgegeben.

## So funktioniert es

- Beim ersten Aufruf eines Listing-Befehls startet amoxide einen abgekoppelten Hintergrundprozess, der crates.io anfragt. Die Ausgabe von `am ls` erscheint sofort — du wartest auf nichts.
- Das Ergebnis wird in eine lokale Cache-Datei unter `~/.cache/amoxide/update-check.toml` geschrieben.
- Folgeaufrufe lesen diesen Cache. Ist er älter als 24 Stunden, wird eine neue Hintergrundprüfung gestartet und das gecachte Ergebnis (falls vorhanden) inzwischen angezeigt.
- Alle Fehler (offline, DNS-Fehler, Timeout, fehlerhafte Antwort) werden lautlos geschluckt. Der Listing-Befehl scheitert nie wegen der Update-Prüfung.

Da der Hinweis auf stderr geht, funktionieren Skripte, die `am ls` weiterpipen, unverändert:

```bash
am ls | grep mein-alias   # vom Hinweis nicht betroffen
```

## Prüfung deaktivieren

Du kannst die Update-Prüfung per **Konfiguration** (einmalig) oder **Umgebungsvariable** (pro Aufruf, ideal für CI) abschalten.

### Konfiguration: `~/.config/amoxide/config.toml`

```toml
[update]
check = false
```

Wenn `check = false` gesetzt ist, finden weder Cache-Lesen noch Hintergrund-Spawn statt. Es wird kein Netzwerkaufruf gemacht und keine Cache-Datei angelegt.

### Umgebung: `AM_NO_UPDATE_CHECK`

```bash
AM_NO_UPDATE_CHECK=1 am ls
```

Setze diese auf einen beliebigen nicht-leeren Wert, um die Prüfung für einen Aufruf zu überspringen. Nützlich in CI-Umgebungen, wo du weder einen Netzwerkaufruf noch eine Konfigurationsänderung möchtest.

## Datenschutz

Die Prüfung sendet eine HTTPS-Anfrage an `https://crates.io/api/v1/crates/amoxide`. Die einzigen Daten, die dein Gerät verlassen, sind der Crate-Name und der von ureq gesetzte User-Agent. Keine Telemetrie, keine Identifikatoren, keine Version deines lokalen Binarys wird übertragen.

## Siehe auch

- [Konfigurationsdateien](/de/advanced/config-files) — vollständige TOML-Referenz
