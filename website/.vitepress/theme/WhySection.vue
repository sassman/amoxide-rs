<template>
  <div class="why-section">
    <h2>Why amoxide?</h2>
    <div class="why-showcase">
      <div class="why-prose">
        <p>
          You know what a good alias system looks like — you've tried building one.
          A block in <code>.zshrc</code>, a few <code>alias</code> lines, maybe a Makefile.
          It works until the third project, or until a colleague asks why <code>l</code>
          runs something surprising on their machine.
        </p>
        <p>
          The real problem is scope. Shell aliases are global by default. A shortcut that
          makes sense inside one project leaks into every terminal window. Cleaning it up
          means editing dotfiles, sourcing, checking. The alias you added for client A still
          fires in client B's directory six months later.
        </p>
        <p>
          amoxide solves scope. Aliases can live in a project directory (<code>.aliases</code>
          file, auto-loaded on <code>cd</code>), in a named profile (activated explicitly with
          <code>am use &lt;name&gt;</code>), or globally. Each layer loads independently and unloads
          cleanly when it leaves scope. The TUI gives you a live map of what's active — no
          mental overhead, no guessing.
        </p>
        <p>
          The subcommand alias system is where it gets interesting. Instead of a flat namespace
          full of cryptic prefixes, you define a routing scheme that makes sense to you:
          <code>k get po</code> expands to <code>kubectl get pods</code>, tab completion still
          works, and you chose the abbreviations.
        </p>
      </div>
      <div class="why-image">
        <img
          src="/am-subcommand-alias-show-case.png"
          alt="amoxide subcommand alias showcase"
          title="Click to enlarge"
          @click="open = true"
        />
      </div>
    </div>
    <a href="/guide/" class="why-cta">Get Started →</a>

    <!-- Lightbox -->
    <Teleport to="body">
      <div v-if="open" class="lightbox" @click="open = false">
        <div class="lightbox-inner" @click.stop>
          <img src="/am-subcommand-alias-show-case.png" alt="amoxide subcommand alias showcase" />
          <button class="lightbox-close" @click="open = false" aria-label="Close">✕</button>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const open = ref(false)

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'Escape') open.value = false
}

onMounted(() => window.addEventListener('keydown', onKeydown))
onUnmounted(() => window.removeEventListener('keydown', onKeydown))
</script>

<style scoped>
.why-section {
  margin: 48px 0;
}

.why-section h2 {
  font-size: 24px;
  font-weight: 700;
  margin-bottom: 24px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--vp-c-divider);
}

.why-showcase {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 40px;
  align-items: start;
  margin-top: 24px;
}

.why-image img {
  width: 100%;
  border-radius: 10px;
  border: 1px solid var(--vp-c-divider);
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.18);
  cursor: zoom-in;
  transition: box-shadow 0.15s ease, transform 0.15s ease;
}

.why-image img:hover {
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.28);
  transform: scale(1.01);
}

.why-prose p {
  color: var(--vp-c-text-2);
  margin-bottom: 16px;
  line-height: 1.8;
  font-size: 15px;
}

.why-prose p:last-child {
  margin-bottom: 0;
}

.why-cta {
  display: inline-block;
  margin-top: 20px;
  color: var(--vp-c-brand-1);
  font-weight: 500;
  text-decoration: none;
}

.why-cta:hover {
  text-decoration: underline;
}

/* Lightbox */
.lightbox {
  position: fixed;
  inset: 0;
  z-index: 9999;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: zoom-out;
  animation: lb-in 0.15s ease;
}

@keyframes lb-in {
  from { opacity: 0; }
  to   { opacity: 1; }
}

.lightbox-inner {
  position: relative;
  max-width: min(90vw, 1100px);
  max-height: 90vh;
  cursor: default;
}

.lightbox-inner img {
  display: block;
  max-width: 100%;
  max-height: 90vh;
  border-radius: 10px;
  box-shadow: 0 24px 80px rgba(0, 0, 0, 0.5);
  object-fit: contain;
}

.lightbox-close {
  position: absolute;
  top: -14px;
  right: -14px;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--vp-c-bg);
  border: 1px solid var(--vp-c-divider);
  color: var(--vp-c-text-1);
  font-size: 13px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.lightbox-close:hover {
  background: var(--vp-c-bg-soft);
}

@media (max-width: 768px) {
  .why-showcase {
    grid-template-columns: 1fr;
  }
}
</style>
