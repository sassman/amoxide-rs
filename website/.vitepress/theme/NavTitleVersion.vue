<script setup lang="ts">
declare const __APP_VERSION__: string
const version = __APP_VERSION__
</script>

<template>
  <span class="nav-version">v{{ version }}</span>
</template>

<style scoped>
.nav-version {
  font-size: 0.7rem;
  font-family: var(--vp-font-family-mono);
  color: var(--vp-c-text-3);
  margin-left: 6px;
  position: relative;
  top: 4px;
}
</style>
