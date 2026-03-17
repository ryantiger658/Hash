// @ts-check
import { test, expect } from '@playwright/test'

const API_KEY = process.env.E2E_API_KEY ?? 'test-key'
const API_KEY_STORAGE = 'hash-api-key'

test.describe('Authentication', () => {
  test('shows login screen when not authenticated', async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('#apikey')).toBeVisible()
    await expect(page.getByRole('button', { name: 'Connect' })).toBeVisible()
  })

  test('wrong API key shows an error', async ({ page }) => {
    await page.goto('/')
    await page.fill('#apikey', 'wrong-key')
    await page.getByRole('button', { name: 'Connect' }).click()
    await expect(page.locator('.error')).toBeVisible()
    // Should not advance to the editor
    await expect(page.locator('#apikey')).toBeVisible()
  })

  test('correct API key logs in and shows the editor', async ({ page }) => {
    await page.goto('/')
    await page.fill('#apikey', API_KEY)
    await page.getByRole('button', { name: 'Connect' }).click()
    // After login the sidebar file tree should appear
    await expect(page.locator('.sidebar')).toBeVisible({ timeout: 8_000 })
  })

  test('API key is remembered after page reload', async ({ page }) => {
    await page.goto('/')
    await page.fill('#apikey', API_KEY)
    await page.getByRole('button', { name: 'Connect' }).click()
    await expect(page.locator('.sidebar')).toBeVisible({ timeout: 8_000 })

    await page.reload()
    // Should skip login and show the editor directly
    await expect(page.locator('.sidebar')).toBeVisible({ timeout: 8_000 })
    await expect(page.locator('#apikey')).not.toBeVisible()
  })

  test('logout clears key and returns to login screen', async ({ page }) => {
    // Start logged in via localStorage
    await page.addInitScript(
      ({ key, value }) => localStorage.setItem(key, value),
      { key: API_KEY_STORAGE, value: API_KEY },
    )
    await page.goto('/')
    await expect(page.locator('.sidebar')).toBeVisible({ timeout: 8_000 })

    // Click the logout button in the sidebar footer
    await page.locator('button[title="Logout"]').click()
    await expect(page.locator('#apikey')).toBeVisible({ timeout: 5_000 })
  })

  test('after logout, reloading keeps the user logged out', async ({ page }) => {
    // Log in via form (not addInitScript — that re-seeds on reload)
    await page.goto('/')
    await page.fill('#apikey', API_KEY)
    await page.getByRole('button', { name: 'Connect' }).click()
    await expect(page.locator('.sidebar')).toBeVisible({ timeout: 8_000 })

    await page.locator('button[title="Logout"]').click()
    await expect(page.locator('#apikey')).toBeVisible({ timeout: 5_000 })

    await page.reload()
    await expect(page.locator('#apikey')).toBeVisible()
  })
})
