<script setup lang="ts">
import { ref, computed } from 'vue'
import type { CommunityProfile } from '../../showcase/community.data'

const props = defineProps<{
  profiles: CommunityProfile[]
}>()

const selectedCategory = ref('all')
const copiedSlug = ref<string | null>(null)

const categories = computed(() => {
  const cats = new Set(props.profiles.map(p => p.category))
  return ['all', ...Array.from(cats).sort()]
})

const filtered = computed(() => {
  if (selectedCategory.value === 'all') return props.profiles
  return props.profiles.filter(p => p.category === selectedCategory.value)
})

const grouped = computed(() => {
  const groups: Record<string, typeof props.profiles> = {}
  for (const p of filtered.value) {
    const cat = p.category
    if (!groups[cat]) groups[cat] = []
    groups[cat].push(p)
  }
  return groups
})

function importCommand(profile: CommunityProfile): string {
  return `am import ${profile.importUrl}`
}

async function copyImport(profile: CommunityProfile) {
  await navigator.clipboard.writeText(importCommand(profile))
  copiedSlug.value = profile.slug
  setTimeout(() => { copiedSlug.value = null }, 2000)
}
</script>

<template>
  <div class="gallery">
    <!-- Category filter -->
    <div class="filter-bar">
      <button
        v-for="cat in categories"
        :key="cat"
        :class="['filter-btn', { active: selectedCategory === cat }]"
        @click="selectedCategory = cat"
      >
        {{ cat }}
      </button>
    </div>

    <!-- Grouped tiles -->
    <div v-for="(profiles, category) in grouped" :key="category" class="category-group">
      <h2 class="category-title">{{ category }}</h2>
      <div class="tiles">
        <div v-for="profile in profiles" :key="profile.slug" class="tile">
          <div class="tile-header">
            <h3>{{ profile.description }}</h3>
            <span class="tile-author">by {{ profile.author }}</span>
          </div>
          <div class="tile-meta">
            <span class="tile-shell">{{ profile.shell }}</span>
            <span v-for="tag in profile.tags" :key="tag" class="tile-tag">{{ tag }}</span>
          </div>
          <div class="tile-profiles">
            <code v-for="name in profile.profiles" :key="name">{{ name }}</code>
          </div>

          <!-- Collapsed TOML preview -->
          <details class="tile-toml">
            <summary>View aliases</summary>
            <div class="toml-block">
              <pre><code>{{ profile.toml }}</code></pre>
            </div>
          </details>

          <!-- Import button -->
          <div class="tile-actions">
            <button class="copy-btn" @click="copyImport(profile)">
              {{ copiedSlug === profile.slug ? '✓ Copied!' : '📋 Copy import command' }}
            </button>
          </div>
          <div class="tile-import-cmd">
            <code>{{ importCommand(profile) }}</code>
          </div>
        </div>
      </div>
    </div>

    <div v-if="filtered.length === 0" class="empty">
      No profiles found for this category.
    </div>
  </div>
</template>

<style scoped>
.gallery {
  margin-top: 1rem;
}

.filter-bar {
  display: flex;
  gap: 0.5rem;
  flex-wrap: wrap;
  margin-bottom: 1.5rem;
}

.filter-btn {
  padding: 0.25rem 0.75rem;
  border: 1px solid var(--vp-c-divider);
  border-radius: 1rem;
  background: transparent;
  color: var(--vp-c-text-2);
  cursor: pointer;
  font-size: 0.875rem;
  transition: all 0.2s;
}

.filter-btn:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.filter-btn.active {
  background: var(--vp-c-brand-1);
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-white);
}

.category-title {
  font-size: 1.25rem;
  margin: 1.5rem 0 0.75rem;
  text-transform: capitalize;
  border-bottom: 1px solid var(--vp-c-divider);
  padding-bottom: 0.5rem;
}

.tiles {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 1rem;
}

.tile {
  border: 1px solid var(--vp-c-divider);
  border-radius: 0.5rem;
  padding: 1rem;
  background: var(--vp-c-bg-soft);
  transition: border-color 0.2s;
}

.tile:hover {
  border-color: var(--vp-c-brand-1);
}

.tile-header h3 {
  margin: 0 0 0.25rem;
  font-size: 1rem;
}

.tile-author {
  font-size: 0.8rem;
  color: var(--vp-c-text-3);
}

.tile-meta {
  display: flex;
  gap: 0.375rem;
  flex-wrap: wrap;
  margin: 0.5rem 0;
}

.tile-shell {
  font-size: 0.75rem;
  padding: 0.125rem 0.5rem;
  border-radius: 0.75rem;
  background: var(--vp-c-brand-soft);
  color: var(--vp-c-brand-1);
}

.tile-tag {
  font-size: 0.75rem;
  padding: 0.125rem 0.5rem;
  border-radius: 0.75rem;
  background: var(--vp-c-default-soft);
  color: var(--vp-c-text-2);
}

.tile-profiles {
  display: flex;
  gap: 0.375rem;
  margin: 0.5rem 0;
}

.tile-profiles code {
  font-size: 0.8rem;
  padding: 0.125rem 0.375rem;
  background: var(--vp-c-default-soft);
  border-radius: 0.25rem;
}

.tile-toml {
  margin: 0.75rem 0;
}

.tile-toml summary {
  cursor: pointer;
  font-size: 0.85rem;
  color: var(--vp-c-text-2);
}

.tile-toml summary:hover {
  color: var(--vp-c-brand-1);
}

.toml-block {
  margin-top: 0.5rem;
  max-height: 300px;
  overflow: auto;
}

.toml-block pre {
  margin: 0;
  padding: 0.75rem;
  border-radius: 0.375rem;
  background: var(--vp-c-bg-alt);
  font-size: 0.8rem;
  line-height: 1.5;
}

.tile-actions {
  margin-top: 0.75rem;
}

.copy-btn {
  width: 100%;
  padding: 0.5rem;
  border: 1px solid var(--vp-c-brand-1);
  border-radius: 0.375rem;
  background: transparent;
  color: var(--vp-c-brand-1);
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.2s;
}

.copy-btn:hover {
  background: var(--vp-c-brand-1);
  color: var(--vp-c-white);
}

.tile-import-cmd {
  margin-top: 0.375rem;
}

.tile-import-cmd code {
  font-size: 0.7rem;
  color: var(--vp-c-text-3);
  word-break: break-all;
}

.empty {
  text-align: center;
  color: var(--vp-c-text-3);
  padding: 2rem;
}
</style>
