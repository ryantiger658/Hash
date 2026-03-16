import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],

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
