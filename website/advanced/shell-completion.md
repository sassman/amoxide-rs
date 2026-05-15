# Shell Tab Completion <VersionBadge v="0.10.0" />

amoxide completes profile names, alias names, subcommand-alias segments, and variable names at the tab key — anywhere you'd type them.

## What completes where

| You type | Tab gives you |
|---|---|
| `am use <TAB>` | active and inactive profile names |
| `am profile use <TAB>`, `am profile remove <TAB>` | profile names |
| `am remove <TAB>` | alias names from the current context (global + active profiles + project) |
| `am remove -p rust <TAB>` | aliases from the `rust` profile only |
| `am remove -g <TAB>` | global aliases only |
| `am remove -l <TAB>` | project (local) aliases only |
| `am remove jj --sub <TAB>` | the next segment of a subcommand-alias chain (e.g. `b`, `ab`) |
| `am var get <TAB>`, `am var unset <TAB>` | variable names from the current context, scoped by `-p` / `-l` / `-g` |
| `am export -p <TAB>`, `am import -p <TAB>`, `am share -p <TAB>` | profile names |

## How it works

`am init <shell>` includes a one-line registration that wires amoxide into your shell's completion system. When you hit tab, the shell calls back into `am` to ask what's valid in the current spot. Because the lookup runs against the live config, completions stay accurate without any cache to invalidate.

Nothing extra to install or source — if you've already run `am setup`, completion turns on the next time you open a new shell.

## Supported shells

- **Bash** (3.2+)
- **Zsh**
- **Fish**
- **PowerShell** (5.1 + 7)
- **Brush** — rides the bash shim

Nushell is not yet supported — upstream tracking is in [`clap-rs/clap#5841`](https://github.com/clap-rs/clap/pull/5841).

## Troubleshooting

**Completion isn't working after upgrade.** Open a new shell, or re-run `eval "$(am init <shell>)"` in your current session. The `am init` output is what carries the completion registration.

**`am init` shows the registration line but completion still does nothing.** Confirm your shell has a recent enough version (bash ≥ 3.2, zsh, fish, powershell 5.1+). For bash, also check that the `bash-completion` package is installed if you rely on it elsewhere — amoxide's registration doesn't need it, but a broken `complete -F` setup can shadow it.
