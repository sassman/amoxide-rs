---
layout: home

hero:
  name: "amoxide"
  text: "Die richtigen Aliase,\nzur richtigen Zeit"
  tagline: Wie direnv, aber für Aliase. Definiere Aliase pro Projekt, pro Toolchain oder global — sie werden automatisch geladen.
  image:
    src: /logo.svg
    alt: amoxide Logo
  actions:
    - theme: brand
      text: Erste Schritte
      link: /de/guide/
    - theme: alt
      text: GitHub
      link: https://github.com/sassman/amoxide-rs

features:
  - icon: 📂
    title: Kontextbezogen
    details: Projekt-Aliase werden automatisch geladen, wenn du in ein Verzeichnis wechselst, und entladen, wenn du es verlässt.
    link: /de/usage/project-aliases
    linkText: Mehr erfahren
  - icon: 📦
    title: Profile
    details: Gruppiere Aliase nach Kontext — Rust, Git, Node. Aktiviere mehrere Profile gleichzeitig mit klarer Priorität.
    link: /de/usage/profiles
    linkText: Mehr erfahren
  - icon: 🔧
    title: Parametrisierte Aliase
    details: Verwende Positions- und Sammel-Argument-Templates, um leistungsstarke, wiederverwendbare Aliase zu erstellen.
    link: /de/advanced/parameterized-aliases
    linkText: Mehr erfahren
  - icon: 🖥️
    title: Interaktives TUI
    details: Durchsuche, erstelle, verschiebe und verwalte Aliase visuell mit dem am-tui Companion.
    link: /de/guide/installation
    linkText: am-tui installieren
---

## Installation

::: code-group

```sh [Homebrew]
brew install sassman/tap/amoxide sassman/tap/amoxide-tui
```

```sh [Shell-Skript]
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.sh | sh
curl -fsSL https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.sh | sh
```

```powershell [PowerShell]
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.ps1 | iex"
powershell -ExecutionPolicy Bypass -c "irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-tui-installer.ps1 | iex"
```

```sh [Cargo (pre-built)]
cargo binstall amoxide amoxide-tui
```

```sh [Cargo]
cargo install amoxide amoxide-tui
```

:::

<div class="before-after">

## Warum amoxide?

<div class="comparison">
<div class="before">

**Vorher:**
```sh
cargo clippy --locked --all-targets -- -D warnings
```

</div>
<div class="after">

**Nachher:**
```sh
l
```

</div>
</div>

Starte mit Projekt-Aliasen. Refaktoriere in Profile. [Erste Schritte →](/de/guide/)

</div>

## In Aktion

<div class="screenshots">

<figure>

![am tui Screenshot](/am-tui.png)

<figcaption>Interaktives TUI — <code>am tui</code></figcaption>
</figure>

<figure>

![am ls Screenshot](/am-ls.png)

<figcaption>CLI-Auflistung — <code>am ls</code></figcaption>
</figure>

</div>
