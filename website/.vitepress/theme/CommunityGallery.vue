<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from 'vue'
import type { CommunityProfile, CommunityProfileDetail } from '../../showcase/community.data'
import { useData } from 'vitepress'

const { site } = useData()

const props = defineProps<{
  profiles: CommunityProfile[]
}>()

const copiedSlug = ref<string | null>(null)
const expandedImport = ref<string | null>(null)
const modalExpandedImport = ref<string | null>(null)
const viewSlug = ref<string | null>(null)
const detailRef = ref<HTMLElement | null>(null)

// Cache for lazy-loaded detail data
const detailCache = ref<Record<string, CommunityProfileDetail>>({})
const loadingDetail = ref<string | null>(null)

async function loadDetail(slug: string): Promise<CommunityProfileDetail | null> {
  if (detailCache.value[slug]) return detailCache.value[slug]
  loadingDetail.value = slug
  try {
    const base = site.value.base || '/'
    const res = await fetch(`${base}showcase/data/${slug}.json`)
    if (!res.ok) return null
    const data: CommunityProfileDetail = await res.json()
    detailCache.value[slug] = data
    return data
  } catch {
    return null
  } finally {
    loadingDetail.value = null
  }
}

function parseHash(): { filter: Record<string, string>; view: string | null } {
  if (typeof window === 'undefined') return { filter: {}, view: null }
  const hash = window.location.hash.slice(1)
  if (!hash) return { filter: {}, view: null }
  const params: Record<string, string> = {}
  for (const part of hash.split('&')) {
    const [key, ...rest] = part.split('=')
    if (key && rest.length) params[key] = rest.join('=')
  }
  const { view, ...filterParams } = params
  return { filter: filterParams, view: view || null }
}

function updateHash() {
  const parts: string[] = []
  const f = filter.value
  if (f.tag) parts.push(`tag=${f.tag}`)
  if (f.author) parts.push(`author=${f.author}`)
  if (f.name) parts.push(`name=${f.name}`)
  if (viewSlug.value) parts.push(`view=${viewSlug.value}`)
  const hash = parts.length ? `#${parts.join('&')}` : ''
  history.replaceState(null, '', `/showcase/${hash}`)
}

const filter = ref<Record<string, string>>({})

function applyHash() {
  const parsed = parseHash()
  filter.value = parsed.filter
  viewSlug.value = parsed.view
  // Pre-load detail if URL has a view param
  if (parsed.view) loadDetail(parsed.view)
}

onMounted(() => {
  applyHash()
  window.addEventListener('hashchange', applyHash)
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && viewSlug.value) closeDetail()
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

const viewProfile = computed(() => {
  if (!viewSlug.value) return null
  return props.profiles.find(p => p.slug === viewSlug.value) || null
})

const viewDetail = computed(() => {
  if (!viewSlug.value) return null
  return detailCache.value[viewSlug.value] || null
})

async function openDetail(slug: string) {
  viewSlug.value = slug
  updateHash()
  await loadDetail(slug)
  nextTick(() => {
    detailRef.value?.scrollIntoView({ behavior: 'smooth', block: 'start' })
  })
}

function closeDetail() {
  viewSlug.value = null
  modalExpandedImport.value = null
  updateHash()
}

// Tile inline preview: load on demand when <details> opens
const tileTomlCache = ref<Record<string, string>>({})

async function onTileToggle(slug: string, event: Event) {
  const details = event.target as HTMLDetailsElement
  if (details.open && !tileTomlCache.value[slug]) {
    const detail = await loadDetail(slug)
    if (detail) tileTomlCache.value[slug] = detail.toml
  }
}

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
      <a href="/showcase/#" class="filter-clear" @click.prevent="filter = {}; updateHash()">clear</a>
    </div>

    <div class="tiles">
      <template v-for="profile in filtered" :key="profile.slug">
      <div
        :class="['tile', { 'tile-selected': viewSlug === profile.slug }]"
      >
        <button class="tile-expand-btn" @click.stop="openDetail(profile.slug)" title="Expand">↗</button>
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
            @click.stop.prevent="filter = { tag }; updateHash()"
          >{{ tag }}</a>
        </div>
        <!-- Collapsed TOML preview (lazy loaded) -->
        <details class="tile-toml" @click.stop @toggle="onTileToggle(profile.slug, $event)">
          <summary>View aliases ({{ profile.profiles.length }} {{ profile.profiles.length === 1 ? 'profile' : 'profiles' }}, {{ profile.aliasCount }} {{ profile.aliasCount === 1 ? 'alias' : 'aliases' }})</summary>
          <div class="toml-block">
            <pre v-if="tileTomlCache[profile.slug]"><code>{{ tileTomlCache[profile.slug] }}</code></pre>
            <p v-else class="loading">Loading...</p>
          </div>
        </details>

        <!-- Import actions -->
        <div class="tile-actions" @click.stop>
          <button class="copy-btn" @click="copyImport(profile)">
            {{ copiedSlug === profile.slug ? '✓ Copied!' : '📋 Copy import command' }}
          </button>
          <button class="expand-btn" @click="toggleImport(profile.slug)">
            {{ expandedImport === profile.slug ? '▾' : '▸' }}
          </button>
        </div>
        <div v-if="expandedImport === profile.slug" class="import-detail" @click.stop>
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

      </template>
    </div>

    <div v-if="filtered.length === 0" class="empty">
      No profiles found.
    </div>

    <!-- Modal overlay -->
    <Teleport to="body">
      <div v-if="viewProfile" class="modal-overlay" @click.self="closeDetail()">
        <div class="modal-content" ref="detailRef">
          <button class="modal-close" @click="closeDetail()">✕</button>

          <div class="detail-header">
            <h2>{{ viewProfile.description }}</h2>
          </div>
          <div class="detail-byline">
            by <a :href="`https://github.com/${viewProfile.author}`" target="_blank">{{ viewProfile.author }}</a>
            <template v-if="viewProfile.profiles.length"> · provides <code v-for="(p, i) in viewProfile.profiles" :key="p"><template v-if="i > 0">, </template>{{ p }}</code></template>
            · {{ viewProfile.aliasCount }} {{ viewProfile.aliasCount === 1 ? 'alias' : 'aliases' }}
            · <a :href="originUrl(viewProfile)" target="_blank">source</a>
          </div>
          <div class="detail-meta">
            <a
              v-for="tag in viewProfile.tags"
              :key="tag"
              :href="`/showcase/#tag=${tag}`"
              :class="['tile-tag', { 'tile-tag-active': filter.tag === tag }]"
              @click.prevent="filter = { tag }; closeDetail(); updateHash()"
            >{{ tag }}</a>
          </div>

          <div class="detail-scroll">
            <div v-if="viewDetail" class="detail-body" v-html="viewDetail.readmeHtml" />
            <p v-else class="loading">Loading...</p>

            <div v-if="viewDetail" class="detail-section">
              <h3>Aliases ({{ viewProfile.profiles.length }} {{ viewProfile.profiles.length === 1 ? 'profile' : 'profiles' }}, {{ viewProfile.aliasCount }} {{ viewProfile.aliasCount === 1 ? 'alias' : 'aliases' }})</h3>
              <div class="detail-toml">
                <pre><code>{{ viewDetail.toml }}</code></pre>
              </div>
            </div>
          </div>

          <div class="detail-footer">
            <div class="tile-actions">
              <button class="copy-btn" @click="copyImport(viewProfile)">
                {{ copiedSlug === viewProfile.slug ? '✓ Copied!' : '📋 Copy import command' }}
              </button>
              <button class="expand-btn" @click="modalExpandedImport = modalExpandedImport === viewProfile.slug ? null : viewProfile.slug">
                {{ modalExpandedImport === viewProfile.slug ? '▾' : '▸' }}
              </button>
            </div>
            <div v-if="modalExpandedImport === viewProfile.slug" class="import-detail">
              <textarea
                readonly
                rows="2"
                :value="importCommand(viewProfile)"
                class="import-input"
                @click="($event.target as HTMLTextAreaElement).select()"
              />
              <p class="import-hint">
                <a :href="originUrl(viewProfile)" target="_blank">See the original source</a> before running this command.
              </p>
            </div>

            <div class="activate-hint">
              <p>Then activate the {{ viewProfile.profiles.length === 1 ? 'profile' : 'profiles' }}:</p>
              <pre><code>{{ viewProfile.profiles.map(p => `am profile use ${p}`).join('\n') }}</code></pre>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
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
  grid-template-columns: repeat(3, 1fr);
  gap: 1rem;
}

@media (max-width: 960px) {
  .tiles {
    grid-template-columns: repeat(2, 1fr);
  }
}

@media (max-width: 640px) {
  .tiles {
    grid-template-columns: 1fr;
  }
}

.tile {
  border: 1px solid var(--vp-c-divider);
  border-radius: 0.5rem;
  padding: 1rem;
  background: var(--vp-c-bg-soft);
  transition: border-color 0.2s;
  position: relative;
}

.tile:hover {
  border-color: var(--vp-c-brand-1);
}

.tile-selected {
  border-color: var(--vp-c-brand-1);
}

.tile-expand-btn {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  width: 32px;
  height: 32px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 0.375rem;
  background: var(--vp-c-bg-alt);
  color: var(--vp-c-text-3);
  cursor: pointer;
  font-size: 1rem;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: all 0.15s;
  z-index: 1;
  padding: 0;
}

.tile:hover .tile-expand-btn {
  opacity: 1;
}

.tile-expand-btn:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
  background: var(--vp-c-bg-soft);
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
  line-height: 1.25rem;
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

.loading {
  font-size: 0.85rem;
  color: var(--vp-c-text-3);
  padding: 0.5rem 0;
}

.empty {
  text-align: center;
  color: var(--vp-c-text-3);
  padding: 2rem;
}
</style>

<!-- Non-scoped styles for the Teleported modal -->
<style>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  z-index: 100;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem 1rem;
  backdrop-filter: blur(4px);
}

.modal-content {
  position: relative;
  width: 100%;
  max-width: 720px;
  max-height: calc(100vh - 6rem);
  display: flex;
  flex-direction: column;
  padding: 2rem;
  border: 1px solid var(--vp-c-brand-1);
  border-radius: 0.75rem;
  background: var(--vp-c-bg);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  animation: modal-in 0.2s ease;
}

@keyframes modal-in {
  from { opacity: 0; transform: translateY(16px) scale(0.98); }
  to { opacity: 1; transform: translateY(0) scale(1); }
}

.modal-close {
  position: absolute;
  top: 1rem;
  right: 1rem;
  width: 32px;
  height: 32px;
  border: 1px solid var(--vp-c-divider);
  border-radius: 0.375rem;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-2);
  cursor: pointer;
  font-size: 0.9rem;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.modal-close:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.modal-content .detail-header h2 {
  margin: 0 2rem 0 0;
  font-size: 1.25rem;
  border: none;
  padding: 0;
}

.modal-content .detail-byline {
  font-size: 0.85rem;
  color: var(--vp-c-text-3);
  margin: 0.25rem 0 0.75rem;
}

.modal-content .detail-byline code {
  font-size: 0.8rem;
  padding: 3px 8px;
  border-radius: 3px;
  background: var(--vp-c-bg-alt);
  color: var(--vp-c-tip-1, #8be9fd);
  font-family: var(--vp-font-family-mono);
}

.modal-content .detail-byline a {
  color: var(--vp-c-tip-1, #8be9fd);
  text-decoration: none;
  cursor: pointer;
}

.modal-content a {
  cursor: pointer;
}

.modal-content .detail-byline a:hover {
  color: var(--vp-c-brand-1);
}

.modal-content .detail-meta {
  display: flex;
  gap: 0.375rem;
  flex-wrap: wrap;
  margin-bottom: 1.25rem;
}

.modal-content .detail-scroll {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
  margin: 0 -2rem;
  padding: 0 2rem;
}

.modal-content .detail-footer {
  flex-shrink: 0;
  padding-top: 0.75rem;
  border-top: 1px solid var(--vp-c-divider);
  margin-top: 0.75rem;
}

.modal-content .detail-body {
  margin-bottom: 1.5rem;
  font-size: 0.9375rem;
  line-height: 1.7;
  color: var(--vp-c-text-1);
}

.modal-content .detail-body h1,
.modal-content .detail-body h2,
.modal-content .detail-body h3,
.modal-content .detail-body h4 {
  font-size: 1rem;
  margin: 1rem 0 0.5rem;
  border: none;
  padding: 0;
}

.modal-content .detail-body code {
  font-size: 0.85rem;
  padding: 0.125rem 0.375rem;
  background: var(--vp-c-bg-soft);
  border-radius: 0.25rem;
}

.modal-content .detail-body pre {
  padding: 0.75rem;
  border-radius: 0.375rem;
  background: var(--vp-c-bg-soft);
  font-size: 0.85rem;
  line-height: 1.5;
  overflow-x: auto;
}

.modal-content .detail-body pre code {
  padding: 0;
  background: none;
}

.modal-content .detail-section {
  margin-bottom: 1.25rem;
}

.modal-content .detail-section h3 {
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--vp-c-text-3);
  margin: 0 0 0.5rem;
  border: none;
  padding: 0;
}

.modal-content .detail-toml {
  max-height: 500px;
  overflow: auto;
}

.modal-content .detail-toml pre {
  margin: 0;
  padding: 1rem;
  border-radius: 0.375rem;
  background: var(--vp-c-bg-soft);
  font-size: 0.85rem;
  line-height: 1.6;
}

.modal-content .loading {
  font-size: 0.85rem;
  color: var(--vp-c-text-3);
  padding: 0.5rem 0;
}

.modal-content .tile-tag {
  font-size: 0.75rem;
  padding: 0.125rem 0.5rem;
  line-height: 1.25rem;
  border-radius: 0.75rem;
  border: 1px solid var(--vp-c-brand-1);
  color: var(--vp-c-text-2);
  background: transparent;
  text-decoration: none;
  cursor: pointer;
  transition: all 0.2s;
}

.modal-content .tile-tag:hover {
  box-shadow: 0 0 8px var(--vp-c-brand-1);
}

.modal-content .tile-tag-active {
  box-shadow: 0 0 8px var(--vp-c-brand-1);
}

.modal-copy-btn {
  width: 100%;
  padding: 0.625rem;
  border: 1px solid var(--vp-c-brand-1);
  border-radius: 0.375rem;
  background: transparent;
  color: var(--vp-c-brand-1);
  cursor: pointer;
  font-size: 0.9rem;
  transition: all 0.15s;
}

.modal-copy-btn:hover {
  background: var(--vp-c-brand-1);
  color: var(--vp-c-bg);
}

.modal-import-cmd {
  margin-top: 0.5rem;
  font-size: 0.75rem;
  color: var(--vp-c-text-3);
  word-break: break-all;
}

.modal-import-cmd code {
  font-family: var(--vp-font-family-mono);
}

.modal-content .tile-actions {
  display: flex;
  gap: 0;
  margin-top: 0.75rem;
  width: 100%;
}

.modal-content .import-detail {
  width: 100%;
  margin-top: 0.5rem;
}

.modal-content .copy-btn {
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

.modal-content .copy-btn:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.modal-content .expand-btn {
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

.modal-content .expand-btn:hover {
  border-color: var(--vp-c-brand-1);
  color: var(--vp-c-brand-1);
}

.modal-content .import-detail {
  margin-top: 0.5rem;
}

.modal-content .import-input {
  width: 100%;
  padding: 0.375rem 0.5rem;
  border: 1px solid var(--vp-c-divider);
  border-radius: 0.375rem;
  background: var(--vp-c-bg-soft);
  color: var(--vp-c-text-1);
  font-family: var(--vp-font-family-mono);
  font-size: 0.8125rem;
  line-height: 1.6;
  outline: none;
  resize: none;
  word-break: break-all;
}

.modal-content .import-input:focus {
  border-color: var(--vp-c-brand-1);
}

.modal-content .import-hint {
  margin: 0.375rem 0 0;
  font-size: 0.75rem;
  color: var(--vp-c-text-3);
}

.modal-content .import-hint a {
  color: var(--vp-c-brand-1);
  text-decoration: underline;
  text-underline-offset: 2px;
}

.modal-content .import-hint a:hover {
  color: var(--vp-c-brand-2);
}

.modal-content .activate-hint {
  margin-top: 1rem;
  font-size: 0.85rem;
  color: var(--vp-c-text-2);
}

.modal-content .activate-hint p {
  margin: 0 0 0.375rem;
}

.modal-content .activate-hint pre {
  margin: 0;
  padding: 0.5rem 0.75rem;
  border-radius: 0.375rem;
  background: var(--vp-c-bg-alt);
  font-size: 0.8rem;
  line-height: 1.6;
}

.modal-content .activate-hint code {
  background: none;
  padding: 0;
}
</style>
