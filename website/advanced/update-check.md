# Update Check

amoxide checks crates.io periodically for a newer release and prints a one-line nudge when one exists. The check is **non-blocking**, **cached for 24 hours**, and **fully opt-out**.

## What you see

When a newer version is available, the listing commands (`am ls`, `am la`, `am profile list`) print a single line to **stderr** after the listing:

```text
am: 💡 a new version is available: v0.9.0 -> visit https://github.com/sassman/amoxide-rs/releases
```

When you are already on the latest version, nothing is printed.

## How it works

- The first time you run a listing command, amoxide spawns a detached background process that calls crates.io. The current `am ls` output prints immediately — you wait for nothing.
- The result is written to a local cache file under `~/.cache/amoxide/update-check.toml`.
- Subsequent listing calls read that cache. If it's older than 24 hours, a fresh background check is kicked off and the cached result (if any) is shown in the meantime.
- All errors (offline, DNS failure, timeout, malformed response) are swallowed silently. The listing command never fails because of the update check.

Because the nudge goes to stderr, scripts that pipe `am ls` continue to work unchanged:

```bash
am ls | grep my-alias    # unaffected by the nudge
```

## Disabling the check

You can disable the update check via **config** (once-and-done) or **environment variable** (per-invocation, ideal for CI).

### Config: `~/.config/amoxide/config.toml`

```toml
[update]
check = false
```

When `check = false`, neither the cache read nor the background spawn happens. No network call is ever made, and no cache file is created.

### Environment: `AM_NO_UPDATE_CHECK`

```bash
AM_NO_UPDATE_CHECK=1 am ls
```

Set this to any non-empty value to skip the check for one invocation. Useful for CI environments where you neither want a network call nor a config edit.

## Privacy

The check sends one HTTPS request to `https://crates.io/api/v1/crates/amoxide`. The only data leaving your machine is the crate name and the User-Agent that ureq sets. No telemetry, no identifiers, no version of your local binary is reported.

## See also

- [Config Files](/advanced/config-files) — full TOML reference
