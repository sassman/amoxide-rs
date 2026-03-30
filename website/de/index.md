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
  - title: Profile
    details: Gruppiere Aliase nach Kontext — Rust, Git, Node. Aktiviere mehrere Profile gleichzeitig mit klarer Priorität.
  - title: Parametrisierte Aliase
    details: Verwende Positions- und Sammel-Argument-Templates, um leistungsstarke, wiederverwendbare Aliase zu erstellen.
  - title: Interaktives TUI
    details: Durchsuche, erstelle, verschiebe und verwalte Aliase visuell mit dem am-tui Companion.
---

## In Aktion

### Interaktives TUI

Durchsuche, erstelle, verschiebe und lösche Aliase visuell mit `am tui`:

![am tui Screenshot](/am-tui.png)

### CLI-Auflistung

Sieh deine geschichtete Alias-Hierarchie auf einen Blick mit `am ls`:

![am ls Screenshot](/am-ls.png)
