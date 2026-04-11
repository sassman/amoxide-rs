import { h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import CommunityGallery from './CommunityGallery.vue'
import VersionBadge from './VersionBadge.vue'
import NavTitleVersion from './NavTitleVersion.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  Layout() {
    return h(DefaultTheme.Layout, null, {
      'nav-bar-title-after': () => h(NavTitleVersion),
    })
  },
  enhanceApp({ app }) {
    app.component('CommunityGallery', CommunityGallery)
    app.component('VersionBadge', VersionBadge)
  },
}
