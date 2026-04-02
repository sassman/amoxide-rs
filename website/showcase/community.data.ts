import fs from 'node:fs'
import path from 'node:path'
import matter from 'gray-matter'

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
  /** Raw TOML content for preview */
  toml: string
  /** README body (markdown, without frontmatter) */
  readme: string
  /** Total alias count across all profiles */
  aliasCount: number
  /** Raw GitHub URL for am import */
  importUrl: string
}

const COMMUNITY_DIR = path.resolve(__dirname, '../../community')
const REPO_BASE = 'https://raw.githubusercontent.com/sassman/amoxide-rs/main/community'

export default {
  watch: ['../../community/**/README.md'],
  load(): CommunityProfile[] {
    const entries: CommunityProfile[] = []

    if (!fs.existsSync(COMMUNITY_DIR)) return entries

    for (const folder of fs.readdirSync(COMMUNITY_DIR)) {
      const folderPath = path.join(COMMUNITY_DIR, folder)
      if (!fs.statSync(folderPath).isDirectory()) continue

      const readmePath = path.join(folderPath, 'README.md')
      const tomlPath = path.join(folderPath, 'profiles.toml')

      if (!fs.existsSync(readmePath) || !fs.existsSync(tomlPath)) continue

      const readmeRaw = fs.readFileSync(readmePath, 'utf-8')
      const { data, content } = matter(readmeRaw)
      const toml = fs.readFileSync(tomlPath, 'utf-8')
      // Count aliases: lines matching `key = "value"` under [profiles.aliases] sections
      const aliasCount = (toml.match(/^\w[\w-]* = /gm) || []).length

      entries.push({
        slug: folder,
        author: data.author || 'unknown',
        description: data.description || '',
        category: data.category || 'misc',
        tags: data.tags || [],
        shell: data.shell || '',
        profiles: data.profiles || [],
        toml,
        aliasCount,
        readme: content.trim(),
        importUrl: `${REPO_BASE}/${folder}/profiles.toml`,
      })
    }

    return entries.sort((a, b) => a.category.localeCompare(b.category) || a.slug.localeCompare(b.slug))
  },
}
