import DefaultTheme from 'vitepress/theme'
import CommunityGallery from './CommunityGallery.vue'
import './custom.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('CommunityGallery', CommunityGallery)
  },
}
