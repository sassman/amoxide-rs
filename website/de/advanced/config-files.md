# Konfigurationsdateien

amoxide speichert seine Konfiguration in `~/.config/amoxide/` als TOML-Dateien. Du musst diese selten manuell bearbeiten — das CLI verwaltet sie — aber das Verständnis des Formats hilft beim Debugging oder Teilen von Setups.

## Dateiübersicht

| Datei | Zweck |
|-------|-------|
| `config.toml` | Globale Aliase und aktive Profilliste |
| `profiles.toml` | Alle Profildefinitionen und deren Aliase |
| `security.toml` | Vertrauensentscheidungen für Projekt-`.aliases`-Dateien <VersionBadge v="0.5.0" /> |
| `.aliases` | Projektlokale Aliase (liegt im Projektstamm) |

## `config.toml` — Globale Konfiguration

```toml
# Welche Profile aktuell aktiv sind, in Prioritätsreihenfolge
active_profiles = ["git", "rust"]

# Globale Aliase — immer verfügbar
[aliases]
helo = "echo hello world"
ll = "ls -lha"

# Globale Subcommand-Aliase — Kurzformen für subcommandbasierte Tools
[subcommands]
"jj:ab" = ["abandon"]
"jj:b:l" = ["branch", "list"]
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

[profiles.subcommands]
"cargo:t" = ["test", "--all-features"]

[[profiles]]
name = "node"

[profiles.aliases]
t = "npm run test"
b = "npm run build"
```

Jeder `[[profiles]]`-Block definiert ein benanntes Profil mit seinen Aliasen und optionalen Subcommand-Aliasen. Beachte, dass verschiedene Profile den gleichen Alias-Namen verwenden können (z.B. `t` in `rust` und `node`) — welches Profil höhere Priorität in `active_profiles` hat, gewinnt.

## `security.toml` — Vertrauensentscheidungen

<VersionBadge v="0.5.0" />

Verfolgt, welche Projekt-`.aliases`-Dateien du überprüft und als vertrauenswürdig markiert hast. Wird automatisch von `am trust` und `am untrust` verwaltet — du solltest diese Datei nicht manuell bearbeiten müssen.

```toml
[[trusted]]
path = "/home/user/projects/my-app/.aliases"
hash = "a1b2c3d4e5f6..."

[[untrusted]]
path = "/home/user/projects/declined-repo/.aliases"
```

Jeder vertrauenswürdige Eintrag speichert den Dateipfad und einen BLAKE3-Hash des Inhalts. Wenn sich die Datei ändert, stimmt der Hash nicht mehr überein und amoxide fordert dich auf, sie erneut zu überprüfen. Siehe [Vertrauensmodell](/de/usage/project-aliases#vertrauensmodell) für Details.

Ein dritter Abschnitt, `[[tampered]]`, erscheint automatisch, wenn eine vertrauenswürdige Datei außerhalb von `am` geändert wird. Er verschwindet, wenn du `am trust` ausführst, um die Änderungen zu überprüfen.

## `.aliases` — Projekt-Aliase

Diese Datei liegt im Projektstamm und wird automatisch geladen, wenn du in das Verzeichnis wechselst.

```toml
[aliases]
i = "cargo install --path crates/am && cargo install --path crates/am-tui"
l = "cargo clippy --locked --all-targets -- -D warnings"
t = "cargo test --all-features"

[subcommands]
"jj:ab" = ["abandon"]
"jj:b:l" = ["branch", "list"]
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

Die gleiche Prioritätsreihenfolge gilt für Subcommand-Aliase. Ein `[subcommands]`-Eintrag in `.aliases` überschreibt denselben Schlüssel aus einem aktiven Profil, der wiederum denselben Schlüssel in `config.toml` überschreibt.

Siehe [Subcommand-Aliase](/de/usage/subcommand-aliases) für Nutzungsbeispiele und wie die Shell-Wrapper generiert werden.
