<template>
  <div class="use-cases-section">
    <h2>Anwendungsfälle</h2>

    <!-- 01 Polyglot-Entwickler -->
    <div class="uc">
      <div class="uc-head">
        <div class="uc-num">01</div>
        <div class="uc-titles">
          <div class="role-badge">Profile</div>
          <h3>Der Polyglot-Entwickler</h3>
          <span class="uc-sub">Zwei Stacks. Ein Satz Aliase. Null mentaler Aufwand beim Wechsel.</span>
        </div>
      </div>
      <div class="uc-body">
        <div class="uc-prose">
          <p>
            Du arbeitest tagsüber an einem Node-Dienst und abends an einem Rust-Seitenprojekt.
            Der Kontextwechsel ist eigentlich klein — gleicher Editor, gleiches Terminal, gleiche
            Arbeitsroutine. Aber sobald du linten oder testen willst, musst du kurz nachdenken,
            in welcher Welt du gerade bist. Ist es <code>npm run lint</code> oder
            <code>cargo clippy</code>? <code>npm run dev</code> oder <code>cargo run</code>?
          </p>
          <p>
            Die übliche Antwort sind Präfix-Aliase: <code>nl</code> für Node-Lint,
            <code>rl</code> für Rust-Lint. Es funktioniert, aber die Hände bauen nie echtes
            Muskelgedächtnis auf. Du denkst immer einen Schritt voraus — "welcher Präfix gilt
            hier?" — statt einfach zu tippen. Diese kleine Reibung summiert sich über den Tag.
          </p>
          <p>
            Mit zwei amoxide-Profilen vergibst du in beiden Kontexten denselben kurzen Namen
            für jede Aufgabe. <code>l</code> ist immer Lint. <code>t</code> ist immer Test.
            <code>b</code> ist immer Build. <code>r</code> ist immer Run. Nach ein paar Tagen
            fühlt es sich nicht mehr wie ein Werkzeug an, sondern wie eine Gewohnheit. Der
            Profilwechsel — <code>am use node</code> oder <code>am use rust</code> — ist die
            eine Entscheidung zu Beginn einer Session.
          </p>
          <table>
            <thead>
              <tr><th>Alias</th><th>Node-Profil</th><th>Rust-Profil</th></tr>
            </thead>
            <tbody>
              <tr><td><code>l</code></td><td><code>npm run lint</code></td><td><code>cargo clippy --locked --all-targets -- -D warnings</code></td></tr>
              <tr><td><code>t</code></td><td><code>npm test</code></td><td><code>cargo test</code></td></tr>
              <tr><td><code>b</code></td><td><code>npm run build</code></td><td><code>cargo build --release</code></td></tr>
              <tr><td><code>r</code></td><td><code>npm run dev</code></td><td><code>cargo run</code></td></tr>
            </tbody>
          </table>
          <p class="uc-note">Alle Profile liegen in <code>~/.config/amoxide/profiles.toml</code> — reines TOML, versionierbar neben deinen Dotfiles.</p>
        </div>
        <div class="code-stack">
          <div class="code-panel">
            <div class="ctab"><span>einmalig einrichten</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-kw">am</span> profile add node
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> node <span class="t-hl">l</span> <span class="t-st">"npm run lint"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> node <span class="t-hl">t</span> <span class="t-st">"npm test"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> node <span class="t-hl">b</span> <span class="t-st">"npm run build"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> node <span class="t-hl">r</span> <span class="t-st">"npm run dev"</span>

<span class="t-pr">❯ </span><span class="t-kw">am</span> profile add rust
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> rust <span class="t-hl">l</span> <span class="t-st">"cargo clippy --locked --all-targets -- -D warnings"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> rust <span class="t-hl">t</span> <span class="t-st">"cargo test"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> rust <span class="t-hl">b</span> <span class="t-st">"cargo build --release"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> rust <span class="t-hl">r</span> <span class="t-st">"cargo run"</span></pre>
          </div>
          <div class="code-panel">
            <div class="ctab"><span>danach jeden Tag</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-kw">am use</span> node
<span class="t-out">am: profile node activated — 4 loaded: l, b, r, t</span>
<span class="t-pr">❯ </span><span class="t-hl">l</span>    <span class="t-cm">  # npm run lint</span>
<span class="t-pr">❯ </span><span class="t-hl">t</span>    <span class="t-cm">  # npm test</span>

<span class="t-pr">❯ </span><span class="t-kw">am use</span> rust
<span class="t-out">am: profile rust activated — 4 loaded: l, b, r, t</span>
<span class="t-pr">❯ </span><span class="t-hl">l</span>    <span class="t-cm">  # cargo clippy --locked --all-targets -- -D warnings</span>
<span class="t-pr">❯ </span><span class="t-hl">t</span>    <span class="t-cm">  # cargo test</span></pre>
          </div>
        </div>
      </div>
    </div>

    <!-- 02 Gemeinsames Team-Projekt -->
    <div class="uc">
      <div class="uc-head">
        <div class="uc-num">02</div>
        <div class="uc-titles">
          <div class="role-badge">Projekt-Aliase</div>
          <h3>Das gemeinsame Team-Projekt</h3>
          <span class="uc-sub">Repo klonen, Shortcuts bekommen. Kein README, kein Onboarding-Dokument.</span>
        </div>
      </div>
      <div class="uc-body">
        <div class="uc-prose">
          <p>
            Euer Repo hat ein Justfile mit 30-odd Rezepten: CI-Checks, Integrationstests,
            Datenbank-Migrationen, Staging-Deployments. Neue Teammitglieder wissen nicht,
            dass <code>just</code> existiert. Erfahrene vergessen Rezepte, die sie seit drei
            Wochen nicht getippt haben. Der Befehl für Integrationstests ist
            <code>cargo test --features integration -- --test-threads 1</code> und steht nur
            in der CI-Konfiguration.
          </p>
          <p>
            Jeder hat seine eigenen Shortcuts. Manche nutzen <code>make</code>, manche
            <code>just</code>, manche kopieren aus Slack. Das Ergebnis: Ein Team, das
            technisch an derselben Codebase arbeitet, aber mit völlig unterschiedlichem
            Muskelgedächtnis. Neue Leute verbringen die erste Woche damit zu fragen: "Wie
            starte ich die Tests?"
          </p>
          <p>
            Eine <code>.aliases</code>-Datei im Repo-Root löst das für alle. Wenn ein
            Entwickler ins Projektverzeichnis wechselt, lädt amoxide sie automatisch.
            <code>am ls</code> zeigt jeden verfügbaren Shortcut — kein Fragen, kein Raten.
            Ändert sich ein Befehl, aktualisiert ein Commit ihn für das gesamte Team.
          </p>
          <p class="uc-note">Die <code>.aliases</code>-Datei nutzt dasselbe TOML-Format wie Profile. Teams können auch Subcommand-Aliase für komplexe Aufrufe hinzufügen, die sonst nur im Kopf einzelner Personen existieren würden.</p>
        </div>
        <div class="code-stack">
          <div class="code-panel">
            <div class="ctab"><span>einmalig ins Repo einchecken</span></div>
            <pre v-pre><span class="t-cm"># im Projekt-Root</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">ti</span>     <span class="t-st">"cargo test --features integration -- --test-threads 1"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">ci</span>     <span class="t-st">"just ci-check"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">db</span>     <span class="t-st">"just db-migrate --env staging"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">deploy</span> <span class="t-st">"just deploy --target production"</span>
<span class="t-cm">
# committen</span>
<span class="t-pr">❯ </span><span class="t-kw">git</span> add .aliases &amp;&amp; git commit <span class="t-fl">-m</span> <span class="t-st">"Projekt-Aliase hinzufügen"</span></pre>
          </div>
          <div class="code-panel">
            <div class="ctab"><span>für jeden Entwickler, automatisch</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-kw">cd</span> ~/work/myproject
<span class="t-out">am: loaded .aliases
  ci     → just ci-check
  db     → just db-migrate --env staging
  deploy → just deploy --target production
  ti     → cargo test --features integration -- --test-threads 1</span>

<span class="t-pr">❯ </span><span class="t-kw">am ls</span>
<span class="t-out">📁 project (.aliases)
  ├─ ci     → just ci-check
  ├─ db     → just db-migrate --env staging
  ├─ deploy → just deploy --target production
  ╰─ ti     → cargo test --features integration ...</span>

<span class="t-pr">❯ </span><span class="t-hl">ti</span>    <span class="t-cm">  # cargo test --features integration -- --test-threads 1</span></pre>
          </div>
        </div>
      </div>
    </div>

    <!-- 03 Kubernetes-Operator -->
    <div class="uc">
      <div class="uc-head">
        <div class="uc-num">03</div>
        <div class="uc-titles">
          <div class="role-badge">Subcommand-Aliase</div>
          <h3>Der Kubernetes-Operator</h3>
          <span class="uc-sub">Ersetze 120 flache Aliase durch ein Schema, das du selbst entworfen hast.</span>
        </div>
      </div>
      <div class="uc-body">
        <div class="uc-prose">
          <p>
            Oh-my-zsh liefert über 120 kubectl-Aliase. <code>kgp</code> ist get pods.
            <code>kdp</code> ist describe pod. <code>kgpw</code> ist get pods --watch. Das
            Schema wurde von jemand anderem entworfen, verwendet Abkürzungen, die du nicht
            gewählt hast, und lässt sich nicht anpassen, ohne das Plugin zu forken. Entweder
            du nimmst alle 120 oder keine.
          </p>
          <p>
            Und die Aliase, die du wirklich brauchst, sind gar nicht dabei.
            <code>kubectl logs deployment/api --since=10m --tail=100</code> — den du bei jedem
            Incident tippst — hat keinen Alias. Also tippst du ihn vollständig, oder du fügst
            <code>klogs-api</code> zu deiner <code>.zshrc</code> hinzu und vergisst ihn drei
            Wochen später, begraben neben den anderen Einzel-Shortcuts.
          </p>
          <p>
            amoxide-Subcommand-Aliase lassen dich das Schema selbst definieren. Ein Basis-Alias
            leitet zu einem Befehl weiter, und kurze Subcommand-Sequenzen expandieren zu den
            Flags, die für dich sinnvoll sind. <code>k get po</code> wird zu
            <code>kubectl get pods</code>. <code>k logs api</code> wird genau zu dem Aufruf,
            den du immer brauchst. Alles liegt an einem Ort, ist mit <code>am ls</code> sichtbar
            und gehört dir — ohne fremde Konfiguration zu berühren.
          </p>
          <p class="uc-note">Subcommand-Aliase können in einem globalen Profil (immer verfügbar) oder in einer Projekt-<code>.aliases</code>-Datei leben (nur aktiv in diesem Repo).</p>
        </div>
        <div class="code-stack">
          <div class="code-panel">
            <div class="ctab"><span>oh-my-zsh kubectl-Plugin ersetzen</span></div>
            <pre v-pre><span class="t-cm"># Basis-Alias — k leitet zu kubectl weiter</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k</span> kubectl

<span class="t-cm"># Subcommand-Routing</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k:get:po</span>   <span class="t-st">"get pods"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k:get:svc</span>  <span class="t-st">"get svc"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k:desc:po</span>  <span class="t-st">"describe pod"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k:logs:api</span> <span class="t-st">"logs deployment/api --since=10m --tail=100"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k:rr</span>      <span class="t-st">"rollout restart deployment"</span></pre>
          </div>
          <div class="code-panel">
            <div class="ctab"><span>danach einfach tippen</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-hl">k</span> get po
<span class="t-out">NAME                   READY   STATUS    RESTARTS
api-7d4f9b8c6-xk2pm    1/1     Running   0</span>

<span class="t-pr">❯ </span><span class="t-hl">k</span> logs api
<span class="t-out">2026-04-15T09:12:44Z INFO  server started on :8080</span>

<span class="t-pr">❯ </span><span class="t-hl">k</span> rr api
<span class="t-out">deployment.apps/api restarted</span>

<span class="t-cm"># Tab-Completion funktioniert weiterhin</span></pre>
          </div>
        </div>
      </div>
    </div>

    <!-- 04 Freiberufler -->
    <div class="uc">
      <div class="uc-head">
        <div class="uc-num">04</div>
        <div class="uc-titles">
          <div class="role-badge">Projekt-Aliase</div>
          <h3>Der Freiberufler zwischen Kunden</h3>
          <span class="uc-sub">Jedes Kundenverzeichnis lädt seinen eigenen Kontext. Nichts vermischt sich.</span>
        </div>
      </div>
      <div class="uc-body">
        <div class="uc-prose">
          <p>
            Du arbeitest mit drei Kunden. Jeder hat einen anderen Cloud-Provider, eine andere
            Staging-URL, eine andere Deployment-Pipeline, ein anderes VPN. Deine
            <code>.zshrc</code> enthält drei auskommentierte Blöcke, die du von Hand umschaltest,
            wenn du den Kontext wechselst. Der falsche <code>deploy</code>-Alias hat einmal zehn
            Minuten lang auf die falsche Staging-Umgebung gezeigt, bevor du es gemerkt hast.
          </p>
          <p>
            Das eigentliche Problem: Globale Aliase gehören niemandem. Sie häufen sich an —
            alte Kunden-Shortcuts neben aktuellen, manche kaputt, manche überlagern sich
            gegenseitig. Alle paar Monate machst du ein Aufräumen, das einen Nachmittag
            dauert und sich trotzdem nie vollständig anfühlt.
          </p>
          <p>
            Mit amoxide lebt jeder Kunde in seinem eigenen Verzeichnis. <code>cd client-a/</code>
            lädt automatisch deren Aliase — ihren AWS-Shortcut, ihren Deploy-Befehl, ihren
            Staging-URL-Öffner. <code>cd client-b/</code> entlädt alles und lädt deren Aliase.
            Nichts vermischt sich. Die <code>.aliases</code>-Datei für jeden Kunden kann im
            Projekt-Repo liegen — versioniert, nachvollziehbar, teilbar.
          </p>
          <p class="uc-note">Projekt-Aliase erfordern einmalig <code>am trust</code> pro Repository — amoxide lädt keine Datei automatisch, die es noch nicht gesehen hat.</p>
        </div>
        <div class="code-stack">
          <div class="code-panel">
            <div class="ctab"><span>client-a/.aliases</span></div>
            <pre v-pre><span class="t-cm"># einmalig hinzugefügt, ins Repo eingecheckt</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">deploy</span>  <span class="t-st">"./scripts/deploy.sh --env staging"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">logs</span>    <span class="t-st">"ssh app@client-a.internal journalctl -u api -f"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">stage</span>   <span class="t-st">"open https://staging.client-a.internal"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">tf:plan</span> <span class="t-st">"terraform plan -var-file=client-a.tfvars"</span></pre>
          </div>
          <div class="code-panel">
            <div class="ctab"><span>Kunden wechseln — nichts zu merken</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-kw">cd</span> ~/clients/client-a
<span class="t-out">am: loaded .aliases
  deploy  → ./scripts/deploy.sh --env staging
  logs    → ssh app@client-a.internal journalctl -u api -f
  stage   → open https://staging.client-a.internal
  tf:plan → terraform plan -var-file=client-a.tfvars</span>
<span class="t-pr">❯ </span><span class="t-hl">deploy</span>

<span class="t-pr">❯ </span><span class="t-kw">cd</span> ~/clients/client-b
<span class="t-out">am: loaded .aliases
  infra:plan → terraform -chdir=infra plan
  preview    → open https://preview.client-b.internal
  ship       → ./scripts/ship.sh</span>
<span class="t-pr">❯ </span><span class="t-hl">preview</span>

<span class="t-cm"># keine veralteten Aliase. keine Vermischung.</span></pre>
          </div>
        </div>
      </div>
    </div>

    <!-- 05 Das große Dotfile-Aufräumen -->
    <div class="uc">
      <div class="uc-head">
        <div class="uc-num">05</div>
        <div class="uc-titles">
          <div class="role-badge">Profile</div>
          <h3>Das große Dotfile-Aufräumen</h3>
          <span class="uc-sub">Acht Jahre Aliase. Ein Nachmittag, um Ordnung zu schaffen.</span>
        </div>
      </div>
      <div class="uc-body">
        <div class="uc-prose">
          <p>
            Acht Jahre Entwicklung, etwa 200 Aliase. <code>gcm</code> könnte
            <code>git commit -m</code> oder <code>git checkout master</code> sein, je nachdem,
            wann du ihn geschrieben hast. Manche beziehen sich auf Projekte, an denen du nicht
            mehr arbeitest. Manche sind kaputt, weil das Tool umbenannt wurde.
            <code>alias | grep git</code> gibt 40 Zeilen zurück und du findest trotzdem nicht
            den einen, den du suchst.
          </p>
          <p>
            Der einzige Weg, einen Alias zu entdecken, ist, sich zu erinnern, dass man ihn
            erstellt hat. Der einzige Weg, sie aufzuräumen, ist, durch eine Datei zu scrollen,
            die man seit Jahren nicht vollständig gelesen hat. Du weißt, dass es gemacht werden
            muss. Du schiebst es immer wieder auf.
          </p>
          <p>
            amoxide macht daraus einen handhabbaren Nachmittag. Verschiebe Aliase in benannte
            Profile — <code>work</code>, <code>personal</code>, <code>git</code>, <code>rust</code>,
            <code>ops</code> — jedes ein reiner TOML-Abschnitt, den du lesen, kommentieren und
            verstehen kannst. <code>am ls</code> zeigt genau, was gerade aktiv ist. Nichts wird
            geladen, außer du oder ein Projektverzeichnis haben darum gebeten. Aus dem 200-Alias-Haufen
            werden fünf Profile, denen du wirklich vertraust.
          </p>
          <p class="uc-note">Profile können zusammen aktiviert werden: <code>am use git work</code> schichtet beide übereinander, wobei spätere Profile bei Namenskonflikten Vorrang haben.</p>
        </div>
        <div class="code-stack">
          <div class="code-panel">
            <div class="ctab"><span>aus .zshrc migrieren</span></div>
            <pre v-pre><span class="t-cm"># organisierte Profile erstellen</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> profile add git
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> git <span class="t-hl">gs</span>      <span class="t-st">"git status --short"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> git <span class="t-hl">gp</span>      <span class="t-st">"git push"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> git <span class="t-hl">gl</span>      <span class="t-st">"git log --oneline --graph -20"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> git <span class="t-hl">gm</span>      <span class="t-st">"git commit -m"</span>

<span class="t-pr">❯ </span><span class="t-kw">am</span> profile add work
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> work <span class="t-hl">vpn</span>    <span class="t-st">"sudo openconnect vpn.company.com"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> work <span class="t-hl">jira</span>   <span class="t-st">"open https://company.atlassian.net"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-p</span> work <span class="t-hl">standup</span> <span class="t-st">"open https://meet.google.com/xyz"</span></pre>
          </div>
          <div class="code-panel">
            <div class="ctab"><span>was gerade aktiv ist</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-kw">am use</span> git work
<span class="t-out">am: profile git activated — 4 loaded: gl, gm, gp, gs
am: profile work activated — 3 loaded: jira, standup, vpn</span>

<span class="t-pr">❯ </span><span class="t-kw">am ls</span>
<span class="t-out">├─● git (active)
│   ├─ gl     → git log --oneline --graph -20
│   ├─ gm     → git commit -m
│   ├─ gp     → git push
│   ╰─ gs     → git status --short
│
╰─● work (active)
    ├─ jira    → open https://company.atlassian.net
    ├─ standup → open https://meet.google.com/xyz
    ╰─ vpn     → sudo openconnect vpn.company.com</span></pre>
          </div>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
.use-cases-section {
  margin: 48px -24px 0;
}

.use-cases-section h2 {
  padding: 0 24px;
  font-size: 24px;
  font-weight: 700;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--vp-c-divider);
  margin-bottom: 0;
}

.uc {
  padding: 56px 24px;
  border-top: 1px solid var(--vp-c-divider);
}

.uc:nth-child(even) {
  background: var(--vp-c-bg-alt);
}

.uc-head {
  display: flex;
  align-items: baseline;
  gap: 20px;
  margin-bottom: 28px;
}

.uc-num {
  font-size: 72px;
  font-weight: 900;
  line-height: 1;
  flex-shrink: 0;
  min-width: 100px;
  text-align: right;
  color: var(--vp-c-divider);
  user-select: none;
}

.uc-titles { flex: 1; }

.role-badge {
  display: inline-block;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--vp-c-brand-1);
  background: var(--vp-c-brand-soft);
  border: 1px solid var(--vp-c-brand-soft);
  padding: 3px 10px;
  border-radius: 4px;
  margin-bottom: 10px;
}

.uc-titles h3 {
  font-size: 24px;
  font-weight: 700;
  border: none;
  margin: 0 0 4px;
  padding: 0;
  line-height: 1.25;
}

.uc-sub {
  font-size: 15px;
  color: var(--vp-c-text-3);
  display: block;
}

.uc-body {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 36px;
  align-items: start;
}

.uc-prose {
  min-width: 0;
}

.uc-prose p {
  color: var(--vp-c-text-2);
  font-size: 15px;
  line-height: 1.8;
  margin-bottom: 14px;
}

.uc-prose p:last-child { margin-bottom: 0; }

.uc-prose table {
  width: 100%;
  border-collapse: collapse;
  margin: 16px 0;
  font-size: 13px;
}

.uc-prose th {
  background: var(--vp-c-bg-alt);
  color: var(--vp-c-brand-1);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  padding: 7px 10px;
  text-align: left;
  border: 1px solid var(--vp-c-divider);
}

.uc-prose td {
  padding: 7px 10px;
  border: 1px solid var(--vp-c-divider);
  color: var(--vp-c-text-2);
  font-size: 13px;
}

.uc-prose td:first-child {
  color: var(--vp-c-text-1);
  font-weight: 500;
}

.uc-note {
  font-size: 13px;
  color: var(--vp-c-text-3);
  margin-top: 16px !important;
  line-height: 1.6;
}

.code-stack {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-width: 0;
}

.code-panel {
  min-width: 0;
}

.ctab {
  background: var(--vp-c-bg-alt);
  border: 1px solid var(--vp-c-divider);
  border-bottom: none;
  border-radius: 8px 8px 0 0;
  padding: 6px 14px;
  font-size: 11px;
  color: var(--vp-c-text-3);
  font-family: var(--vp-font-family-mono);
}

.ctab span { color: var(--vp-c-brand-1); }

.code-panel pre {
  background: var(--vp-code-block-bg, var(--vp-c-bg-alt));
  border: 1px solid var(--vp-c-divider);
  border-radius: 0 0 8px 8px;
  padding: 16px 18px;
  font-family: var(--vp-font-family-mono);
  font-size: 13px;
  line-height: 1.85;
  overflow-x: auto;
  color: var(--vp-c-text-1);
  margin: 0;
}

:deep(.t-pr)  { color: var(--vp-c-text-3); user-select: none; }
:deep(.t-kw)  { color: var(--vp-c-brand-1); }
:deep(.t-hl)  { color: var(--vp-c-tip-1, #8be9fd); }
:deep(.t-st)  { color: var(--vp-c-warning-1, #f1fa8c); }
:deep(.t-cm)  { color: var(--vp-c-text-3); }
:deep(.t-fl)  { color: #60a5fa; }
:deep(.t-out) { color: var(--vp-c-text-2); }

@media (max-width: 900px) {
  .uc-body { grid-template-columns: 1fr; }
  .uc-num { font-size: 48px; min-width: 64px; }
}

@media (max-width: 640px) {
  .uc { padding: 40px 16px; }
  .uc-head { gap: 12px; }
}
</style>
