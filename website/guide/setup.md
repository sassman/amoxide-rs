# Shell Setup

## Guided Setup (Recommended)

The easiest way to set up amoxide:

::: code-group

```sh [Fish]
am setup fish
```

```sh [Zsh]
am setup zsh
```

```powershell [PowerShell]
am setup powershell
```

```sh [Bash (v0.4.0+)]
am setup bash
```

```sh [Brush]
am setup brush
```

:::

This detects your profile file, shows exactly what it will add, and asks for confirmation.

## Manual Setup

Add the init line to your shell configuration:

::: code-group

```fish [Fish]
# ~/.config/fish/config.fish
am init fish | source
```

```zsh [Zsh]
# ~/.zshrc
eval "$(am init zsh)"
```

```powershell [PowerShell]
# Add to your PowerShell profile (echo $PROFILE to find it)
(am init powershell) -join "`n" | Invoke-Expression
```

```bash [Bash (v0.4.0+)]
# ~/.bashrc
eval "$(am init bash)"
```

```bash [Brush]
# ~/.brushrc
eval "$(am init brush)"
```

:::

::: tip Bash / Brush load order
If you use starship, oh-my-bash, or bash-it, add the `am init` line **after** their initialization. This ensures amoxide's cd hook isn't overwritten.
:::

## What the Init Does

The `am init` command does two things:

1. **Loads aliases** from your active profiles into the current shell
2. **Installs a cd hook** that automatically loads/unloads project aliases (from `.aliases` files) when you change directories

## Verify Setup

Check that everything is configured correctly:

```sh
am status
```

This verifies that the shell hook is installed and your profiles are loaded.
