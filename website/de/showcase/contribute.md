# Profile teilen

Hast du eine Profil-Sammlung, die andere nützlich finden könnten? So fügst du sie dem Showcase hinzu.

## Voraussetzungen

- [amoxide](https://github.com/sassman/amoxide-rs) installiert
- Ein [GitHub](https://github.com)-Konto

## Schritt für Schritt

Angenommen, dein GitHub-Nutzername ist **john** und du möchtest deine Git-Profile für konventionelle Commits teilen.

### 1. Repository forken

Gehe zu [github.com/sassman/amoxide-rs](https://github.com/sassman/amoxide-rs) und klicke auf **Fork** (oben rechts). Dadurch wird deine eigene Kopie unter `github.com/john/amoxide-rs` erstellt.

### 2. Fork klonen

```bash
git clone git@github.com:john/amoxide-rs.git
cd amoxide-rs
```

### 3. Branch erstellen

```bash
git checkout -b community/john-git-conventional
```

### 4. Vorlage kopieren

```bash
cp -r community/TEMPLATE community/john-git-conventional
```

Das ergibt:

```
community/john-git-conventional/
├── README.md     ← bearbeite diese Datei
└── profiles.toml ← ersetze mit deinem Export
```

### 5. Profile exportieren

Ersetze die Vorlagen-`profiles.toml` mit deinem eigentlichen Export:

```bash
am export -p git-conventional > community/john-git-conventional/profiles.toml
```

Oder exportiere mehrere Profile:

```bash
am export -p git -p git-conventional > community/john-git-conventional/profiles.toml
```

### 6. README bearbeiten

Öffne `community/john-git-conventional/README.md` und fülle das Frontmatter aus:

```yaml
---
author: john
description: Git-Aliase für konventionelle Commit-Workflows
category: git
tags: [git, conventional-commits, workflow]
profiles: [git, git-conventional]
---
```

Schreibe dann ein paar Sätze darüber, was deine Aliase tun, wie du sie verwendest und welche Tools installiert sein müssen.

::: details Frontmatter-Referenz
| Feld | Pflichtfeld | Beschreibung |
|------|-------------|--------------|
| `author` | ja | Dein GitHub-Nutzername |
| `description` | ja | Einzeilige Zusammenfassung (wird auf der Kachel angezeigt) |
| `category` | ja | Eines von: `git`, `docker`, `rust`, `k8s`, `python`, `node`, `misc` |
| `tags` | ja | Array von Stichwörtern für die Filterung |
| `profiles` | ja | Profilnamen in deiner `profiles.toml` |
| `shell` | nein | Nur setzen, wenn deine Aliase shell-spezifische Syntax verwenden (z. B. `fish`) |
:::

### 7. Testen

Stelle sicher, dass der Import funktioniert:

```bash
cat community/john-git-conventional/profiles.toml | am import --yes
```

### 8. Committen und pushen

```bash
git add community/john-git-conventional/
git commit -m "community: add john-git-conventional"
git push origin community/john-git-conventional
```

### 9. Pull Request öffnen

Gehe zu deinem Fork auf GitHub — du siehst ein Banner zum Erstellen eines Pull Requests. Klicke darauf und wähle die **Community Profile** PR-Vorlage.

Die Checkliste führt dich durch das Notwendige:

- [ ] Ordner mit dem Namen `community/john-git-conventional/`
- [ ] `profiles.toml` ist eine gültige `am export`-Ausgabe
- [ ] `README.md` hat das erforderliche Frontmatter
- [ ] Nur Dateien in deinem eigenen Ordner wurden geändert
- [ ] Import lokal getestet

Dein Beitrag erscheint nach der Überprüfung im Showcase.

## Regeln

- Füge nur Dateien in deinem eigenen Ordner hinzu oder ändere sie
- Ein Ordner pro Alias-Sammlung (mehrere Profile in einer `profiles.toml` sind in Ordnung)
- Für eine zweite Sammlung erstelle einen zweiten Ordner (z. B. `john-docker-compose`)

## Was macht einen guten Beitrag aus?

- **Nützlich für andere** — Aliase, die häufige Workflows lösen
- **Gut dokumentiert** — erkläre, was jeder Alias tut
- **In sich geschlossen** — weise auf Abhängigkeiten hin
- **Getestet** — überprüfe, dass der Import funktioniert

::: warning Sicherheit
Alle Einsendungen werden vor dem Zusammenführen geprüft. Wir prüfen auf verdächtige Inhalte, aber du solltest Aliase immer selbst inspizieren, bevor du sie importierst — auch aus diesem Showcase.
:::
