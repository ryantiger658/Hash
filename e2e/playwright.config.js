// @ts-check
import { defineConfig, devices } from '@playwright/test'

const BASE_URL = process.env.E2E_BASE_URL ?? 'http://localhost:3535'

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,   // tests share a vault; run sequentially to avoid races
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? 'github' : 'list',
  timeout: 15_000,
  expect: { timeout: 5_000 },

  use: {
    baseURL: BASE_URL,
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // In CI the server binary is started externally before the tests run.
  // Locally, point E2E_BASE_URL at your running dev server or use `make e2e`.
})
