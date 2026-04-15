import { h } from 'vue'
import DefaultTheme from 'vitepress/theme'
import CommunityGallery from './CommunityGallery.vue'
import VersionBadge from './VersionBadge.vue'
import NavTitleVersion from './NavTitleVersion.vue'
import WhySection from './WhySection.vue'
import UseCases from './UseCases.vue'
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
    app.component('WhySection', WhySection)
    app.component('UseCases', UseCases)
  },
}
