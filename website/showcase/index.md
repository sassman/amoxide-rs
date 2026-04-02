---
layout: page
---

<script setup>
import { data } from './community.data'
</script>

# Community Showcase

Browse alias profiles shared by the community. Find something useful, inspect the aliases, and import with one command.

<CommunityGallery :profiles="data" />

## Share yours

Got a profile collection others might find useful?

1. Export your profiles: `am export -p <name> > profiles.toml`
2. Create a folder: `community/<your-github-handle>-<descriptive-name>/`
3. Add your `profiles.toml` and a `README.md` (copy from [`TEMPLATE.md`](https://github.com/sassman/amoxide-rs/blob/main/community/TEMPLATE.md))
4. Open a PR — use the **Community Profile** template

Your aliases will appear here after review. Before importing anyone's profile, **always inspect the aliases first** — expand "View aliases" to see exactly what you're getting.

::: warning Security
When importing aliases from others, `am` scans for suspicious content (hidden escape sequences, control characters) and will warn you. Never blindly trust external input — review the aliases before confirming the import.
:::
