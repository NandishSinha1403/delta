import { expect, test } from '@playwright/test';

test('the front page hero is a live Pad', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'The language of computation.' })).toBeVisible();
  const hero = page.frameLocator('iframe[title^="A live Delta program"]');
  await expect(hero.locator('[data-layout="embed"]')).toBeVisible();
  await expect(hero.getByTestId('diagram')).toHaveAttribute('aria-label', /^DFA EndsAB, 3 states/);
});

test('Open in the Pad carries the example’s code', async ({ page }) => {
  await page.goto('/docs/tour/01-first-program/');
  await page.getByRole('link', { name: 'Open in the Pad ↗' }).first().click();
  await expect(page).toHaveURL(/\/pad\/#code=/);
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toContainText('dfa EvenAs');
  await expect(page.getByTestId('error-count').or(page.getByText(/^✓ \d+\/\d+$/)).first()).toBeVisible();
});

test('docs search finds the subset construction', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name === 'phone', 'The search box is behind a button on phones');
  await page.goto('/docs/');
  await page.locator('site-search button[data-open-modal]').click();
  await page.locator('.pagefind-ui__search-input').fill('subset construction');
  await expect(page.locator('.pagefind-ui__result-link').first()).toBeVisible();
});

test('a cookbook recipe renders with its runnable program', async ({ page }) => {
  await page.goto('/docs/cookbook/pda-anbn/');
  await expect(page.getByRole('heading', { name: 'PDA: aⁿbⁿ' })).toBeVisible();
  await expect(page.locator('pre').first()).toContainText('pda P');
  await expect(page.getByRole('link', { name: 'Open in the Pad ↗' })).toHaveCount(1);
});

test('the AI guide and the output schema are published', async ({ request }) => {
  const llms = await request.get('/llms.txt');
  expect(llms.ok()).toBe(true);
  expect(await llms.text()).toContain('Delta 2');
  const schema = await request.get('/schema/output-v2.json');
  expect((await schema.json()).title).toBe('Delta 2 run output');
});

test('the phone layout has no horizontal scroll', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'phone', 'phone only');
  for (const path of ['/', '/docs/', '/docs/languages/']) {
    await page.goto(path);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    expect(overflow, path).toBeLessThanOrEqual(1);
  }
});
