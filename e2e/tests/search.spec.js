// @ts-check
import { test, expect } from '../fixtures.js'

test.describe('Search', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 12_000 })

    // Create a note with known searchable content
    await page.keyboard.press('Control+n')
    const modal = page.getByRole('dialog', { name: 'New note' })
    await modal.locator('input[placeholder="note.md"]').fill(`e2e-search-${Date.now()}.md`)
    await modal.getByRole('button', { name: 'Create' }).click()
    await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 8_000 })
    // Wait for the new empty file to fully load before filling
    await expect(page.locator('textarea.raw')).toHaveValue('', { timeout: 5_000 })
    await page.locator('textarea.raw').fill('uniquesearchterm xyz content')
    await page.keyboard.press('Control+s')
    await page.waitForTimeout(500)
  })

  async function openSearch(page) {
    await page.locator('input[type="search"]').click()
  }

  test('search field is present in the sidebar', async ({ page }) => {
    await expect(page.locator('input[type="search"]')).toBeVisible()
  })

  test('typing a query returns results', async ({ page }) => {
    await openSearch(page)
    await page.locator('input[type="search"]').fill('uniquesearchterm')
    await expect(page.locator('.result-path').first()).toBeVisible({ timeout: 3_000 })
  })

  test('each result shows a file path and snippet', async ({ page }) => {
    await openSearch(page)
    await page.locator('input[type="search"]').fill('uniquesearchterm')
    const firstResult = page.locator('ul.results li').first()
    await expect(firstResult.locator('.result-path')).toBeVisible({ timeout: 3_000 })
    await expect(firstResult.locator('.result-snippet')).toBeVisible()
  })

  test('clicking a result opens the file', async ({ page }) => {
    await openSearch(page)
    await page.locator('input[type="search"]').fill('uniquesearchterm')
    const firstResult = page.locator('ul.results li').first()
    await firstResult.click()
    await expect(page.locator('textarea.raw')).toBeVisible()
    const value = await page.locator('textarea.raw').inputValue()
    expect(value).toContain('uniquesearchterm')
  })

  test('empty query returns no results', async ({ page }) => {
    await openSearch(page)
    await page.locator('input[type="search"]').fill('')
    await expect(page.locator('ul.results li')).toHaveCount(0)
  })

  test('query with no matches shows an empty results list', async ({ page }) => {
    await openSearch(page)
    await page.locator('input[type="search"]').fill('zzznomatchxyz99999')
    await page.waitForTimeout(500)
    await expect(page.locator('ul.results li')).toHaveCount(0)
  })

  test('search is case-insensitive', async ({ page }) => {
    await openSearch(page)
    await page.locator('input[type="search"]').fill('UNIQUESEARCHTERM')
    await expect(page.locator('.result-path').first()).toBeVisible({ timeout: 3_000 })
  })
})
