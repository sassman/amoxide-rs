---
layout: home

hero:
  name: "amoxide"
  text: "Shell-Aliase, die deinem Kontext folgen"
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
  - title: Kontextbezogen
    details: Projekt-Aliase werden automatisch geladen, wenn du in ein Verzeichnis wechselst, und entladen, wenn du es verlässt.
    link: /de/usage/project-aliases
    linkText: Mehr erfahren
  - title: Profile
    details: Gruppiere Aliase nach Kontext — Rust, Git, Node. Aktiviere mehrere Profile gleichzeitig mit klarer Priorität.
    link: /de/usage/profiles
    linkText: Mehr erfahren
  - title: Parametrisierte Aliase
    details: Verwende Positions- und Sammel-Argument-Templates, um leistungsstarke, wiederverwendbare Aliase zu erstellen.
    link: /de/advanced/parameterized-aliases
    linkText: Mehr erfahren
  - title: Interaktives TUI
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
```

```powershell [PowerShell]
irm https://github.com/sassman/amoxide-rs/releases/latest/download/amoxide-installer.ps1 | iex
```

```sh [Cargo (pre-built)]
cargo binstall amoxide amoxide-tui
```

```sh [Cargo (source)]
cargo install amoxide amoxide-tui
```

:::

## In Aktion

### Interaktives TUI

Durchsuche, erstelle, verschiebe und lösche Aliase visuell mit `am tui`:

![am tui Screenshot](/am-tui.png)

### CLI-Auflistung

Sieh deine geschichtete Alias-Hierarchie auf einen Blick mit `am ls`:

![am ls Screenshot](/am-ls.png)
