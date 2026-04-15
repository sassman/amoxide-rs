import { defineConfig } from 'vitepress'
import fs from 'node:fs'
import path from 'node:path'
import matter from 'gray-matter'

const base = process.env.VITEPRESS_BASE || '/'

let appVersion = '0.0.0'
try {
  const res = await fetch('https://crates.io/api/v1/crates/amoxide', {
    headers: { 'User-Agent': 'amoxide-website/1.0 (https://amoxide.rs)' },
  })
  const data = await res.json() as any
  appVersion = data.crate.newest_version ?? appVersion
} catch {
  const cargoToml = fs.readFileSync(path.resolve(__dirname, '../../Cargo.toml'), 'utf-8')
  appVersion = cargoToml.match(/^version = "(.+)"/m)?.[1] ?? appVersion
}

// Build showcase sidebar from community folder
function buildShowcaseSidebar() {
  const communityDir = path.resolve(__dirname, '../../community')
  const tags = new Set<string>()
  const authors = new Set<string>()
  const names: { slug: string; label: string }[] = []

  if (fs.existsSync(communityDir)) {
    for (const folder of fs.readdirSync(communityDir)) {
      if (folder === 'TEMPLATE') continue
      const readmePath = path.join(communityDir, folder, 'README.md')
      if (!fs.existsSync(readmePath)) continue
      const { data } = matter(fs.readFileSync(readmePath, 'utf-8'))
      if (data.tags) data.tags.forEach((t: string) => tags.add(t))
      if (data.author) authors.add(data.author)
      names.push({ slug: folder, label: folder.replace(/^[^-]+-/, '') })
    }
  }

  const COLLAPSE_THRESHOLD = 10

  return [
    {
      text: 'Showcase',
      items: [
        { text: `All Profiles (${names.length})`, link: '/showcase/' },
        { text: 'Contribute', link: '/showcase/contribute' },
      ],
    },
    {
      text: 'By Tag',
      collapsed: false,
      items: Array.from(tags).sort().map(tag => ({
        text: tag,
        link: `/showcase/#tag=${tag}`,
      })),
    },
    {
      text: 'By Author',
      collapsed: authors.size > COLLAPSE_THRESHOLD,
      items: Array.from(authors).sort().map(author => ({
        text: author,
        link: `/showcase/#author=${author}`,
      })),
    },
    {
      text: 'By Name',
      collapsed: names.length > COLLAPSE_THRESHOLD,
      items: names.sort((a, b) => a.label.localeCompare(b.label)).map(n => ({
        text: n.label,
        link: `/showcase/#name=${n.slug}`,
      })),
    },
  ]
}

const jsonLd = JSON.stringify({
  '@context': 'https://schema.org',
  '@type': 'SoftwareApplication',
  'name': 'amoxide',
  'applicationCategory': 'DeveloperApplication',
  'operatingSystem': 'Linux, macOS, Windows',
  'description': 'Shell alias manager for developers. Define context-aware aliases per project, toolchain, or globally. Like direnv, but for shell aliases.',
  'url': 'https://amoxide.rs',
  'downloadUrl': 'https://github.com/sassman/amoxide-rs/releases',
  'offers': { '@type': 'Offer', 'price': '0', 'priceCurrency': 'USD' },
  'author': { '@type': 'Person', 'name': 'Sven Kanoldt', 'url': 'https://d34dl0ck.me/' },
  'license': 'https://www.gnu.org/licenses/gpl-3.0.html',
  'codeRepository': 'https://github.com/sassman/amoxide-rs',
  'programmingLanguage': 'Rust',
})

export default defineConfig({
  base,
  title: 'amoxide — Shell Alias Manager',
  vite: {
    define: {
      __APP_VERSION__: JSON.stringify(appVersion),
    },
  },
  description: 'amoxide is a shell alias manager for developers. Define context-aware aliases per project, toolchain, or globally — and load the right ones automatically. Like direnv, but for aliases.',
  cleanUrls: true,
  sitemap: {
    hostname: 'https://amoxide.rs',
  },
  transformHead({ pageData }) {
    const canonicalUrl = `https://amoxide.rs/${pageData.relativePath}`
      .replace(/index\.md$/, '')
      .replace(/\.md$/, '')
    const tags: [string, Record<string, string>][] = [
      ['link', { rel: 'canonical', href: canonicalUrl }],
      ['meta', { property: 'og:url', content: canonicalUrl }],
    ]
    if (pageData.relativePath.startsWith('de/')) {
      // German page — point hreflang back to the English equivalent
      const enUrl = `https://amoxide.rs/${pageData.relativePath.replace(/^de\//, '')}`.replace(/index\.md$/, '').replace(/\.md$/, '')
      tags.push(['link', { rel: 'alternate', hreflang: 'de', href: canonicalUrl }])
      tags.push(['link', { rel: 'alternate', hreflang: 'en', href: enUrl }])
      tags.push(['link', { rel: 'alternate', hreflang: 'x-default', href: enUrl }])
    } else {
      // English page — point hreflang to the German equivalent
      const deUrl = `https://amoxide.rs/de/${pageData.relativePath}`.replace(/index\.md$/, '').replace(/\.md$/, '')
      tags.push(['link', { rel: 'alternate', hreflang: 'en', href: canonicalUrl }])
      tags.push(['link', { rel: 'alternate', hreflang: 'de', href: deUrl }])
      tags.push(['link', { rel: 'alternate', hreflang: 'x-default', href: canonicalUrl }])
    }
    return tags
  },
  head: [
    ['link', { rel: 'icon', href: `${base}logo.svg` }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:site_name', content: 'amoxide' }],
    ['meta', { property: 'og:title', content: 'amoxide — The right aliases, at the right time' }],
    ['meta', { property: 'og:description', content: 'Like direnv, but for aliases. Define aliases per project, per toolchain, or globally — and load the right ones automatically.' }],
    ['meta', { property: 'og:image', content: 'https://amoxide.rs/og-image.png' }],
    ['meta', { name: 'twitter:card', content: 'summary_large_image' }],
    // og:url is injected per-page via transformHead
    ['meta', { name: 'twitter:title', content: 'amoxide — The right aliases, at the right time' }],
    ['meta', { name: 'twitter:description', content: 'Like direnv, but for aliases. Define aliases per project, per toolchain, or globally.' }],
    ['meta', { name: 'twitter:image', content: 'https://amoxide.rs/og-image.png' }],
    ['script', { type: 'application/ld+json' }, jsonLd],
    ['script', { async: '', src: 'https://plausible.io/js/pa-a6anBYLf5imqR2fPCnzHy.js' }],
    ['script', {}, 'window.plausible=window.plausible||function(){(plausible.q=plausible.q||[]).push(arguments)},plausible.init=plausible.init||function(i){plausible.o=i||{}};plausible.init()'],
  ],

  locales: {
    root: {
      label: 'English',
      lang: 'en',
      themeConfig: {
        siteTitle: 'amoxide',
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
                { text: 'Subcommand Aliases', link: '/usage/subcommand-aliases' },
                { text: 'Sharing', link: '/usage/sharing' },
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
          '/showcase/': buildShowcaseSidebar(),
        },
      },
    },
    de: {
      label: 'Deutsch',
      lang: 'de',
      link: '/de/',
      themeConfig: {
        siteTitle: 'amoxide',
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
                { text: 'Subcommand-Aliase', link: '/de/usage/subcommand-aliases' },
                { text: 'Teilen', link: '/de/usage/sharing' },
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
    siteTitle: 'amoxide',
    logo: '/logo.svg',
    logoAlt: 'amoxide logo',
    socialLinks: [
      { icon: 'github', link: 'https://github.com/sassman/amoxide-rs' },
    ],
    search: {
      provider: 'local',
    },
    footer: {
      message: 'Released under the GPLv3 License. <a href="/privacy">Privacy Policy</a>',
      copyright: 'Copyright © 2024-present <a href="https://d34dl0ck.me/" target="_blank" rel="noopener noreferrer">Sven Kanoldt</a>',
    },
  },
})
