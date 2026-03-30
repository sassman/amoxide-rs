<div v-pre>

# Parametrisierte Aliase

Aliase können Template-Argumente verwenden, um leistungsstarke, wiederverwendbare Befehle zu erstellen.

## Template-Syntax

| Template | Beschreibung |
|----------|-------------|
| `{{1}}`, `{{2}}`, ... | Positionsargumente |
| `{{@}}` | Alle restlichen Argumente |

## Beispiele

### Alle Argumente weiterleiten

```sh
am add -p git cm "git commit -S --signoff -m {{@}}"

cm meine commit nachricht
# → git commit -S --signoff -m meine commit nachricht
```

### Aliase verketten

```sh
am add -p git cm "git commit -S --signoff -m {{@}}"
am add -p git-conventional cmf "cm feat: {{@}}"

cmf mein neues Feature
# → cm feat: mein neues Feature
# → git commit -S --signoff -m feat: mein neues Feature
```

### Positionsargumente

```sh
am add greet "echo Hallo {{1}}, willkommen in {{2}}"

greet Alice Wunderland
# → echo Hallo Alice, willkommen in Wunderland
```

## Raw-Modus

Wenn dein Befehl tatsächlich `{{N}}` enthält (z.B. in awk-Mustern), verwende `--raw` um die Template-Erkennung zu deaktivieren:

```sh
am add --raw my-awk "awk '{print {{1}}}'"
```

</div>
