# Shell-Tab-Vervollständigung <VersionBadge v="0.10.0" />

amoxide vervollständigt Profilnamen, Aliasnamen, Subcommand-Alias-Segmente und Variablennamen mit Tab — überall dort, wo du sie eingeben würdest.

## Was wird wo vervollständigt

| Du tippst | Tab liefert |
|---|---|
| `am use <TAB>` | aktive und inaktive Profilnamen |
| `am profile use <TAB>`, `am profile remove <TAB>` | Profilnamen |
| `am remove <TAB>` | Aliasnamen aus dem aktuellen Kontext (global + aktive Profile + Projekt) |
| `am remove -p rust <TAB>` | nur Aliase aus dem Profil `rust` |
| `am remove -g <TAB>` | nur globale Aliase |
| `am remove -l <TAB>` | nur Projekt-Aliase (lokal) |
| `am remove jj --sub <TAB>` | das nächste Segment einer Subcommand-Alias-Kette (z. B. `b`, `ab`) |
| `am var get <TAB>`, `am var unset <TAB>` | Variablennamen aus dem aktuellen Kontext, durch `-p` / `-l` / `-g` eingeschränkt |
| `am export -p <TAB>`, `am import -p <TAB>`, `am share -p <TAB>` | Profilnamen |

## Wie es funktioniert

`am init <shell>` enthält eine einzeilige Registrierung, die amoxide in das Vervollständigungssystem deiner Shell einbindet. Wenn du Tab drückst, fragt die Shell `am`, was an dieser Stelle gültig ist. Die Abfrage läuft gegen deine aktive Konfiguration, also bleiben Vervollständigungen ohne Cache-Invalidierung aktuell.

Nichts zusätzlich zu installieren oder zu sourcen. Wenn du `am setup` bereits ausgeführt hast, wird die Vervollständigung beim nächsten Öffnen einer Shell aktiv.

## Unterstützte Shells

- **Bash** (3.2+)
- **Zsh**
- **Fish**
- **PowerShell** (5.1 + 7)
- **Brush** — verwendet die Bash-Anbindung

Nushell wird noch nicht unterstützt; der Upstream-Stand ist in [`clap-rs/clap#5841`](https://github.com/clap-rs/clap/pull/5841).

## Fehlerbehebung

### Vervollständigung funktioniert nach dem Upgrade nicht

Öffne eine neue Shell oder führe in der aktuellen Sitzung erneut `eval "$(am init <shell>)"` aus. Die Registrierung wird über die `am init`-Ausgabe geladen; alte Shells sehen sie erst nach erneuter Auswertung.

### Registrierung ist da, aber Tab passiert nichts

Prüfe die Shell-Version (bash ≥ 3.2, zsh, fish, powershell 5.1+). Bei bash zusätzlich sicherstellen, dass kein anderes Tool mit `complete -F` amoxides Registrierung überschreibt. Das Paket `bash-completion` ist nicht nötig.
