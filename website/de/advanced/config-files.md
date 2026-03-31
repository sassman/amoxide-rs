# Konfigurationsdateien

amoxide speichert seine Konfiguration in `~/.config/amoxide/` als TOML-Dateien. Du musst diese selten manuell bearbeiten — das CLI verwaltet sie — aber das Verständnis des Formats hilft beim Debugging oder Teilen von Setups.

## Dateiübersicht

| Datei | Zweck |
|-------|-------|
| `config.toml` | Globale Aliase und aktive Profilliste |
| `profiles.toml` | Alle Profildefinitionen und deren Aliase |
| `.aliases` | Projektlokale Aliase (liegt im Projektstamm) |

## `config.toml` — Globale Konfiguration

```toml
# Welche Profile aktuell aktiv sind, in Prioritätsreihenfolge
active_profiles = ["git", "rust"]

# Globale Aliase — immer verfügbar
[aliases]
helo = "echo hello world"
ll = "ls -lha"
```

Das `active_profiles`-Array bestimmt, welche Profile geladen werden und ihre Priorität. Der letzte Eintrag hat die höchste Priorität — wenn sowohl `git` als auch `rust` einen Alias mit dem gleichen Namen definieren, gewinnt `rust`.

## `profiles.toml` — Profildefinitionen

```toml
[[profiles]]
name = "git"

[profiles.aliases]
ga = "git commit --amend"
gcm = "git commit -S --signoff -m"
gst = "git status"

[[profiles]]
name = "rust"

[profiles.aliases]
f = "cargo fmt"
t = "cargo test --all-features"
l = "cargo clippy --locked --all-targets -- -D warnings"

[[profiles]]
name = "node"

[profiles.aliases]
t = "npm run test"
b = "npm run build"
```

Jeder `[[profiles]]`-Block definiert ein benanntes Profil mit seinen Aliasen. Beachte, dass verschiedene Profile den gleichen Alias-Namen verwenden können (z.B. `t` in `rust` und `node`) — welches Profil höhere Priorität in `active_profiles` hat, gewinnt.

## `.aliases` — Projekt-Aliase

Diese Datei liegt im Projektstamm und wird automatisch geladen, wenn du in das Verzeichnis wechselst.

```toml
[aliases]
i = "cargo install --path crates/am && cargo install --path crates/am-tui"
l = "cargo clippy --locked --all-targets -- -D warnings"
t = "cargo test --all-features"
```

Projekt-Aliase überschreiben Profil-Aliase mit dem gleichen Namen. So kannst du Abkürzungen pro Projekt anpassen, ohne dein globales Setup zu ändern.

## Prioritätsreihenfolge

Wenn mehrere Ebenen denselben Alias-Namen definieren, gewinnt die spezifischste:

```
Projekt-Aliase (.aliases)     ← höchste Priorität
  ↑ überschreibt
Aktive Profile (letztes gewinnt)
  ↑ überschreibt
Globale Aliase (config.toml)  ← niedrigste Priorität
```
