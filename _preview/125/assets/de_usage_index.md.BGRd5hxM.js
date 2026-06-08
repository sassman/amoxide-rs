import{_ as p,C as r,o as t,c as o,a3 as a,j as n,a as i,E as l}from"./chunks/framework.CMLpj6w5.js";const _=JSON.parse('{"title":"Nutzung","description":"","frontmatter":{},"headers":[],"relativePath":"de/usage/index.md","filePath":"de/usage/index.md"}'),d={name:"de/usage/index.md"};function c(u,e,g,m,b,h){const s=r("VersionBadge");return t(),o("div",null,[e[5]||(e[5]=a(`<h1 id="nutzung" tabindex="-1">Nutzung <a class="header-anchor" href="#nutzung" aria-label="Permalink to &quot;Nutzung&quot;">​</a></h1><p>amoxide organisiert Aliase in drei Ebenen, von breitester zu spezifischster:</p><ol><li><strong>Global</strong> — immer aktiv, in jeder Shell-Sitzung verfügbar</li><li><strong>Profile</strong> — benannte Alias-Gruppen, die aktiviert/deaktiviert werden können</li><li><strong>Projekt</strong> — lokale <code>.aliases</code>-Dateien, die sich automatisch pro Verzeichnis laden</li></ol><p>Jede Ebene kann die vorherige überschreiben. Projekt-Aliase überschreiben Profil-Aliase, die wiederum globale Aliase überschreiben.</p><p>Alle drei Ebenen unterstützen auch <strong>Subcommand-Aliase</strong> — Kurzformen für Programme, die Subcommands verwenden (wie <code>jj</code>, <code>git</code>, <code>cargo</code> oder <code>kubectl</code>).</p><div class="language- vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang"></span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span>🌐 global</span></span>
<span class="line"><span>│  ╰─ ll → ls -lha</span></span>
<span class="line"><span>│</span></span>
<span class="line"><span>├─● rust (active: 1)</span></span>
<span class="line"><span>│   ├─ i → cargo install --path .</span></span>
<span class="line"><span>│   ├─ l → cargo clippy --locked --all-targets -- -D warnings</span></span>
<span class="line"><span>│   ╰─ t → cargo test --all-features</span></span>
<span class="line"><span>│</span></span>
<span class="line"><span>├─● git (active: 2)</span></span>
<span class="line"><span>│   ├─ gm → git commit -S --signoff -m</span></span>
<span class="line"><span>│   ╰─◆ git (subcommands)</span></span>
<span class="line"><span>│     ├─ psh → push</span></span>
<span class="line"><span>│     ╰─ st → status --short</span></span>
<span class="line"><span>│</span></span>
<span class="line"><span>╰─📁 project (~/path/to/project/.aliases)</span></span>
<span class="line"><span>  ├─ b → ./x.py build</span></span>
<span class="line"><span>  ╰─ t → ./x.py test</span></span>
<span class="line"><span></span></span>
<span class="line"><span>○ node</span></span>
<span class="line"><span>  ╰─ nr → npm run</span></span></code></pre></div>`,6)),n("ul",null,[e[4]||(e[4]=a('<li><a href="/_preview/125/de/usage/global">Globale Aliase</a> — immer verfügbare Aliase für jede Sitzung</li><li><a href="/_preview/125/de/usage/profiles">Profile</a> — benannte Alias-Gruppen verwalten</li><li><a href="/_preview/125/de/usage/project-aliases">Projekt-Aliase</a> — verzeichnisbezogene <code>.aliases</code>-Dateien</li><li><a href="/_preview/125/de/usage/subcommand-aliases">Subcommand-Aliase</a> — Kurzformen für subcommandbasierte Tools</li><li><a href="/_preview/125/de/usage/sharing">Teilen</a> — Aliase exportieren, importieren und teilen</li>',5)),n("li",null,[e[0]||(e[0]=n("code",null,"am ls -d",-1)),e[1]||(e[1]=i(" zeigt die Beschreibungsspalte, wenn Beschreibungen vorhanden sind ",-1)),l(s,{v:"0.10.0"})]),n("li",null,[e[2]||(e[2]=n("code",null,"am la",-1)),e[3]||(e[3]=i(" zeigt immer Beschreibungen für den aktiven Bereich ",-1)),l(s,{v:"0.10.0"})])])])}const v=p(d,[["render",c]]);export{_ as __pageData,v as default};
