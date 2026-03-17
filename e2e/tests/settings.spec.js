// @ts-check
import { test, expect } from '../fixtures.js'

test.describe('Settings panel', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 12_000 })
  })

  function openSettings(page) {
    return page.keyboard.press('Control+,')
  }

  test('Ctrl+, opens the settings panel', async ({ page }) => {
    await openSettings(page)
    await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible()
  })

  test('gear icon opens the settings panel', async ({ page }) => {
    await page.locator('button[title="Settings"]').click()
    await expect(page.getByRole('dialog', { name: 'Settings' })).toBeVisible()
  })

  test('Escape closes the panel', async ({ page }) => {
    await openSettings(page)
    const panel = page.getByRole('dialog', { name: 'Settings' })
    await expect(panel).toBeVisible()
    await page.keyboard.press('Escape')
    await expect(panel).not.toBeVisible()
  })

  test('clicking the backdrop closes the panel', async ({ page }) => {
    await openSettings(page)
    const panel = page.getByRole('dialog', { name: 'Settings' })
    await expect(panel).toBeVisible()
    // Click on the backdrop (outside the dialog)
    await page.locator('.backdrop').click({ position: { x: 5, y: 5 } })
    await expect(panel).not.toBeVisible()
  })

  test('accent color picker is visible', async ({ page }) => {
    await openSettings(page)
    await expect(page.locator('input.color-swatch')).toBeVisible()
    await expect(page.locator('input.color-text')).toBeVisible()
  })

  test('theme radio buttons are visible and one is selected', async ({ page }) => {
    await openSettings(page)
    const radios = page.locator('.radio-opt')
    await expect(radios).toHaveCount(3)
    await expect(page.locator('.radio-opt.selected')).toHaveCount(1)
  })

  test('toggling line numbers on shows the gutter in the editor', async ({ page }) => {
    // Open a file first so the editor is visible
    await page.locator('.tree-row.file').first().click()
    await expect(page.locator('textarea.raw')).toBeVisible()

    await openSettings(page)
    const panel = page.getByRole('dialog', { name: 'Settings' })

    // Find the line numbers toggle and turn it on
    const lineNumbersRow = panel.locator('.setting-row.toggle').filter({ hasText: 'Line numbers' })
    const toggle = lineNumbersRow.locator('.toggle-track')
    const checkbox = lineNumbersRow.locator('input[type="checkbox"]')
    const isOn = await checkbox.isChecked()

    if (!isOn) {
      await lineNumbersRow.click()
      await expect(toggle).toHaveClass(/on/)
    }

    await page.keyboard.press('Escape')
    await expect(page.locator('.gutter')).toBeVisible()
  })

  test('sync interval input is present and accepts numeric input', async ({ page }) => {
    await openSettings(page)
    const numInput = page.locator('input.num-input')
    await expect(numInput).toBeVisible()
    await numInput.fill('5')
    await numInput.press('Enter')
    // Value should be 5 (or clamped to ≥1)
    const val = await numInput.inputValue()
    expect(Number(val)).toBeGreaterThanOrEqual(1)
  })
})
