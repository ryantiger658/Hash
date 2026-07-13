// @ts-check
import { test, expect } from '../fixtures.js'

async function openFreshNote(page, name) {
  await page.keyboard.press('Control+n')
  const modal = page.getByRole('dialog', { name: 'New note' })
  await modal.locator('input[placeholder="note.md"]').fill(name)
  await modal.getByRole('button', { name: 'Create' }).click()
  await expect(page.locator('textarea.raw')).toHaveValue('', { timeout: 8_000 })
}

test.describe('Tag browser, focus mode, and safe diagrams', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('textarea.raw')).toBeVisible({ timeout: 12_000 })
  })

  test('tag browser matches scalar frontmatter tags and opens the tagged note', async ({ page }) => {
    const name = `e2e-tags-${Date.now()}.md`
    await openFreshNote(page, name)
    await page.locator('textarea.raw').fill('---\ntags: rust, #async systems\n---\n# Tagged')
    await page.keyboard.press('Control+s')

    const browser = page.getByRole('button', { name: 'Browse tags' })
    await expect(browser).toBeVisible({ timeout: 5_000 })
    await browser.click()
    await page.getByRole('button', { name: /#async/ }).click()
    await expect(page.locator('.tag-file-row')).toContainText(name.replace(/\.md$/, ''))
    await page.locator('.tag-file-row').click()
    await expect(page.locator('textarea.raw')).toContainText('tags: rust')
  })

  test('focus mode toggles with the keyboard and Escape restores the chrome', async ({ page }) => {
    await page.keyboard.press('Control+Shift+f')
    await expect(page.locator('.app')).toHaveClass(/focus-mode/)
    await expect(page.locator('header')).not.toBeVisible()
    await page.keyboard.press('Escape')
    await expect(page.locator('.app')).not.toHaveClass(/focus-mode/)
    await expect(page.locator('header')).toBeVisible()
  })

  test('Mermaid renders dynamically and untrusted markdown is sanitized', async ({ page }) => {
    const name = `e2e-mermaid-${Date.now()}.md`
    await openFreshNote(page, name)
    await page.locator('.editor-mode-float button[title="Split"]').click()
    await page.locator('textarea.raw').fill('```mermaid\ngraph TD\n  A --> B\n```\n\n<img src=x onerror="window.__hashXss = true">')
    await expect(page.locator('.viewer pre.mermaid svg')).toBeVisible({ timeout: 10_000 })
    await expect.poll(() => page.evaluate(() => window.__hashXss === true)).toBe(false)
  })
})
