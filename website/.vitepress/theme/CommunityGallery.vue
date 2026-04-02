<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import type { CommunityProfile } from '../../showcase/community.data'

const props = defineProps<{
  profiles: CommunityProfile[]
}>()

const copiedSlug = ref<string | null>(null)
const expandedImport = ref<string | null>(null)

function parseHash(): Record<string, string> {
  if (typeof window === 'undefined') return {}
  const hash = window.location.hash.slice(1) // remove #
  if (!hash) return {}
  const [key, ...rest] = hash.split('=')
  if (key && rest.length) return { [key]: rest.join('=') }
  return {}
}

const filter = ref<Record<string, string>>(parseHash())

onMounted(() => {
  filter.value = parseHash()
  window.addEventListener('hashchange', () => {
    filter.value = parseHash()
  })
})

const filtered = computed(() => {
  const f = filter.value
  if (f.tag) return props.profiles.filter(p => p.tags.includes(f.tag))
  if (f.author) return props.profiles.filter(p => p.author === f.author)
  if (f.name) return props.profiles.filter(p => p.slug === f.name)
  return props.profiles
})

const activeLabel = computed(() => {
  const f = filter.value
  if (f.tag) return `Tag: ${f.tag}`
  if (f.author) return `Author: ${f.author}`
  if (f.name) return f.name.replace(/^[^-]+-/, '')
  return null
})

function importCommand(profile: CommunityProfile): string {
  return `am import ${profile.importUrl}`
}

function originUrl(profile: CommunityProfile): string {
  return `https://github.com/sassman/amoxide-rs/tree/main/community/${profile.slug}`
}

function toggleImport(slug: string) {
  expandedImport.value = expandedImport.value === slug ? null : slug
}

async function copyImport(profile: CommunityProfile) {
  await navigator.clipboard.writeText(importCommand(profile))
  copiedSlug.value = profile.slug
  setTimeout(() => { copiedSlug.value = null }, 2000)
}
</script>

<template>
  <div class="gallery">
    <div v-if="activeLabel" class="filter-active">
      Showing: <strong>{{ activeLabel }}</strong>
      <a href="/showcase/#" class="filter-clear" @click.prevent="filter = {}; history.replaceState(null, '', '/showcase/')">clear</a>
    </div>

    <div class="tiles">
      <div v-for="profile in filtered" :key="profile.slug" class="tile">
        <div class="tile-header">
          <h3>{{ profile.description }}</h3>
          <div class="tile-byline">
            <span class="tile-author">by {{ profile.author }}</span>
          </div>
        </div>
        <div class="tile-meta">
          <span v-if="profile.shell" class="tile-tag">{{ profile.shell }} only</span>
          <a
            v-for="tag in profile.tags"
            :key="tag"
            :href="`/showcase/#tag=${tag}`"
            :class="['tile-tag', { 'tile-tag-active': filter.tag === tag }]"
            @click.prevent="filter = { tag }; history.replaceState(null, '', `/showcase/#tag=${tag}`)"
          >{{ tag }}</a>
        </div>
        <!-- Collapsed TOML preview -->
        <details class="tile-toml">
          <summary>View aliases ({{ profile.profiles.length }} {{ profile.profiles.length === 1 ? 'profile' : 'profiles' }}, {{ profile.aliasCount }} {{ profile.aliasCount === 1 ? 'alias' : 'aliases' }})</summary>
          <div class="toml-block">
            <pre><code>{{ profile.toml }}</code></pre>
          </div>
        </details>

        <!-- Import actions -->
        <div class="tile-actions">
          <button class="copy-btn" @click="copyImport(profile)">
            {{ copiedSlug === profile.slug ? '✓ Copied!' : '📋 Copy import command' }}
          </button>
          <button class="expand-btn" @click="toggleImport(profile.slug)">
            {{ expandedImport === profile.slug ? '▾' : '▸' }}
          </button>
        </div>
        <div v-if="expandedImport === profile.slug" class="import-detail">
          <textarea
            readonly
            rows="4"
            :value="importCommand(profile)"
            class="import-input"
            @click="($event.target as HTMLTextAreaElement).select()"
          />
          <p class="import-hint">
            <a :href="originUrl(profile)" target="_blank">See the original source</a> before running this command.
          </p>
        </div>
      </div>
    </div>

    <div v-if="filtered.length === 0" class="empty">
      No profiles found.
    </div>
  </div>
</template>

<style scoped>
.gallery {
  margin-top: 1rem;
}

.filter-active {
  padding: 0.5rem 0.75rem;
  margin-bottom: 1rem;
  border-radius: 0.375rem;
  background: var(--vp-c-bg-soft);
  font-size: 0.875rem;
  color: var(--vp-c-text-2);
}

.filter-clear {
  margin-left: 0.5rem;
  font-size: 0.8rem;
  color: var(--vp-c-text-3);
}

.filter-clear:hover {
  color: var(--vp-c-brand-1);
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

.tile-byline {
  display: flex;
  align-items: center;
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

.tile-tag {
  font-size: 0.75rem;
  padding: 0.125rem 0.5rem;
  border-radius: 0.75rem;
  border: 1px solid var(--vp-c-brand-1);
  color: var(--vp-c-text-2);
  background: transparent;
  text-decoration: none;
  cursor: pointer;
  transition: all 0.2s;
}

.tile-tag:hover {
  box-shadow: 0 0 8px var(--vp-c-brand-1);
}

.tile-tag-active {
  box-shadow: 0 0 8px var(--vp-c-brand-1);
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
  display: flex;
  gap: 0;
  margin-top: 0.75rem;
}

.copy-btn {
  flex: 1;
  padding: 0.5rem;
  border: 1px solid var(--vp-c-divider);
  border-radius: 0.375rem 0 0 0.375rem;
  background: var(--vp-c-bg-alt);
  color: var(--vp-c-text-1);
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.15s;
}

.copy-btn:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.expand-btn {
  padding: 0.5rem 0.625rem;
  border: 1px solid var(--vp-c-divider);
  border-left: none;
  border-radius: 0 0.375rem 0.375rem 0;
  background: var(--vp-c-bg-alt);
  color: var(--vp-c-text-2);
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.15s;
}

.expand-btn:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.import-detail {
  margin-top: 0.5rem;
}

.import-input {
  width: 100%;
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--vp-c-divider);
  border-radius: 0.375rem;
  background: var(--vp-c-bg-alt);
  color: var(--vp-c-text-1);
  font-family: var(--vp-font-family-mono);
  font-size: 0.8125rem;
  line-height: 1.6;
  outline: none;
  resize: none;
  word-break: break-all;
}

.import-input:focus {
  border-color: var(--vp-c-brand-1);
}

.import-hint {
  margin: 0.375rem 0 0;
  font-size: 0.75rem;
  color: var(--vp-c-text-3);
}

.empty {
  text-align: center;
  color: var(--vp-c-text-3);
  padding: 2rem;
}
</style>
