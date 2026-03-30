<div v-pre>

# Parametrisierte Aliase

Standardmäßig werden alle Argumente, die du an einen Alias übergibst, am Ende angehängt — wie bei normalen Shell-Aliasen. Parametrisierte Aliase ermöglichen es, Argumente **überall** im Befehl zu platzieren.

## Template-Syntax

| Template | Beschreibung |
|----------|-------------|
| `{{1}}`, `{{2}}`, ... | Ein bestimmtes Positionsargument einfügen |
| `{{@}}` | Alle Argumente an einer bestimmten Stelle einfügen |

## Wann du keine Templates brauchst

Nachfolgende Argumente funktionieren automatisch — kein Template nötig:

```sh
am add -p git cm "git commit -S --signoff -m"

cm meine commit nachricht
# → git commit -S --signoff -m meine commit nachricht
```

## Wann Templates glänzen

### Argumente in der Mitte eines Befehls

```sh
am add deploy "rsync -avz {{@}} user@server:/var/www/"

deploy ./dist/ --exclude=node_modules
# → rsync -avz ./dist/ --exclude=node_modules user@server:/var/www/
```

Ohne `{{@}}` würde das Ziel an der falschen Position landen.

### Positionsargumente

```sh
am add gri "git rebase -i HEAD~{{1}}"

gri 3
# → git rebase -i HEAD~3
```

```sh
am add gcf "git commit --fixup={{1}}"

gcf abc123
# → git commit --fixup=abc123
```

## Raw-Modus

Wenn dein Befehl tatsächlich `{{N}}` enthält (z.B. in awk-Mustern), verwende `--raw` um die Template-Erkennung zu deaktivieren:

```sh
am add --raw my-awk "awk '{print {{1}}}'"
```

</div>
