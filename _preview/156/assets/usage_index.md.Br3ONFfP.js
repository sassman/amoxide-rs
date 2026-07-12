import{_ as o,C as p,o as i,c as r,a3 as t,j as a,a as n,E as l}from"./chunks/framework.CEcKgAl8.js";const b=JSON.parse('{"title":"Usage","description":"","frontmatter":{},"headers":[],"relativePath":"usage/index.md","filePath":"usage/index.md"}'),d={name:"usage/index.md"};function c(u,s,g,m,f,v){const e=p("VersionBadge");return i(),r("div",null,[s[11]||(s[11]=t(`<h1 id="usage" tabindex="-1">Usage <a class="header-anchor" href="#usage" aria-label="Permalink to &quot;Usage&quot;">​</a></h1><p>amoxide organizes aliases in three layers, from broadest to most specific:</p><ol><li><strong>Global</strong> — always active, available in every shell session</li><li><strong>Profiles</strong> — named groups of aliases you can activate/deactivate</li><li><strong>Project</strong> — local <code>.aliases</code> files that auto-load per directory</li></ol><p>Each layer can override the previous one. Project aliases override profile aliases, which override global aliases.</p><p>All three layers also support <strong>subcommand aliases</strong> — short forms for programs that use subcommands (like <code>jj</code>, <code>git</code>, <code>cargo</code>, or <code>kubectl</code>).</p><div class="language- vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang"></span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span>🌐 global</span></span>
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
<span class="line"><span>  ╰─ nr → npm run</span></span></code></pre></div>`,6)),a("ul",null,[s[6]||(s[6]=a("li",null,[a("a",{href:"/_preview/156/usage/global"},"Global Aliases"),n(" — always-on aliases for every session")],-1)),s[7]||(s[7]=a("li",null,[a("a",{href:"/_preview/156/usage/profiles"},"Profiles"),n(" — managing named alias groups")],-1)),s[8]||(s[8]=a("li",null,[a("a",{href:"/_preview/156/usage/project-aliases"},"Project Aliases"),n(" — directory-scoped "),a("code",null,".aliases"),n(" files")],-1)),s[9]||(s[9]=a("li",null,[a("a",{href:"/_preview/156/usage/subcommand-aliases"},"Subcommand Aliases"),n(" — short forms for subcommand-based tools")],-1)),a("li",null,[s[0]||(s[0]=a("a",{href:"/_preview/156/usage/variables"},"Variables",-1)),s[1]||(s[1]=n(" — named placeholders shared across aliases ",-1)),l(e,{v:"0.9.0"})]),s[10]||(s[10]=a("li",null,[a("a",{href:"/_preview/156/usage/sharing"},"Sharing"),n(" — export, import, and share with others")],-1)),a("li",null,[s[2]||(s[2]=a("code",null,"am ls -d",-1)),s[3]||(s[3]=n(" shows the description column when descriptions are present ",-1)),l(e,{v:"0.10.0"})]),a("li",null,[s[4]||(s[4]=a("code",null,"am la",-1)),s[5]||(s[5]=n(" always shows descriptions for the active set ",-1)),l(e,{v:"0.10.0"})])])])}const y=o(d,[["render",c]]);export{b as __pageData,y as default};
