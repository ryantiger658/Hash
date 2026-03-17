// @ts-check
import { test as base } from '@playwright/test'

const API_KEY = process.env.E2E_API_KEY ?? 'test-key'
const API_KEY_STORAGE = 'hash-api-key'

/**
 * Authenticated test fixture — seeds localStorage with the API key so every
 * test starts already logged in and on the main editor view.
 */
export const test = base.extend({
  page: async ({ page }, use) => {
    // Inject API key before any navigation so the app boots logged in.
    await page.addInitScript(
      ({ key, value }) => localStorage.setItem(key, value),
      { key: API_KEY_STORAGE, value: API_KEY },
    )
    await use(page)
  },
})

export { expect } from '@playwright/test'
