# Parameterized Aliases

Aliases can use template arguments to compose powerful, reusable commands.

## Template Syntax

| Template | Description |
|----------|-------------|
| `{{1}}`, `{{2}}`, ... | Positional arguments |
| `{{@}}` | All remaining arguments |

## Examples

### Forward all arguments

```sh
am add -p git cm "git commit -S --signoff -m {{@}}"

cm my commit message
# → git commit -S --signoff -m my commit message
```

### Compose aliases together

```sh
am add -p git cm "git commit -S --signoff -m {{@}}"
am add -p git-conventional cmf "cm feat: {{@}}"

cmf my feature description
# → cm feat: my feature description
# → git commit -S --signoff -m feat: my feature description
```

### Positional arguments

```sh
am add greet "echo Hello {{1}}, welcome to {{2}}"

greet Alice Wonderland
# → echo Hello Alice, welcome to Wonderland
```

## Raw Mode

If your command literally contains `{{N}}` (e.g., in awk patterns), use `--raw` to disable template detection:

```sh
am add --raw my-awk "awk '{print {{1}}}'"
```
