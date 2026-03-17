import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { VitePWA } from 'vite-plugin-pwa'

export default defineConfig({
  plugins: [
    svelte(),
    VitePWA({
      registerType: 'autoUpdate',
      injectRegister: 'auto',
      // Only generate the service worker for the server build target.
      // In dev (Tauri/desktop) builds the SW is not needed.
      disable: process.env.BUILD_TARGET !== 'server',
      manifest: {
        name: '#ash',
        short_name: '#ash',
        description: 'Self-hosted, offline-first markdown knowledge base',
        theme_color: '#000000',
        background_color: '#000000',
        display: 'standalone',
        start_url: '/',
        icons: [
          {
            src: '/android-chrome-192x192.png',
            sizes: '192x192',
            type: 'image/png',
          },
          {
            src: '/android-chrome-512x512.png',
            sizes: '512x512',
            type: 'image/png',
            purpose: 'any maskable',
          },
          {
            src: '/apple-touch-icon.png',
            sizes: '180x180',
            type: 'image/png',
          },
        ],
      },
      workbox: {
        // Cache the app shell (HTML, JS, CSS, fonts) for offline load.
        // Vault content is NOT cached — it always comes from the server.
        globPatterns: ['**/*.{js,css,html,ico,png,woff,woff2}'],
        // Never cache API calls — vault data must always be fresh.
        runtimeCaching: [],
      },
    }),
  ],

  // When building for the server, output to server/static so Axum can serve it.
  // When building for the desktop, Tauri picks up the dist/ directory.
  build: {
    outDir: process.env.BUILD_TARGET === 'server' ? '../server/static' : 'dist',
    emptyOutDir: true,
  },

  server: {
    port: 5173,
    // Proxy API calls to the Rust server during development.
    proxy: {
      '/api': {
        target: 'http://localhost:3535',
        changeOrigin: true,
      },
    },
  },
})
