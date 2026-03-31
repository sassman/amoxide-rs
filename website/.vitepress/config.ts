import { defineConfig } from 'vitepress'

const base = process.env.VITEPRESS_BASE || '/'

export default defineConfig({
  base,
  title: 'amoxide',
  description: 'Shell aliases that follow your context — like direnv, but for aliases.',
  head: [
    ['link', { rel: 'icon', href: `${base}logo.svg` }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'amoxide — The right aliases, at the right time' }],
    ['meta', { property: 'og:description', content: 'Like direnv, but for aliases. Define aliases per project, per toolchain, or globally — and load the right ones automatically.' }],
    ['meta', { property: 'og:image', content: 'https://amoxide.rs/og-image.png' }],
    ['meta', { property: 'og:url', content: 'https://amoxide.rs' }],
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    ['meta', { name: 'twitter:title', content: 'amoxide — The right aliases, at the right time' }],
    ['meta', { name: 'twitter:description', content: 'Like direnv, but for aliases. Define aliases per project, per toolchain, or globally.' }],
    ['meta', { name: 'twitter:image', content: 'https://amoxide.rs/og-image.png' }],
    ['script', { async: '', src: 'https://plausible.io/js/pa-a6anBYLf5imqR2fPCnzHy.js' }],
    ['script', {}, 'window.plausible=window.plausible||function(){(plausible.q=plausible.q||[]).push(arguments)},plausible.init=plausible.init||function(i){plausible.o=i||{}};plausible.init()'],
  ],

  locales: {
    root: {
      label: 'English',
      lang: 'en',
      themeConfig: {
        nav: [
          { text: 'Guide', link: '/guide/' },
          { text: 'Usage', link: '/usage/' },
          { text: 'Advanced', link: '/advanced/' },
          { text: 'Showcase', link: '/showcase/' },
          { text: 'FAQ', link: '/faq' },
        ],
        sidebar: {
          '/guide/': [
            {
              text: 'Guide',
              items: [
                { text: 'Getting Started', link: '/guide/' },
                { text: 'Installation', link: '/guide/installation' },
                { text: 'Shell Setup', link: '/guide/setup' },
              ],
            },
          ],
          '/usage/': [
            {
              text: 'Usage',
              items: [
                { text: 'Overview', link: '/usage/' },
                { text: 'Global Aliases', link: '/usage/global' },
                { text: 'Profiles', link: '/usage/profiles' },
                { text: 'Project Aliases', link: '/usage/project-aliases' },
              ],
            },
          ],
          '/advanced/': [
            {
              text: 'Advanced',
              items: [
                { text: 'Overview', link: '/advanced/' },
                { text: 'Parameterized Aliases', link: '/advanced/parameterized-aliases' },
                { text: 'Composing Aliases', link: '/advanced/composing-aliases' },
                { text: 'Config Files', link: '/advanced/config-files' },
              ],
            },
          ],
        },
      },
    },
    de: {
      label: 'Deutsch',
      lang: 'de',
      link: '/de/',
      themeConfig: {
        nav: [
          { text: 'Anleitung', link: '/de/guide/' },
          { text: 'Nutzung', link: '/de/usage/' },
          { text: 'Erweitert', link: '/de/advanced/' },
          { text: 'Showcase', link: '/de/showcase/' },
          { text: 'FAQ', link: '/de/faq' },
        ],
        sidebar: {
          '/de/guide/': [
            {
              text: 'Anleitung',
              items: [
                { text: 'Erste Schritte', link: '/de/guide/' },
                { text: 'Installation', link: '/de/guide/installation' },
                { text: 'Shell-Einrichtung', link: '/de/guide/setup' },
              ],
            },
          ],
          '/de/usage/': [
            {
              text: 'Nutzung',
              items: [
                { text: 'Übersicht', link: '/de/usage/' },
                { text: 'Globale Aliase', link: '/de/usage/global' },
                { text: 'Profile', link: '/de/usage/profiles' },
                { text: 'Projekt-Aliase', link: '/de/usage/project-aliases' },
              ],
            },
          ],
          '/de/advanced/': [
            {
              text: 'Erweitert',
              items: [
                { text: 'Übersicht', link: '/de/advanced/' },
                { text: 'Parametrisierte Aliase', link: '/de/advanced/parameterized-aliases' },
                { text: 'Aliase verketten', link: '/de/advanced/composing-aliases' },
                { text: 'Konfigurationsdateien', link: '/de/advanced/config-files' },
              ],
            },
          ],
        },
      },
    },
  },

  themeConfig: {
    logo: '/logo.svg',
    socialLinks: [
      { icon: 'github', link: 'https://github.com/sassman/amoxide-rs' },
    ],
    search: {
      provider: 'local',
    },
    footer: {
      message: 'Released under the GPLv3 License. <a href="/privacy">Privacy Policy</a>',
      copyright: 'Copyright © 2024-present Sven Kanoldt',
    },
  },
})
