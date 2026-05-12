# AI Assistants

## The problem

Your aliases live in your interactive shell. AI coding assistants — Claude
Code, Codex, Cursor, GitHub Copilot CLI, Gemini CLI — run commands in
non-interactive subshells that **don't source your shell init**. From the
assistant's point of view, your aliases don't exist.

The result is a quiet, daily friction:

- You type `git cm "fix bug"` in chat. The assistant runs it verbatim. It
  fails with `git: 'cm' is not a git command`.
- You ask "run the tests". The assistant guesses `cargo test` instead of
  your project's `t` (which is `cargo test --all-features --workspace`).
- You watch the assistant re-type the long form of commands you've been
  abbreviating for years.

You built your aliases to think and type less. None of that vocabulary
should evaporate the moment an AI joins the terminal.

## Who this is for

**You're using a terminal-native AI assistant alongside your shell.**
The friction shows up whether you're pair-coding with Claude Code in one
pane, asking Cursor to run a build, or letting a long-running agent work
autonomously while you watch.

**You've invested in amoxide aliases.** Profiles, project `.aliases`
files, subcommand aliases — your typing vocabulary is dense. The more
shorthand you've built, the more value `am context` returns.

**You want zero ongoing work.** This isn't a tool you call per command.
Set it up once, forget it. The assistant gets a fresh snapshot every
time a session starts.

## What `am context` does

It prints a compact markdown snapshot of your **currently effective**
alias set — the same set your shell sees right now, after precedence
between global, profiles, and the project's `.aliases` file is
resolved. You wire that output into your assistant's session-start
hook. From then on, the assistant has your alias map in context and
can expand short forms before running them.

The snapshot teaches the model how to use itself — four numbered usage
rules at the top tell the assistant when to expand a name, what to
watch out for (subcommand aliases that look like real subcommands but
aren't), and how to recover from `command not found` failures.

## What you'll see

A real snapshot looks like this:

````markdown
# amoxide aliases (active set, cwd: /Users/you/projects/your-app)
#
# ## How to use this snapshot
#
# When the user mentions a name from the `Aliases` table below in any context —
# running a command, suggesting one, asking what it does — treat the `expands to`
# value as the canonical form.
#
# 1. Recognise aliases by name match. If the user's input contains a token that
#    matches a `name` from the table — including multi-word names with a space,
#    like `git pl` — it is an alias. Expand it before running.
#
# 2. Subcommand aliases are deceptive. A name like `git pl` looks like a real
#    git subcommand but is not. Running `git pl` verbatim in a subshell fails
#    with `git: 'pl' is not a git command`. Always run the value from
#    `expands to` (`git pull --rebase`), never the alias text.
#
# 3. Recover from `command not found` failures. If a shell command fails because
#    the name is unknown, check this table — the user's shell sees the alias
#    but your subshell does not.
#
# 4. In chat, the user's vocabulary is fine. When suggesting commands in
#    conversation, the short form (`git cm "msg"`) matches the user's mental
#    model. When *running* it in a subshell, use the canonical form.
#
# Precedence (highest first): project > profile(rust, prio 1) > profile(git, prio 2) > global
#
# Templates: {{N}} is a positional placeholder (1-indexed).
# Variables: {{name}} tokens are already substituted in the table below.

## Aliases

| name    | expands to                                 | from         |
|---------|--------------------------------------------|--------------|
| f       | cargo fmt                                  | project      |
| git pl  | git pull --rebase                          | profile:git  |
| gm      | git commit -S --signoff -m                 | profile:git  |
| t       | cargo test --all-features                  | project      |
| tag     | git tag {{1}} && git push o {{1}}          | profile:git  |
| ll      | ls -lha                                    | global       |
````

A few details to notice:

- **`from` column** tells the assistant where each alias came from, so it
  can answer "why does `f` do that?" without a second round trip.
- **Subcommand aliases are flattened** (`git pl → git pull --rebase`),
  not nested. The assistant sees them as ordinary entries.
- **Precedence is already applied**: if `f` is defined in both a profile
  and your project's `.aliases`, only the winner is in the table.
- **Templates are preserved**: `{{1}}` stays literal because it's a
  positional placeholder, filled in by the user's actual arguments.

## Set it up

### Option 1 — automatic

```sh
am context --setup claude
```

Idempotent. Creates `~/.claude/settings.json` if absent, or merges
into it if present without touching other keys or hook events.
Re-running detects an existing entry and does nothing.

That's the entire setup. Start a new Claude Code session and your
assistant has the snapshot.

### Option 2 — manual

If you prefer to see exactly what changes, add this to
`~/.claude/settings.json` yourself:

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

The `"startup|clear|compact"` matcher is important. Without it, the
snapshot is only injected on cold start — the assistant loses your
alias map the first time you `/clear` or `/compact` mid-session.

### Verifying

Open a new Claude Code session in your project directory and ask:
"What aliases do I have?" If wire-up worked, the assistant lists them
straight from the snapshot without running any command.

## Other assistants

`am context` is generic — its stdout works as session-start context
for any assistant whose harness supports running a command at session
start. Until native `--setup <assistant>` support lands for each,
check their docs for the equivalent of Claude Code's hook config:

- **Codex CLI / Codex App**
- **Cursor**
- **GitHub Copilot CLI**
- **Gemini CLI**

The body of the hook is always the same: run `am context` and let its
stdout become session context.

## Output stability

The markdown shape of `am context` is **not** a stable API. The format
may evolve for model-comprehension reasons without a deprecation
window. Scripting against this output is discouraged — for
machine-readable formats, [open an
issue](https://github.com/sassman/amoxide-rs/issues).

## See also

- `am context --verbose` shows the full shadow chain and any
  invalid-alias diagnostics — useful when you want to ask the assistant
  "why is `f` doing the project version?"
- [Project Aliases](/usage/project-aliases) — `.aliases` files give you
  per-repo aliases that the assistant picks up automatically.
- [Variables](/usage/variables) — `{{name}}` substitutions are baked
  into the snapshot, so the assistant sees the resolved command.
