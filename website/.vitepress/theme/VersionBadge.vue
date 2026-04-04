<script setup lang="ts">
defineProps<{ v: string }>()
</script>

<template>
  <span class="version-badge">
    <span class="version-badge-label">since</span>
    <span class="version-badge-version">v{{ v }}</span>
  </span>
</template>

<style scoped>
.version-badge {
  display: inline-flex;
  font-size: 0.65rem;
  line-height: 1;
  border-radius: 4px;
  overflow: hidden;
  font-family: var(--vp-font-family-mono);
  font-weight: 600;
  vertical-align: middle;
  margin-left: 8px;
}

.version-badge-label {
  padding: 3px 6px;
  background: var(--vp-c-default-1, #e2e2e3);
  color: var(--vp-c-text-2);
}

.version-badge-version {
  padding: 3px 6px;
  background: var(--vp-c-brand-1);
  color: var(--vp-button-brand-text, #fff);
}
</style>
