// Author: Aut520 <aut@autkc.com>
// Repository: https://github.com/Aut520/rmechrevo
// License: GPL-3.0

import { createApp } from 'vue'
import './styles/tokens.css'
import './style.css'
import { initAppearance } from './composables/useAppearance'
import App from './App.vue'

initAppearance()
createApp(App).mount('#app')
