## Commit conventions

All commit subjects must follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```
<type>[(scope)][!]: <subject>
```

Allowed types: `build chore ci docs feat fix perf refactor revert style test`.
Trailing `!` marks a breaking change.

A local `commit-msg` hook enforces this. Enable it once per clone:

```bash
git config core.hooksPath .githooks
```

PR bodies should follow the [pull request template](.github/pull_request_template.md) — they
become the squash-merge commit body and are rendered as release-notes paragraphs.
