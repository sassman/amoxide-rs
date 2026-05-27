import{_ as l,C as p,o,c as i,a3 as r,j as a,a as e,E as t}from"./chunks/framework.CUoSxMxn.js";const b=JSON.parse('{"title":"Usage","description":"","frontmatter":{},"headers":[],"relativePath":"usage/index.md","filePath":"usage/index.md"}'),d={name:"usage/index.md"};function c(g,s,u,m,f,h){const n=p("VersionBadge");return o(),i("div",null,[s[7]||(s[7]=r(`<h1 id="usage" tabindex="-1">Usage <a class="header-anchor" href="#usage" aria-label="Permalink to &quot;Usage&quot;">​</a></h1><p>amoxide organizes aliases in three layers, from broadest to most specific:</p><ol><li><strong>Global</strong> — always active, available in every shell session</li><li><strong>Profiles</strong> — named groups of aliases you can activate/deactivate</li><li><strong>Project</strong> — local <code>.aliases</code> files that auto-load per directory</li></ol><p>Each layer can override the previous one. Project aliases override profile aliases, which override global aliases.</p><p>All three layers also support <strong>subcommand aliases</strong> — short forms for programs that use subcommands (like <code>jj</code>, <code>git</code>, <code>cargo</code>, or <code>kubectl</code>).</p><div class="language- vp-adaptive-theme"><button title="Copy Code" class="copy"></button><span class="lang"></span><pre class="shiki shiki-themes github-light github-dark vp-code" tabindex="0"><code><span class="line"><span>🌐 global</span></span>
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
<span class="line"><span>  ╰─ nr → npm run</span></span></code></pre></div>`,6)),a("ul",null,[s[2]||(s[2]=a("li",null,[a("a",{href:"/_preview/134/usage/global"},"Global Aliases"),e(" — always-on aliases for every session")],-1)),s[3]||(s[3]=a("li",null,[a("a",{href:"/_preview/134/usage/profiles"},"Profiles"),e(" — managing named alias groups")],-1)),s[4]||(s[4]=a("li",null,[a("a",{href:"/_preview/134/usage/project-aliases"},"Project Aliases"),e(" — directory-scoped "),a("code",null,".aliases"),e(" files")],-1)),s[5]||(s[5]=a("li",null,[a("a",{href:"/_preview/134/usage/subcommand-aliases"},"Subcommand Aliases"),e(" — short forms for subcommand-based tools")],-1)),a("li",null,[s[0]||(s[0]=a("a",{href:"/_preview/134/usage/variables"},"Variables",-1)),s[1]||(s[1]=e(" — named placeholders shared across aliases ",-1)),t(n,{v:"0.9.0"})]),s[6]||(s[6]=a("li",null,[a("a",{href:"/_preview/134/usage/sharing"},"Sharing"),e(" — export, import, and share with others")],-1))])])}const y=l(d,[["render",c]]);export{b as __pageData,y as default};
