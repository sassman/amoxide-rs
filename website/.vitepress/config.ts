import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'amoxide',
  description: 'Shell aliases that follow your context — like direnv, but for aliases.',
  head: [['link', { rel: 'icon', href: '/logo.svg' }]],

  themeConfig: {
    logo: '/logo.svg',
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
