// @ts-check
import { test, expect } from '../fixtures.js'

// Wait helper — ensures onMount's loadVault().then(openTodayJournal) has
// completed and the editor is showing a file before tests start.
async function waitForAppReady(page) {
  await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 12_000 })
}

test.describe('File tree', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await waitForAppReady(page)
  })

  test('shows .md files in the tree', async ({ page }) => {
    await expect(page.locator('.tree-row').first()).toBeVisible()
  })

  test('clicking a file opens it in the editor', async ({ page }) => {
    const firstFile = page.locator('.tree-row.file').first()
    await firstFile.click()
    await expect(page.locator('textarea.raw')).toBeVisible()
  })
})

test.describe('Creating files', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await waitForAppReady(page)
  })

  test('Ctrl+N opens the new note modal', async ({ page }) => {
    await page.keyboard.press('Control+n')
    await expect(page.getByRole('dialog', { name: 'New note' })).toBeVisible()
  })

  test('creates a file and opens it in the editor', async ({ page }) => {
    const name = `e2e-test-${Date.now()}.md`
    await page.keyboard.press('Control+n')
    const modal = page.getByRole('dialog', { name: 'New note' })
    await modal.locator('input[placeholder="note.md"]').fill(name)
    await modal.getByRole('button', { name: 'Create' }).click()

    await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 8_000 })
    // Tree displays names without .md extension
    await expect(page.locator(`.tree-row:has-text("${name.replace(/\.md$/, '')}")`)).toBeVisible()
  })

  test('creating a nested path creates folder and file', async ({ page }) => {
    const folder = `e2e-folder-${Date.now()}`
    const name = `${folder}/nested.md`
    await page.keyboard.press('Control+n')
    const modal = page.getByRole('dialog', { name: 'New note' })
    await modal.locator('input[placeholder="note.md"]').fill(name)
    await modal.getByRole('button', { name: 'Create' }).click()

    await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 8_000 })
    await expect(page.locator(`.tree-row:has-text("${folder}")`)).toBeVisible()
  })

  test('path with .. is rejected by the modal', async ({ page }) => {
    await page.keyboard.press('Control+n')
    const modal = page.getByRole('dialog', { name: 'New note' })
    await modal.locator('input[placeholder="note.md"]').fill('../escape.md')
    await modal.getByRole('button', { name: 'Create' }).click()
    // Modal should stay open and show an error
    await expect(modal).toBeVisible()
    await expect(modal.locator('.error')).toBeVisible()
  })
})

test.describe('Deleting files', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await waitForAppReady(page)
  })

  test('hovering a file reveals a delete button', async ({ page }) => {
    // Rename/delete buttons are siblings of .tree-row.file inside .file-row-wrap
    const fileWrap = page.locator('.file-row-wrap').first()
    await fileWrap.hover()
    await expect(fileWrap.locator('button[title="Delete file"]')).toBeVisible()
  })

  test('deleting a file removes it from the tree', async ({ page }) => {
    const name = `e2e-delete-${Date.now()}.md`
    await page.keyboard.press('Control+n')
    const modal = page.getByRole('dialog', { name: 'New note' })
    await modal.locator('input[placeholder="note.md"]').fill(name)
    await modal.getByRole('button', { name: 'Create' }).click()
    await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 8_000 })

    const baseName = name.replace(/\.md$/, '')
    const fileWrap = page.locator(`.file-row-wrap:has-text("${baseName}")`)
    await fileWrap.hover()
    // Accept the browser confirm() dialog that appears before deletion
    page.once('dialog', dialog => dialog.accept())
    await fileWrap.locator('button[title="Delete file"]').click()

    await expect(fileWrap).not.toBeVisible()
  })
})

test.describe('Renaming files', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await waitForAppReady(page)
  })

  test('hovering a file reveals a rename button', async ({ page }) => {
    const fileWrap = page.locator('.file-row-wrap').first()
    await fileWrap.hover()
    await expect(fileWrap.locator('button[title="Rename"]')).toBeVisible()
  })

  test('rename shows inline input pre-filled with current name', async ({ page }) => {
    const name = `e2e-rename-src-${Date.now()}.md`
    await page.keyboard.press('Control+n')
    const modal = page.getByRole('dialog', { name: 'New note' })
    await modal.locator('input[placeholder="note.md"]').fill(name)
    await modal.getByRole('button', { name: 'Create' }).click()
    await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 8_000 })

    const baseName = name.replace(/\.md$/, '')
    const fileWrap = page.locator(`.file-row-wrap:has-text("${baseName}")`)
    await fileWrap.hover()
    await fileWrap.locator('button[title="Rename"]').click()

    // After clicking rename, the has-text filter no longer matches (input replaces text),
    // so use a page-level locator for the rename input
    const renameInput = page.locator('.rename-input')
    await expect(renameInput).toBeVisible()
    // Rename input is pre-filled without .md extension
    await expect(renameInput).toHaveValue(baseName)
  })

  test('Escape cancels rename without changing anything', async ({ page }) => {
    const name = `e2e-rename-esc-${Date.now()}.md`
    await page.keyboard.press('Control+n')
    const modal = page.getByRole('dialog', { name: 'New note' })
    await modal.locator('input[placeholder="note.md"]').fill(name)
    await modal.getByRole('button', { name: 'Create' }).click()
    await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 8_000 })

    const baseName = name.replace(/\.md$/, '')
    const fileWrap = page.locator(`.file-row-wrap:has-text("${baseName}")`)
    await fileWrap.hover()
    await fileWrap.locator('button[title="Rename"]').click()
    await page.keyboard.press('Escape')

    await expect(page.locator('.rename-input')).not.toBeVisible()
    // has-text should match again after rename is cancelled (text is restored)
    await expect(page.locator(`.file-row-wrap:has-text("${baseName}")`)).toBeVisible()
  })
})
