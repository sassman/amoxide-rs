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
        <img src="/am-subcommand-alias-show-case.png" alt="amoxide subcommand alias showcase" />
      </div>
    </div>
    <a href="/guide/" class="why-cta">Get Started →</a>
  </div>
</template>

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

@media (max-width: 768px) {
  .why-showcase {
    grid-template-columns: 1fr;
  }
}
</style>
