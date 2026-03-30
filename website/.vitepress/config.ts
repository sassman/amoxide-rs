import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'amoxide',
  description: 'Shell aliases that follow your context — like direnv, but for aliases.',
  head: [['link', { rel: 'icon', href: '/logo.svg' }]],

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
      message: 'Released under the GPLv3 License.',
      copyright: 'Copyright © 2024-present Sven Kanoldt',
    },
  },
})
