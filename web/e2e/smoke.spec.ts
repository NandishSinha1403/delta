import { expect, test } from '@playwright/test';

// Proves the whole Phase 0 pipeline on every screen size:
// the right shell renders, and Rust (WASM, in a Web Worker) answers with M1's 5-tuple.
test('shell renders and shows the tuple computed by the Rust engine', async ({ page }, testInfo) => {
  const layout = testInfo.project.name; // 'phone' | 'tablet' | 'desktop'
  await page.goto('/');

  await expect(page.locator(`[data-layout="${layout}"]`)).toBeVisible();
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toBeVisible();

  if (layout === 'phone') {
    // One view at a time: the editor's symbol bar is here, the tuple is behind a tab.
    await expect(page.getByRole('toolbar', { name: 'Insert book symbols' })).toBeVisible();
    await page.getByRole('tab', { name: 'Tuple' }).click();
  } else {
    await expect(page.getByRole('region', { name: 'Diagram' })).toBeVisible();
  }

  const tuple = page.getByTestId('tuple');
  await expect(tuple).toContainText('M1 = (Q, Σ, δ, q1, F)');
  await expect(tuple).toContainText('Q = {q1, q2, q3}');
  await expect(tuple).toContainText('F = {q2}');
});
