# Update Check <VersionBadge v="0.9.0" />

When a newer version of amoxide lands on crates.io, `am ls` tells you the next time you list. The nudge goes to stderr, the check runs in the background, and you can turn it off.

## What you see

If you're behind, the listing commands (`am ls`, `am la`, `am profile list`) print one line on stderr after the listing:

```text
am: 💡 a new version is available: v0.9.0 -> visit https://github.com/sassman/amoxide-rs/releases
```

If you're up to date, you see nothing.

## How it works

- First time you run a listing command, amoxide spawns a detached child that hits crates.io. The listing prints right away — you don't wait for the network.
- The result lands in `~/.cache/amoxide/update-check.toml`.
- Next time you list, amoxide reads that file. If it's older than 24 hours, a fresh background check kicks off and the previous result (if there is one) is shown in the meantime.
- Network errors are silent. Offline, DNS broken, crates.io down — the listing still works, the nudge just doesn't appear.

The nudge is on stderr, so pipes are unaffected:

```bash
am ls | grep my-alias    # nudge stays out of the pipe
```

## Disabling the check

Two ways: a config flag (persistent) or an env var (one-shot, for CI).

### Config: `~/.config/amoxide/config.toml`

```toml
[update]
check = false
```

With `check = false`, no cache read, no spawn, no network call. The cache file is never created.

### Environment: `AM_NO_UPDATE_CHECK`

```bash
AM_NO_UPDATE_CHECK=1 am ls
```

Any non-empty value skips the check for that one call. Handy in CI where you don't want network traffic and don't want to maintain a config.

## Privacy

One HTTPS GET to `https://crates.io/api/v1/crates/amoxide`. That's the whole call — crate name plus ureq's default User-Agent. Nothing identifies you, nothing tracks usage.

## See also

- [Config Files](/advanced/config-files) — full TOML reference
