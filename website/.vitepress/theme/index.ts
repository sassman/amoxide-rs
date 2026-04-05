import DefaultTheme from 'vitepress/theme'
import CommunityGallery from './CommunityGallery.vue'
import VersionBadge from './VersionBadge.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('CommunityGallery', CommunityGallery)
    app.component('VersionBadge', VersionBadge)
  },
}
