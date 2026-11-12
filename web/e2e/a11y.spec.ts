// Callbacks passed to page.evaluate run in the browser, so this file needs the DOM types.
/// <reference lib="dom" />
// Accessibility audit (04-ROADMAP.md, Phase 7): axe-core's WCAG 2.2 A and AA rules on the Pad's
// main states, at every viewport in playwright.config.ts, in both themes.
import AxeBuilder from '@axe-core/playwright';
import { expect, test, type Page } from '@playwright/test';

const TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];

/** Runs axe on the page (or one part of it) and fails with a readable list of violations. */
async function audit(page: Page, include?: string) {
  let builder = new AxeBuilder({ page }).withTags(TAGS);
  if (include) builder = builder.include(include);
  const { violations } = await builder.analyze();
  const report = violations.map(
    (v) => `${v.id} (${v.impact}): ${v.help}\n${v.nodes.map((n) => `  ${n.target.join(' ')}: ${n.failureSummary ?? ''}`).join('\n')}`,
  );
  expect(report, report.join('\n\n')).toEqual([]);
}

/** Waits for the first evaluation, so the panels hold real content. */
async function ready(page: Page) {
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toBeVisible();
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /EndsAB/);
}

for (const theme of ['dark', 'light'] as const) {
  test.describe(`${theme} theme`, () => {
    test.beforeEach(async ({ page }) => {
      await page.addInitScript((t) => window.localStorage.setItem('delta.theme', t), theme);
    });

    test('the Pad has no WCAG A/AA violations', async ({ page }) => {
      await page.goto('/');
      await ready(page);
      await audit(page);
    });

    test('the command palette has no WCAG A/AA violations', async ({ page }) => {
      await page.goto('/');
      await ready(page);
      await page.getByRole('button', { name: /search/i }).click();
      await page.getByRole('combobox', { name: /search/i }).fill('present');
      await audit(page, 'dialog[aria-label="Command palette"]');
    });

    test('presentation mode has no WCAG A/AA violations', async ({ page }) => {
      await page.goto('/?present=diagram');
      await expect(page.getByTestId('presentation')).toBeVisible();
      await audit(page);
    });

    test('the embedded Pad has no WCAG A/AA violations', async ({ page }) => {
      await page.goto('/#embed=1&panel=tests');
      await page.reload();
      await expect(page.locator('[data-layout="embed"]')).toBeVisible();
      await audit(page);
    });
  });
}
