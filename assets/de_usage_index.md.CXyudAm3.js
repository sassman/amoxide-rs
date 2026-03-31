import{_ as a,o as n,c as s,ag as i}from"./chunks/framework.CvgP6Fyv.js";const u=JSON.parse('{"title":"Nutzung","description":"","frontmatter":{},"headers":[],"relativePath":"de/usage/index.md","filePath":"de/usage/index.md"}'),l={name:"de/usage/index.md"};function p(t,e,r,o,c,d){return n(),s("div",null,[...e[0]||(e[0]=[i(`<h1 id="nutzung" tabindex="-1">Nutzung <a class="header-anchor" href="#nutzung" aria-label="Permalink to &quot;Nutzung&quot;">​</a></h1><p>amoxide organisiert Aliase in drei Ebenen, von breitester zu spezifischster:</p><ol><li><strong>Global</strong> — immer aktiv, in jeder Shell-Sitzung verfügbar</li><li><strong>Profile</strong> — benannte Alias-Gruppen, die aktiviert/deaktiviert werden können</li><li><strong>Projekt</strong> — lokale <code>.aliases</code>-Dateien, die sich automatisch pro Verzeichnis laden</li></ol><p>Jede Ebene kann die vorherige überschreiben. Projekt-Aliase überschreiben Profil-Aliase, die wiederum globale Aliase überschreiben.</p><div class="language- vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang"></span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span>🌐 global</span></span>
<span class="line"><span>│ helo → echo hello world global</span></span>
<span class="line"><span>│</span></span>
<span class="line"><span>├─● git (active: 1)</span></span>
<span class="line"><span>│ gm → git commit -S --signoff -m</span></span>
<span class="line"><span>│</span></span>
<span class="line"><span>├─● rust (active: 2)</span></span>
<span class="line"><span>│ ct → cargo test</span></span>
<span class="line"><span>│ cb → cargo build</span></span>
<span class="line"><span>│</span></span>
<span class="line"><span>╰─📁 project aliases (.aliases)</span></span>
<span class="line"><span>  t → ./x.py test</span></span>
<span class="line"><span>  b → ./x.py build</span></span>
<span class="line"><span></span></span>
<span class="line"><span>○ node</span></span>
<span class="line"><span>  nr → npm run</span></span></code></pre></div><ul><li><a href="/de/usage/global.html">Globale Aliase</a> — immer verfügbare Aliase für jede Sitzung</li><li><a href="/de/usage/profiles.html">Profile</a> — benannte Alias-Gruppen verwalten</li><li><a href="/de/usage/project-aliases.html">Projekt-Aliase</a> — verzeichnisbezogene <code>.aliases</code>-Dateien</li></ul>`,6)])])}const h=a(l,[["render",p]]);export{u as __pageData,h as default};
