# AI Assistants

Amoxide aliases live in your interactive shell. AI coding assistants
(Claude Code, Codex, Cursor, …) execute commands in non-interactive
subshells that do not see those aliases. `am context` bridges the gap:
it prints a compact, model-friendly snapshot of your active alias set,
to be injected into the assistant's session via a session-start hook.

After wire-up, your assistant can expand short forms like `git cm`,
`gst`, or your subcommand aliases (`git pl`) into the canonical
commands before running them.

## Claude Code

Two equally valid options — pick based on preference.

### Option 1 — automatic setup

```sh
am context --setup claude
```

Idempotent. Detects an existing entry and no-ops. Creates
`~/.claude/settings.json` if absent; merges into it if present without
touching other keys or hook events.

### Option 2 — manual setup

Add to `~/.claude/settings.json`:

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

The `"startup|clear|compact"` matcher ensures the snapshot is also
re-injected after `/clear` and `/compact` — without it, the assistant
loses your alias map mid-session.

## Other assistants

`am context` is generic — its stdout works as session-start context
for any assistant whose harness supports running a command at session
start. Codex CLI, Cursor, GitHub Copilot CLI all have similar
mechanisms; consult their docs for the equivalent of Claude Code's
hook config. Native `--setup <assistant>` support for these is
planned.

## A note on output stability

The markdown shape of `am context` is **not** a stable API. The format
may evolve for model-comprehension reasons without a deprecation
window. Scripting against this output is discouraged — for
machine-readable formats, [open an
issue](https://github.com/sassman/amoxide-rs/issues).
