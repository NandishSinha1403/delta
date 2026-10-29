import { expect, test, type Page } from '@playwright/test';

// Runs at the three layout sizes from docs/plan/05-PLAYGROUND-UX.md §2 (see playwright.config.ts).
// The page opens on the tour example (examples/tour.delta): an NFA N, D = dfa(N), R = regex(D).

/** Shows an inspector view, wherever it lives in the current layout. */
async function openView(page: Page, layout: string, view: 'Tuple' | 'Steps' | 'Run' | 'Problems') {
  if (layout === 'phone') {
    if (view === 'Tuple' || view === 'Run') {
      await page.getByRole('tab', { name: view }).click();
    } else {
      await page.getByRole('tab', { name: 'More' }).click();
      await page.getByRole('tab', { name: view }).click();
    }
  } else if (layout === 'tablet' || view !== 'Run') {
    await page.getByRole('tab', { name: view, exact: true }).click();
  }
}

test.beforeEach(async ({ page }) => {
  await page.goto('/');
});

test('shell renders the tour with its tuple and diagram', async ({ page }, testInfo) => {
  const layout = testInfo.project.name; // 'phone' | 'tablet' | 'desktop'
  await expect(page.locator(`[data-layout="${layout}"]`)).toBeVisible();
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toBeVisible();
  if (layout === 'phone') {
    await expect(page.getByRole('toolbar', { name: 'Insert book symbols' })).toBeVisible();
    await page.getByRole('tab', { name: 'Diagram' }).click();
  }
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^NFA N, 3 states, start q1, accept q3/);

  await openView(page, layout, 'Tuple');
  const tuple = page.getByTestId('tuple');
  await expect(tuple.getByRole('math', { name: 'N = (Q, Σ, δ, q1, F)' })).toBeVisible();
  await expect(tuple.getByRole('math', { name: 'Q = {q1, q2, q3}' })).toBeVisible();
  await expect(tuple.getByRole('math', { name: 'F = {q3}' })).toBeVisible();
});

// Phase 1 exit (docs/plan/06-ROADMAP-AND-TESTING.md): an NFA becomes a DFA, the student scrubs
// the subset construction, then gets the regular expression.
test('NFA → dfa(N) → scrub the steps → regex(D)', async ({ page }, testInfo) => {
  const layout = testInfo.project.name;
  await page.getByLabel('Object').selectOption('D');
  await openView(page, layout, 'Steps');
  await expect(page.getByTestId('step-counter')).toHaveText('Result · 8 steps');

  await page.getByRole('button', { name: 'Start from step 1' }).click();
  await expect(page.getByTestId('step-counter')).toHaveText('Step 1 of 8');
  await expect(page.getByTestId('step-message')).toContainText('E({q1}) = {q1}');
  await page.getByRole('button', { name: 'Next step' }).click();
  await expect(page.getByTestId('step-message')).toContainText('{q1,q2} (a new state)');
  await page.getByRole('slider', { name: 'Step' }).fill('8');
  await expect(page.getByTestId('step-counter')).toHaveText('Result · 8 steps');

  await page.getByLabel('Object').selectOption('R');
  await openView(page, layout, 'Tuple');
  // Typeset like the book: ∪ (never a "U"), and every star a superscript.
  const regex = page.getByTestId('regex');
  await expect(regex.getByRole('math', { name: 'R = b*aa*b((bb*a∪a)a*b)*' })).toBeVisible();
  await expect(regex.locator('.katex')).toContainText('∪');
  await expect(regex.locator('.katex')).not.toContainText('U');
  await expect(regex.locator('.katex .msupsub')).toHaveCount(5);
});

test('runs N step by step to a verdict', async ({ page }, testInfo) => {
  const layout = testInfo.project.name;
  await openView(page, layout, 'Run');
  const input = page.getByRole('textbox', { name: 'Input for N' });
  await input.fill('bab');
  await page.getByRole('button', { name: 'Run', exact: true }).click();
  await page.getByRole('button', { name: 'Step forward' }).click();
  await expect(page.getByTestId('run-message')).toContainText('Read b');
  await page.getByRole('button', { name: 'Jump to end' }).click();
  await expect(page.getByTestId('verdict')).toHaveText('Accepted');

  await input.fill('ba');
  await page.getByRole('button', { name: 'Run', exact: true }).click();
  await page.getByRole('button', { name: 'Jump to end' }).click();
  await expect(page.getByTestId('verdict')).toHaveText('Rejected');
});

test('reports a problem as the code is edited', async ({ page }, testInfo) => {
  const layout = testInfo.project.name;
  const editor = page.getByRole('textbox', { name: 'Delta code' });
  await editor.click();
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.insertText('dfa M {\n  Q = { q1 }\n  Σ = { 0 }\n  δ { q1 -0-> q9 }\n  q0 = q1\n  F = { q1 }\n}\n');
  await openView(page, layout, 'Problems');
  await expect(page.getByRole('list', { name: 'Problems' })).toContainText('q9');
});

test('opens an example from the gallery', async ({ page }) => {
  await page.getByRole('button', { name: 'Examples' }).click();
  await page.getByRole('button', { name: /Example 1\.41/ }).click();
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toContainText('nfa N4');
  await expect(page.getByLabel('Object')).toHaveValue('N4');
});

// Chapter 1.1–1.3 closure constructions, opened from the gallery and checked end to end.
test('closure constructions from the gallery build and pass their checks', async ({ page }, testInfo) => {
  const layout = testInfo.project.name;
  for (const [title, object, kind] of [
    [/^Theorem 1\.25.*closed under union/, 'M', 'DFA'],
    [/Theorem 1\.49/, 'N', 'NFA'],
  ] as const) {
    await page.getByRole('button', { name: 'Examples' }).click();
    await page.getByRole('button', { name: title }).click();
    await page.getByLabel('Object').selectOption(object);
    if (layout === 'phone') await page.getByRole('tab', { name: 'Diagram' }).click();
    await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', new RegExp(`^${kind} ${object},`));
    if (layout !== 'phone') await expect(page.getByText(/^Tests (\d+)\/\1$/)).toBeVisible();
  }
});
