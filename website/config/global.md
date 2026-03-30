# Global Aliases

Global aliases are always active — available in every shell session, regardless of which profiles are enabled or which directory you're in.

## Adding Global Aliases

```sh
am add gs git status
am add ll "ls -lha"
```

When you run `am add` without `-p` (profile) or `-l` (local), the alias is added globally.

::: tip
Short form: `am a gs git status`
:::

## Removing Global Aliases

```sh
am remove gs
am r gs              # short form
```

## When to Use Global Aliases

Global aliases are best for commands you use everywhere, regardless of project context:

- Git shortcuts (`gs`, `gp`, `gl`)
- System utilities (`ll`, `la`)
- Editor shortcuts

For aliases that only make sense in certain contexts, use [Profiles](/config/profiles) (e.g., Rust toolchain aliases) or [Project Aliases](/config/project-aliases) (e.g., project-specific build commands).

## How They Work

Global aliases are stored in `~/.config/amoxide/config.toml` and loaded into every shell session via `am init`. They sit at the base of the alias hierarchy — profiles and project aliases can override them if they define an alias with the same name.
