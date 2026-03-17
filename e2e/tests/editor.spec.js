// @ts-check
import { test, expect } from '../fixtures.js'

test.describe('Editor', () => {
  /** Open a fresh temp note and return its name.
   * @param {import('@playwright/test').Page} page */
  async function openFreshNote(page) {
    const name = `e2e-editor-${Date.now()}.md`
    await page.keyboard.press('Control+n')
    const modal = page.getByRole('dialog', { name: 'New note' })
    await modal.locator('input[placeholder="note.md"]').fill(name)
    await modal.getByRole('button', { name: 'Create' }).click()
    await expect(page.locator('textarea.raw')).toBeVisible()
    // Wait for the empty file to fully load so subsequent fills aren't overwritten
    await expect(page.locator('textarea.raw')).toHaveValue('', { timeout: 5_000 })
    return name
  }

  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 12_000 })
  })

  test('typing in the editor updates the preview pane', async ({ page }) => {
    await openFreshNote(page)
    // Ensure split mode so the preview pane is visible
    await page.locator('.editor-mode-float button[title="Split"]').click()
    const editor = page.locator('textarea.raw')
    await editor.fill('# Hello preview')
    const preview = page.locator('.preview')
    await expect(preview.locator('h1')).toHaveText('Hello preview')
  })

  test('Ctrl+S saves the file (save dot turns chartreuse)', async ({ page }) => {
    await openFreshNote(page)
    await page.locator('textarea.raw').fill('saved content')
    await page.keyboard.press('Control+s')
    // Save dot should show a saved state — no error class
    await expect(page.locator('.float-save-dot')).toBeVisible()
    await expect(page.locator('.float-save-dot.error')).not.toBeVisible({ timeout: 3_000 })
  })

  test('auto-save fires after typing stops', async ({ page }) => {
    await openFreshNote(page)
    await page.locator('textarea.raw').fill('auto-saved')
    // Wait for autosave (1.5 s) + small buffer
    await page.waitForTimeout(2_500)
    await expect(page.locator('.float-save-dot.error')).not.toBeVisible()
  })

  test.describe('Editor mode switching', () => {
    test('Edit mode shows only the textarea', async ({ page }) => {
      await openFreshNote(page)
      await page.locator('.editor-mode-float button[title="Edit"]').click()
      await expect(page.locator('textarea.raw')).toBeVisible()
      await expect(page.locator('.preview')).not.toBeVisible()
    })

    test('Preview mode shows only the rendered preview', async ({ page }) => {
      await openFreshNote(page)
      await page.locator('textarea.raw').fill('# Preview only')
      await page.locator('.editor-mode-float button[title="Preview"]').click()
      await expect(page.locator('.preview')).toBeVisible()
      await expect(page.locator('textarea.raw')).not.toBeVisible()
    })

    test('Split mode shows both panes', async ({ page }) => {
      await openFreshNote(page)
      // Switch away then back
      await page.locator('.editor-mode-float button[title="Preview"]').click()
      await page.locator('.editor-mode-float button[title="Split"]').click()
      await expect(page.locator('textarea.raw')).toBeVisible()
      await expect(page.locator('.preview')).toBeVisible()
    })
  })

  test.describe('Auto-continue lists', () => {
    test('Enter at end of "- item" inserts "- " on next line', async ({ page }) => {
      await openFreshNote(page)
      const editor = page.locator('textarea.raw')
      await editor.click()
      await editor.fill('- first item')
      await page.keyboard.press('End')
      await page.keyboard.press('Enter')
      const value = await editor.inputValue()
      expect(value).toContain('- first item\n- ')
    })

    test('Enter at end of "1. item" inserts "2. " on next line', async ({ page }) => {
      await openFreshNote(page)
      const editor = page.locator('textarea.raw')
      await editor.click()
      await editor.fill('1. first')
      await page.keyboard.press('End')
      await page.keyboard.press('Enter')
      const value = await editor.inputValue()
      expect(value).toContain('1. first\n2. ')
    })

    test('Enter at end of "- [ ] task" inserts "- [ ] " on next line', async ({ page }) => {
      await openFreshNote(page)
      const editor = page.locator('textarea.raw')
      await editor.click()
      await editor.fill('- [ ] task')
      await page.keyboard.press('End')
      await page.keyboard.press('Enter')
      const value = await editor.inputValue()
      expect(value).toContain('- [ ] task\n- [ ] ')
    })

    test('Enter on empty list item exits the list', async ({ page }) => {
      await openFreshNote(page)
      const editor = page.locator('textarea.raw')
      await editor.click()
      await editor.fill('- item\n- ')
      await page.keyboard.press('End')
      await page.keyboard.press('Enter')
      const value = await editor.inputValue()
      // The empty marker should be removed
      expect(value).not.toMatch(/- $/)
    })
  })

  test.describe('Checkbox toggles in preview', () => {
    test('clicking a checkbox in preview toggles [ ] to [x] in the editor', async ({ page }) => {
      await openFreshNote(page)
      // Ensure split mode so the preview pane is visible
      await page.locator('.editor-mode-float button[title="Split"]').click()
      const editor = page.locator('textarea.raw')
      await editor.fill('- [ ] task one')
      await page.keyboard.press('Control+s')
      await page.waitForTimeout(300)

      // Click the rendered checkbox in the preview
      const checkbox = page.locator('.preview input[type="checkbox"]').first()
      await expect(checkbox).toBeVisible({ timeout: 3_000 })
      await checkbox.click()

      // Editor content should now contain [x]
      await page.waitForTimeout(300)
      const value = await editor.inputValue()
      expect(value).toContain('[x]')
    })
  })
})
