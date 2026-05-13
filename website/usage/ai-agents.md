# AI Agents

Your aliases live in your interactive shell. AI coding agents like Claude
Code, Codex, and Cursor run commands in non-interactive subshells that
don't load your shell init. So if you've defined `ct = cargo test` in a
`rust` profile and ask the agent to "run the tests", it tries `ct` and
gets `command not found`.

`am context` prints your active alias set as markdown that the agent can
read at session start. Once wired up, the agent expands `ct` to `cargo
test` before running.

## Setup

```sh
am context --setup claude
```

This creates `~/.claude/settings.json` if it's missing, or merges into it
without touching any other keys. Idempotent: re-running it does nothing.

For other agents, run `am context` from their session-start hook
yourself. Check the agent's hook docs.

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

The `"startup|clear|compact"` matcher is what re-injects the snapshot
after `/clear` and `/compact`. Drop it and the agent loses your aliases
the first time you clear the conversation.

Verified with Claude Code 2.1.126. See [Anthropic's hooks
docs](https://code.claude.com/docs/en/hooks) for the full reference of
configurability.

## What to expect in a Claude Code session

Open a new session in your project directory. The agent now sees your
active aliases: `ll`, `gs`, `ct`, anything in active profiles, anything
from a trusted `.aliases` file.

Ask it to "run the tests". It runs `cargo test`, not `ct`. Same for
`git pl` → `git pull --rebase`, `gst` → `git status`.

Subcommand aliases work too. The agent knows `git pl` looks like a real
git subcommand but isn't, and runs the expansion instead.

## Verify

In a fresh session, ask: **"what aliases do I have?"**

The agent should list them straight from the snapshot, without running a
command. If it doesn't, the hook didn't fire. Check
`~/.claude/settings.json`.

## Notes

- `am context` output is markdown written for a model to read. The
  shape may change as models improve, so don't script against it.
- `am context --verbose` adds the full shadow chain and any
  invalid-alias diagnostics.
