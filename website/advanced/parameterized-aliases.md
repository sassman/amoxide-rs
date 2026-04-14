<div v-pre>

# Parameterized Aliases

By default, any arguments you pass to an alias are appended at the end — just like regular shell aliases. Parameterized aliases let you place arguments **anywhere** in the command using templates.

## Template Syntax

| Template | Description |
|----------|-------------|
| `{{1}}`, `{{2}}`, ... | Insert a specific positional argument |
| `{{@}}` | Insert all arguments at a specific position |

## When You Don't Need Templates

Trailing arguments work automatically — no template needed:

```sh
am add -p git cm "git commit -S --signoff -m"

cm my commit message
# → git commit -S --signoff -m my commit message
```

## When Templates Shine

### Arguments in the middle of a command

```sh
am add deploy "rsync -avz {{@}} user@server:/var/www/"

deploy ./dist/ --exclude=node_modules
# → rsync -avz ./dist/ --exclude=node_modules user@server:/var/www/
```

Without `{{@}}`, the destination would end up at the wrong position.

### Positional arguments

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

## Raw Mode

If your command literally contains `{{N}}` (e.g., in awk patterns), use `--raw` to disable template detection:

```sh
am add --raw my-awk "awk '{print {{1}}}'"
```

## Fish Shell Note

Parameterized aliases are always emitted as `function` definitions in fish, even when [`use_abbr = true`](/advanced/config-files#fish-shellfishfish) is set in your config. Fish abbreviations do not support arguments, so the function form is the only way to place them correctly.

</div>
