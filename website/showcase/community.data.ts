import fs from 'node:fs'
import path from 'node:path'
import matter from 'gray-matter'
import MarkdownIt from 'markdown-it'

const md = new MarkdownIt({ html: true, linkify: true })

export interface CommunityProfile {
  /** Folder name, e.g. "sassman-git-conventional" */
  slug: string
  /** README frontmatter fields */
  author: string
  description: string
  category: string
  tags: string[]
  shell: string
  profiles: string[]
  /** Total alias count across all profiles */
  aliasCount: number
  /** Raw GitHub URL for am import */
  importUrl: string
}

/** Detail data loaded on demand when a tile is expanded */
export interface CommunityProfileDetail {
  toml: string
  /** Pre-rendered HTML from the README markdown */
  readmeHtml: string
}

const COMMUNITY_DIR = path.resolve(__dirname, '../../community')
const REPO_BASE = 'https://raw.githubusercontent.com/sassman/amoxide-rs/main/community'
const PUBLIC_DATA_DIR = path.resolve(__dirname, '../public/showcase/data')

export default {
  watch: ['../../community/**/README.md'],
  load(): CommunityProfile[] {
    const entries: CommunityProfile[] = []

    if (!fs.existsSync(COMMUNITY_DIR)) return entries

    // Ensure the public data directory exists for detail JSON files
    fs.mkdirSync(PUBLIC_DATA_DIR, { recursive: true })

    for (const folder of fs.readdirSync(COMMUNITY_DIR)) {
      const folderPath = path.join(COMMUNITY_DIR, folder)
      if (!fs.statSync(folderPath).isDirectory()) continue
      if (folder === 'TEMPLATE') continue

      const readmePath = path.join(folderPath, 'README.md')
      const tomlPath = path.join(folderPath, 'profiles.toml')

      if (!fs.existsSync(readmePath) || !fs.existsSync(tomlPath)) continue

      const readmeRaw = fs.readFileSync(readmePath, 'utf-8')
      const { data, content } = matter(readmeRaw)
      const toml = fs.readFileSync(tomlPath, 'utf-8')
      // Count aliases: lines matching `key = "value"` under [profiles.aliases] sections
      const aliasCount = (toml.match(/^\w[\w-]* = /gm) || []).length

      // Write detail JSON for lazy loading (README pre-rendered to HTML)
      const detail: CommunityProfileDetail = {
        toml,
        readmeHtml: md.render(content.trim()),
      }
      fs.writeFileSync(
        path.join(PUBLIC_DATA_DIR, `${folder}.json`),
        JSON.stringify(detail),
      )

      // Gallery only gets lightweight metadata
      entries.push({
        slug: folder,
        author: data.author || 'unknown',
        description: data.description || '',
        category: data.category || 'misc',
        tags: data.tags || [],
        shell: data.shell || '',
        profiles: data.profiles || [],
        aliasCount,
        importUrl: `${REPO_BASE}/${folder}/profiles.toml`,
      })
    }

    return entries.sort((a, b) => a.category.localeCompare(b.category) || a.slug.localeCompare(b.slug))
  },
}
