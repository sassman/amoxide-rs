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
active aliases, including active profiles and trusted project aliases (`.aliases` files).

For example: you have a project alias like `t → cargo test --all-features --verbose`
Ask claude to "run the tests". Claude Code sees `t` in the context, expands it to `cargo test --all-features --verbose`, and runs that. So it knows the flavor of `cargo test` you prefer in this very project or even from a active profile, and doesn't have to guess which flags you prefer.

Subcommand aliases work too. For example given a git profile like this:
```
├─● git (active: 2)
│   ├─ tag → git tag {{1}} && git push o {{1}}
│   ╰─◆ git (subcommands)
│     ├─ cm → commit -S --signoff -m
│     ├─ pl → pull --rebase
│     ├─ psh → push
│     ╰─ st → status --short
```

When you ask Claude Code to "pull the latest changes", it sees `pl` in the context, and expands it to `git pull --rebase` before running. Same for "commit the changes", it will use the expanded `git cm` alias. So you get the same experience as if you were typing in your terminal, but with the agent understanding your shorthand.

## Verify

In a fresh session, ask: **"what aliases do I have?"**

The agent should list them straight from the snapshot, without running a
command. 

Or ask it to "what would you run to test the code?" and it should respond with the expansion of your `t` alias.

If it doesn't, the hook didn't fire. Check
`~/.claude/settings.json`.

## Notes

- `am context` output is markdown written for a model to read. The
  shape may change as models improve, so don't script against it.
- `am context --verbose` adds the full shadow chain and any
  invalid-alias diagnostics.
