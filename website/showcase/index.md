<script setup>
import { data } from './community.data'
</script>

# Community Showcase

Browse alias profiles shared by the community. Find something useful, inspect the aliases, and import with one command.

When you run the import command, `am` shows a summary of all aliases before anything is applied — review it carefully before confirming.

<CommunityGallery :profiles="data" />

::: tip Want to share your own profiles?
Check the [contribution guide](./contribute) to learn how to submit yours.
:::
