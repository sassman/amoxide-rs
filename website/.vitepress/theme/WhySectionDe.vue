<template>
  <div class="why-section">
    <h2>Warum amoxide?</h2>
    <div class="why-showcase">
      <div class="why-prose">
        <p>
          Du weißt, wie ein gutes Alias-System aussehen sollte — du hast schon versucht, eines
          zu bauen. Ein Block in <code>.zshrc</code>, ein paar <code>alias</code>-Zeilen,
          vielleicht ein Makefile. Es funktioniert, bis zum dritten Projekt — oder bis ein
          Kollege fragt, warum <code>l</code> auf seinem Rechner etwas Unerwartetes tut.
        </p>
        <p>
          Das eigentliche Problem ist der Geltungsbereich. Shell-Aliase sind standardmäßig
          global. Ein Kürzel, das in einem Projekt sinnvoll ist, taucht in jedem anderen
          Terminalfenster auf. Aufräumen bedeutet: Dotfiles bearbeiten, sourcen, prüfen.
          Der Alias, den du für Client A angelegt hast, feuert noch sechs Monate später im
          Verzeichnis von Client B.
        </p>
        <p>
          amoxide löst das Geltungsbereich-Problem. Aliase können in einem Projektverzeichnis
          liegen (eine <code>.aliases</code>-Datei, die beim <code>cd</code> automatisch geladen
          wird), in einem benannten Profil (explizit aktiviert mit <code>am use &lt;name&gt;</code>)
          oder global. Jede Schicht lädt sich unabhängig und entlädt sich sauber, wenn sie den
          Scope verlässt. Das TUI zeigt dir jederzeit, was gerade aktiv ist — kein Rätselraten.
        </p>
        <p>
          Das Subcommand-Alias-System macht es besonders interessant. Statt einem flachen
          Namensraum voller kryptischer Präfixe definierst du ein Schema, das für dich Sinn
          ergibt: <code>k get po</code> expandiert zu <code>kubectl get pods</code>,
          Tab-Completion funktioniert weiterhin, und die Abkürzungen hast du selbst gewählt.
        </p>
      </div>
      <div class="why-image">
        <img
          src="/am-subcommand-alias-show-case.png"
          alt="amoxide Subcommand-Alias-Showcase"
          title="Klicken zum Vergrößern"
          @click="open = true"
        />
      </div>
    </div>
    <a href="/de/guide/" class="why-cta">Erste Schritte →</a>

    <!-- Lightbox -->
    <Teleport to="body">
      <div v-if="open" class="lightbox" @click="open = false">
        <div class="lightbox-inner" @click.stop>
          <img src="/am-subcommand-alias-show-case.png" alt="amoxide Subcommand-Alias-Showcase" />
          <button class="lightbox-close" @click="open = false" aria-label="Schließen">✕</button>
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
