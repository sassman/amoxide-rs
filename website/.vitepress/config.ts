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
          { text: 'Config', link: '/config/' },
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
          '/config/': [
            {
              text: 'Configuration',
              items: [
                { text: 'Overview', link: '/config/' },
                { text: 'Profiles', link: '/config/profiles' },
                { text: 'Project Aliases', link: '/config/project-aliases' },
              ],
            },
          ],
          '/advanced/': [
            {
              text: 'Advanced',
              items: [
                { text: 'Overview', link: '/advanced/' },
                { text: 'Parameterized Aliases', link: '/advanced/parameterized-aliases' },
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
          { text: 'Konfiguration', link: '/de/config/' },
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
          '/de/config/': [
            {
              text: 'Konfiguration',
              items: [
                { text: 'Übersicht', link: '/de/config/' },
                { text: 'Profile', link: '/de/config/profiles' },
                { text: 'Projekt-Aliase', link: '/de/config/project-aliases' },
              ],
            },
          ],
          '/de/advanced/': [
            {
              text: 'Erweitert',
              items: [
                { text: 'Übersicht', link: '/de/advanced/' },
                { text: 'Parametrisierte Aliase', link: '/de/advanced/parameterized-aliases' },
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
