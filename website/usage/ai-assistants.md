# AI Assistants

Your aliases live in your interactive shell. AI coding assistants — Claude
Code, Codex, Cursor — run commands in non-interactive subshells that don't
source your shell init. So if you've defined `ct = cargo test` in a `rust`
profile and ask the assistant to "run the tests", it tries `ct` and gets
`command not found`.

`am context` prints your active alias set as markdown that the assistant
can ingest at session start. Once wired up, the assistant expands `ct`
into `cargo test` before running it.

## Install

```sh
am context --setup claude
```

Idempotent. Creates `~/.claude/settings.json` if absent, or merges into
it without touching other keys. Re-run safely.

For other assistants, run `am context` from their session-start hook
manually — see the assistant's hook docs.

## What to expect in a Claude Code session

Open a new Claude Code session in your project directory. The assistant
now has your active aliases — `ll`, `gs`, `ct`, anything in active
profiles, anything from a trusted `.aliases` file in scope.

Try: ask "run the tests". The assistant runs `cargo test` (the canonical
form), not `ct`. Same for `git pl` → `git pull --rebase`,
`gst` → `git status`, etc.

Subcommand aliases work too. The assistant knows `git pl` looks like a
subcommand but isn't, and runs the expansion.

## Verify

In a fresh session, ask: **"what aliases do I have?"**

The assistant should list them straight from the snapshot, no command
run. If it doesn't, the hook didn't fire — check
`~/.claude/settings.json`.

## Manual setup

If you'd rather edit the JSON yourself:

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|clear|compact",
        "hooks": [
          { "type": "command", "command": "am context", "async": false }
        ]
      }
    ]
  }
}
```

The `"startup|clear|compact"` matcher matters — without it the snapshot
only injects on cold start, and the assistant loses your aliases the
first time you `/clear` or `/compact`.

## Notes

- The markdown shape of `am context` may evolve for model-comprehension
  reasons. Don't script against it.
- `am context --verbose` adds the full shadow chain and any
  invalid-alias diagnostics.
