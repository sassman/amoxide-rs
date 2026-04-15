<template>
  <div class="use-cases-section">
    <h2>Use Cases</h2>

    <!-- 01 Polyglot Developer -->
    <div class="uc">
      <div class="uc-head">
        <div class="uc-num">01</div>
        <div class="uc-titles">
          <div class="role-badge">Profiles</div>
          <h3>The Polyglot Developer</h3>
          <span class="uc-sub">Two stacks. One set of aliases. Zero mental switching.</span>
        </div>
      </div>
      <div class="uc-body">
        <div class="uc-prose">
          <p>
            You work on a Node service at work and a Rust side project in the evenings. The context
            switch between them is small — same editor, same terminal, same workflow instincts. But
            the moment you need to lint or run tests, you have to stop and remember which world you're
            in. Is it <code>npm run lint</code> or <code>cargo clippy</code>? Is it <code>npm run dev</code>
            or <code>cargo run</code>?
          </p>
          <p>
            The usual answer is prefixed aliases: <code>nl</code> for node-lint, <code>rl</code> for
            rust-lint. It works, but it means your hands never build real muscle memory. You're always
            thinking one step ahead — "which prefix applies here?" — instead of just typing. That small
            friction compounds across a day.
          </p>
          <p>
            With two amoxide profiles you give the same short name to each task in both contexts.
            <code>l</code> is always lint. <code>t</code> is always test. <code>b</code> is always
            build. <code>r</code> is always run. After a few days it stops feeling like a tool and
            starts feeling like a habit. The profile switch — <code>am use node</code> or
            <code>am use rust</code> — is the one decision you make at the start of a session.
          </p>
          <table>
            <thead>
              <tr><th>Alias</th><th>Node profile</th><th>Rust profile</th></tr>
            </thead>
            <tbody>
              <tr><td><code>l</code></td><td><code>npm run lint</code></td><td><code>cargo clippy --locked --all-targets -- -D warnings</code></td></tr>
              <tr><td><code>t</code></td><td><code>npm test</code></td><td><code>cargo test</code></td></tr>
              <tr><td><code>b</code></td><td><code>npm run build</code></td><td><code>cargo build --release</code></td></tr>
              <tr><td><code>r</code></td><td><code>npm run dev</code></td><td><code>cargo run</code></td></tr>
            </tbody>
          </table>
          <p class="uc-note">All profiles live in <code>~/.config/amoxide/profiles.toml</code> — plain TOML, versionable alongside your dotfiles.</p>
        </div>
        <div class="code-stack">
          <div class="code-panel">
            <div class="ctab"><span>set up once</span></div>
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
            <div class="ctab"><span>every day after that</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-kw">am use</span> node
<span class="t-out">node activated — 4 loaded: l, b, r, t</span>
<span class="t-pr">❯ </span><span class="t-hl">l</span>    <span class="t-cm">  # npm run lint</span>
<span class="t-pr">❯ </span><span class="t-hl">t</span>    <span class="t-cm">  # npm test</span>

<span class="t-pr">❯ </span><span class="t-kw">am use</span> rust
<span class="t-out">rust activated — 4 loaded: l, b, r, t</span>
<span class="t-pr">❯ </span><span class="t-hl">l</span>    <span class="t-cm">  # cargo clippy --locked --all-targets -- -D warnings</span>
<span class="t-pr">❯ </span><span class="t-hl">t</span>    <span class="t-cm">  # cargo test</span></pre>
          </div>
        </div>
      </div>
    </div>

    <!-- 02 Shared Team Project -->
    <div class="uc">
      <div class="uc-head">
        <div class="uc-num">02</div>
        <div class="uc-titles">
          <div class="role-badge">Project aliases</div>
          <h3>The Shared Team Project</h3>
          <span class="uc-sub">Clone the repo, get the shortcuts. No README, no onboarding doc.</span>
        </div>
      </div>
      <div class="uc-body">
        <div class="uc-prose">
          <p>
            Your team's repo has a Justfile with 30-odd recipes: CI checks, integration test runs,
            database migrations, staging deployments. New joiners don't know <code>just</code> exists.
            Veterans forget the recipe they haven't typed in three weeks. The command to run integration
            tests is <code>cargo test --features integration -- --test-threads 1</code> and lives only
            in the CI config.
          </p>
          <p>
            Everyone has their own shortcuts. Some run <code>make</code>, some run <code>just</code>,
            some copy-paste from Slack. The result is a team that's technically using the same codebase
            but working with completely different muscle memory. New people spend their first week asking
            "how do I run the tests?"
          </p>
          <p>
            A <code>.aliases</code> file committed to the repo root fixes this for everyone. When a
            developer <code>cd</code>s into the project, amoxide loads the file automatically.
            <code>am ls</code> shows every shortcut available — no asking, no guessing. When a command
            changes, one commit updates it for the whole team.
          </p>
          <p class="uc-note">The <code>.aliases</code> file uses the same TOML format as profiles. Teams can also add subcommand aliases for complex invocations that would otherwise live only in someone's head.</p>
        </div>
        <div class="code-stack">
          <div class="code-panel">
            <div class="ctab"><span>add to the repo once</span></div>
            <pre v-pre><span class="t-cm"># in the project root</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">ti</span>     <span class="t-st">"cargo test --features integration -- --test-threads 1"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">ci</span>     <span class="t-st">"just ci-check"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">db</span>     <span class="t-st">"just db-migrate --env staging"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">deploy</span> <span class="t-st">"just deploy --target production"</span>
<span class="t-cm">
# commit it</span>
<span class="t-pr">❯ </span><span class="t-kw">git</span> add .aliases &amp;&amp; git commit <span class="t-fl">-m</span> <span class="t-st">"add project aliases"</span></pre>
          </div>
          <div class="code-panel">
            <div class="ctab"><span>every developer, automatically</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-kw">cd</span> ~/work/myproject
<span class="t-out">project activated — 4 loaded: ci, db, deploy, ti</span>

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

    <!-- 03 Kubernetes Operator -->
    <div class="uc">
      <div class="uc-head">
        <div class="uc-num">03</div>
        <div class="uc-titles">
          <div class="role-badge">Subcommand aliases</div>
          <h3>The Kubernetes Operator</h3>
          <span class="uc-sub">Replace 120 flat aliases with a scheme you actually designed.</span>
        </div>
      </div>
      <div class="uc-body">
        <div class="uc-prose">
          <p>
            Oh-my-zsh ships over 120 kubectl aliases. <code>kgp</code> is get pods. <code>kdp</code>
            is describe pod. <code>kgpw</code> is get pods --watch. The scheme was designed by someone
            else, uses abbreviations you didn't choose, and can't be changed without forking the plugin.
            You either take all 120 or you take none.
          </p>
          <p>
            And the aliases you actually need aren't in there. <code>kubectl logs deployment/api
            --since=10m --tail=100</code> — the one you run every incident — has no alias. So you
            type it in full, or you add <code>klogs-api</code> to your <code>.zshrc</code> and forget
            it exists three weeks later, buried next to the other one-offs.
          </p>
          <p>
            amoxide subcommand aliases let you define the scheme yourself. A base alias routes to a
            command, and short subcommand sequences expand to whatever flags make sense to you.
            <code>k get po</code> becomes <code>kubectl get pods</code>. <code>k logs api</code>
            becomes the exact invocation you always reach for. The whole thing lives in one place,
            is visible with <code>am ls</code>, and is yours to adjust without touching anyone else's
            config.
          </p>
          <p class="uc-note">Subcommand aliases can live in a global profile (always available) or a project <code>.aliases</code> file (only active in that repo). Cluster-specific aliases belong in the project; daily kubectl verbs belong globally.</p>
        </div>
        <div class="code-stack">
          <div class="code-panel">
            <div class="ctab"><span>replace oh-my-zsh kubectl plugin</span></div>
            <pre v-pre><span class="t-cm"># base alias — k routes to kubectl</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k</span> kubectl

<span class="t-cm"># subcommand routing</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k:get:po</span>   <span class="t-st">"get pods"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k:get:svc</span>  <span class="t-st">"get svc"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k:desc:po</span>  <span class="t-st">"describe pod"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k:logs:api</span> <span class="t-st">"logs deployment/api --since=10m --tail=100"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-g</span> <span class="t-hl">k:rr</span>      <span class="t-st">"rollout restart deployment"</span></pre>
          </div>
          <div class="code-panel">
            <div class="ctab"><span>then just type</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-hl">k</span> get po
<span class="t-out">NAME                   READY   STATUS    RESTARTS
api-7d4f9b8c6-xk2pm    1/1     Running   0</span>

<span class="t-pr">❯ </span><span class="t-hl">k</span> logs api
<span class="t-out">2026-04-15T09:12:44Z INFO  server started on :8080</span>

<span class="t-pr">❯ </span><span class="t-hl">k</span> rr api
<span class="t-out">deployment.apps/api restarted</span>

<span class="t-cm"># completions still work — press Tab after k get</span></pre>
          </div>
        </div>
      </div>
    </div>

    <!-- 04 Freelancer -->
    <div class="uc">
      <div class="uc-head">
        <div class="uc-num">04</div>
        <div class="uc-titles">
          <div class="role-badge">Project aliases</div>
          <h3>The Freelancer Between Clients</h3>
          <span class="uc-sub">Each client directory loads its own context. Nothing leaks between them.</span>
        </div>
      </div>
      <div class="uc-body">
        <div class="uc-prose">
          <p>
            You work with three clients. Each has a different cloud provider, a different staging URL,
            a different deployment pipeline, a different VPN. Your <code>.zshrc</code> has three
            commented-out blocks you toggle by hand when you switch context. The wrong
            <code>deploy</code> alias once pointed at the wrong staging environment for ten minutes
            before you noticed.
          </p>
          <p>
            The deeper problem is that global aliases don't belong to anyone. They accumulate — old
            client shortcuts sitting next to current ones, some broken, some shadowing each other.
            Every few months you do a cleanup that takes an afternoon and still doesn't feel finished.
          </p>
          <p>
            With amoxide, each client lives in its own directory. <code>cd client-a/</code> loads
            their aliases automatically — their AWS shortcut, their deploy command, their staging URL
            opener. <code>cd client-b/</code> unloads all of it and loads theirs. Nothing leaks between
            contexts. The <code>.aliases</code> file for each client can live in the project repo —
            versioned, auditable, shared with anyone else who works there.
          </p>
          <p class="uc-note">Project aliases require a one-time <code>am trust</code> per repository — amoxide won't auto-load a file it hasn't seen before.</p>
        </div>
        <div class="code-stack">
          <div class="code-panel">
            <div class="ctab"><span>client-a/.aliases</span></div>
            <pre v-pre><span class="t-cm"># added once, committed to the repo</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">deploy</span>  <span class="t-st">"./scripts/deploy.sh --env staging"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">logs</span>    <span class="t-st">"ssh app@client-a.internal journalctl -u api -f"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">stage</span>   <span class="t-st">"open https://staging.client-a.internal"</span>
<span class="t-pr">❯ </span><span class="t-kw">am</span> add <span class="t-fl">-l</span> <span class="t-hl">tf:plan</span> <span class="t-st">"terraform plan -var-file=client-a.tfvars"</span></pre>
          </div>
          <div class="code-panel">
            <div class="ctab"><span>switching clients — nothing to remember</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-kw">cd</span> ~/clients/client-a
<span class="t-out">client-a activated — 4 loaded: deploy, logs, stage, tf:plan</span>
<span class="t-pr">❯ </span><span class="t-hl">deploy</span>

<span class="t-pr">❯ </span><span class="t-kw">cd</span> ~/clients/client-b
<span class="t-out">client-a deactivated
client-b activated — 3 loaded: infra:plan, preview, ship</span>
<span class="t-pr">❯ </span><span class="t-hl">preview</span>

<span class="t-cm"># no stale aliases. no cross-contamination.</span></pre>
          </div>
        </div>
      </div>
    </div>

    <!-- 05 Dotfile Cleanup -->
    <div class="uc">
      <div class="uc-head">
        <div class="uc-num">05</div>
        <div class="uc-titles">
          <div class="role-badge">Profiles</div>
          <h3>The Great Dotfile Cleanup</h3>
          <span class="uc-sub">Eight years of aliases. One afternoon to make sense of them.</span>
        </div>
      </div>
      <div class="uc-body">
        <div class="uc-prose">
          <p>
            Eight years of development, 200-odd aliases. <code>gcm</code> might be
            <code>git commit -m</code> or <code>git checkout master</code> depending on which year you
            wrote it. Some refer to projects you no longer work on. Some are broken because the tool
            they wrapped was renamed. <code>alias | grep git</code> returns 40 lines and you still
            can't find the one you want.
          </p>
          <p>
            The only way to discover an alias is to remember you made it. The only way to clean them
            up is to scroll through a file you haven't fully read in years. You know this needs doing.
            You keep putting it off.
          </p>
          <p>
            amoxide turns this into a tractable afternoon. Move aliases into named profiles —
            <code>work</code>, <code>personal</code>, <code>git</code>, <code>rust</code>,
            <code>ops</code> — each a plain TOML section you can read, annotate, and understand.
            <code>am ls</code> shows exactly what's active right now. Nothing loads unless you, or a
            project directory, asked for it. The 200-alias pile becomes five profiles you actually
            trust.
          </p>
          <p class="uc-note">Profiles can be activated together: <code>am use git work</code> layers both, with later profiles taking precedence on name conflicts.</p>
        </div>
        <div class="code-stack">
          <div class="code-panel">
            <div class="ctab"><span>migrate from .zshrc</span></div>
            <pre v-pre><span class="t-cm"># create organised profiles</span>
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
            <div class="ctab"><span>what's actually loaded, right now</span></div>
            <pre v-pre><span class="t-pr">❯ </span><span class="t-kw">am use</span> git work
<span class="t-out">git activated — 4 loaded: gl, gm, gp, gs
work activated — 3 loaded: jira, standup, vpn</span>

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
  margin-bottom: 0;
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

.uc-titles {
  flex: 1;
}

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

.uc-prose p {
  color: var(--vp-c-text-2);
  font-size: 15px;
  line-height: 1.8;
  margin-bottom: 14px;
}

.uc-prose p:last-child {
  margin-bottom: 0;
}

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

.ctab span {
  color: var(--vp-c-brand-1);
}

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

/* token colours */
:deep(.t-pr)  { color: var(--vp-c-text-3); user-select: none; }
:deep(.t-kw)  { color: var(--vp-c-brand-1); }
:deep(.t-hl)  { color: var(--vp-c-tip-1, #8be9fd); }
:deep(.t-st)  { color: var(--vp-c-warning-1, #f1fa8c); }
:deep(.t-cm)  { color: var(--vp-c-text-3); }
:deep(.t-fl)  { color: #60a5fa; }
:deep(.t-out) { color: var(--vp-c-text-2); }

@media (max-width: 900px) {
  .uc-body {
    grid-template-columns: 1fr;
  }
  .uc-num {
    font-size: 48px;
    min-width: 64px;
  }
}

@media (max-width: 640px) {
  .uc { padding: 40px 16px; }
  .uc-head { gap: 12px; }
}
</style>
