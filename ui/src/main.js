import { initTheme } from './lib/theme.js'
import App from './App.svelte'

// Apply stored theme before first render to prevent a flash of wrong colors.
initTheme()

const app = new App({
  target: document.getElementById('app'),
})

export default app
