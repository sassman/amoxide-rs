import{_ as s,o as e,c as n,ah as l}from"./chunks/framework.COsEJIVd.js";const h=JSON.parse('{"title":"Usage","description":"","frontmatter":{},"headers":[],"relativePath":"usage/index.md","filePath":"usage/index.md"}'),i={name:"usage/index.md"};function p(o,a,t,r,c,d){return e(),n("div",null,[...a[0]||(a[0]=[l(`<h1 id="usage" tabindex="-1">Usage <a class="header-anchor" href="#usage" aria-label="Permalink to &quot;Usage&quot;">​</a></h1><p>amoxide organizes aliases in three layers, from broadest to most specific:</p><ol><li><strong>Global</strong> — always active, available in every shell session</li><li><strong>Profiles</strong> — named groups of aliases you can activate/deactivate</li><li><strong>Project</strong> — local <code>.aliases</code> files that auto-load per directory</li></ol><p>Each layer can override the previous one. Project aliases override profile aliases, which override global aliases.</p><div class="language- vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang"></span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span>🌐 global</span></span>
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
<span class="line"><span>  nr → npm run</span></span></code></pre></div><ul><li><a href="/usage/global.html">Global Aliases</a> — always-on aliases for every session</li><li><a href="/usage/profiles.html">Profiles</a> — managing named alias groups</li><li><a href="/usage/project-aliases.html">Project Aliases</a> — directory-scoped <code>.aliases</code> files</li><li><a href="/usage/sharing.html">Sharing</a> — export, import, and share with others</li></ul>`,6)])])}const u=s(i,[["render",p]]);export{h as __pageData,u as default};
