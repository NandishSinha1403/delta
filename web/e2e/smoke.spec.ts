// Callbacks passed to page.evaluate run in the browser, so this file needs the DOM types.
/// <reference lib="dom" />
import { expect, test, type Page } from '@playwright/test';

// Runs at the three layout sizes from docs/plan/05-PLAYGROUND-UX.md §2 (see playwright.config.ts).
// The page opens on the tour example (examples/tour.delta): an NFA N, D = dfa(N), R = regex(D).

/** Shows an inspector view, wherever it lives in the current layout. */
async function openView(page: Page, layout: string, view: 'Tuple' | 'Steps' | 'Run' | 'Problems') {
  if (layout === 'phone') {
    if (view === 'Tuple' || view === 'Run') {
      await page.getByRole('tab', { name: view, exact: true }).click();
    } else {
      await page.getByRole('tab', { name: 'More' }).click();
      await page.getByRole('tab', { name: view }).click();
    }
  } else if (layout === 'tablet' || view !== 'Run') {
    // The desktop run strip is always visible.
    await page.getByRole('tab', { name: view, exact: true }).click();
  }
}

/** Selects an object: the phone's dropdown, or the segmented switcher above the diagram. */
async function pickObject(page: Page, layout: string, name: string) {
  if (layout === 'phone') await page.getByLabel('Object', { exact: true }).selectOption(name);
  else await page.getByRole('tablist', { name: 'Objects' }).getByRole('tab', { name: new RegExp(`^${name} `) }).click();
}

/** Opens the command palette, types `query` and chooses the option matching `option`. */
async function palette(page: Page, query: string, option: RegExp) {
  await page.getByRole('button', { name: /search/i }).click();
  await page.getByRole('combobox', { name: /search/i }).fill(query);
  // Options appear once the file has been evaluated; wait for the one we mean.
  await page.getByRole('option', { name: option }).first().click();
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
  } else {
    await expect(page.getByRole('navigation', { name: 'Open files' })).toContainText('tour.delta');
    await expect(page.getByTestId('codelens').first()).toBeVisible();
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
  await pickObject(page, layout, 'D');
  await openView(page, layout, 'Steps');
  await expect(page.getByTestId('step-counter')).toHaveText('Result · 8 steps');

  await page.getByRole('button', { name: 'Start from step 1' }).click();
  await expect(page.getByTestId('step-counter')).toHaveText('Step 1 of 8');
  await expect(page.getByTestId('step-message')).toContainText('E({q1}) = {q1}');
  await page.getByRole('button', { name: 'Next step' }).click();
  await expect(page.getByTestId('step-message')).toContainText('{q1,q2} (a new state)');
  await page.getByRole('slider', { name: 'Step' }).fill('8');
  await expect(page.getByTestId('step-counter')).toHaveText('Result · 8 steps');

  await pickObject(page, layout, 'R');
  await openView(page, layout, 'Tuple');
  // Typeset like the book: ∪ (never a "U"), and every star a superscript.
  const regex = page.getByTestId('regex');
  await expect(regex.getByRole('math', { name: 'R = b*aa*b((bb*a∪a)a*b)*' })).toBeVisible();
  await expect(regex.locator('.katex')).toContainText('∪');
  await expect(regex.locator('.katex')).not.toContainText('U');
  await expect(regex.locator('.katex .msupsub')).toHaveCount(5);
});

test('runs N step by step: tape, fingers, timeline, sentence, verdict', async ({ page }, testInfo) => {
  const layout = testInfo.project.name;
  await openView(page, layout, 'Run');
  const input = page.getByRole('textbox', { name: 'Input for N' });
  await input.fill('bab');
  await page.getByRole('button', { name: 'Run', exact: true }).click();
  await page.getByRole('button', { name: 'Step forward' }).click();
  await expect(page.getByTestId('run-message')).toContainText('Read b');
  await expect(page.getByTestId('run-explain')).toHaveText('q1 follows b to q1.');
  await expect(page.getByRole('list', { name: 'Configurations' })).toContainText('{q1}');
  for (let i = 0; i < 2; i++) await page.getByRole('button', { name: 'Step forward' }).click();
  await expect(page.getByTestId('verdict')).toHaveAttribute('data-status', 'accepted');
  // Two fingers at the end: on q1 and on the accept state q3.
  await expect(page.locator('[data-finger]')).toHaveCount(2);

  await input.fill('ba');
  await page.getByRole('button', { name: 'Run', exact: true }).click();
  for (let i = 0; i < 2; i++) await page.getByRole('button', { name: 'Step forward' }).click();
  await expect(page.getByTestId('verdict')).toHaveAttribute('data-status', 'rejected');
});

test('an empty input shows an ε cell before ⊣', async ({ page }, testInfo) => {
  const layout = testInfo.project.name;
  await openView(page, layout, 'Run');
  await page.getByRole('textbox', { name: 'Input for N' }).fill('');
  await page.getByRole('button', { name: 'Run', exact: true }).click();
  const eps = page.getByTestId('empty-input');
  await expect(eps).toHaveText('ε');
  await expect(eps).toHaveAttribute('title', 'empty input — no symbols to read');
  await expect(page.getByTestId('verdict')).toHaveAttribute('data-status', 'rejected');
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

test('the command palette opens examples and applies conversions', async ({ page }, testInfo) => {
  const layout = testInfo.project.name;
  await palette(page, '1.41', /Example 1\.41/);
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toContainText('nfa N4');
  if (layout !== 'phone') {
    await expect(page.getByRole('navigation', { name: 'Open files' })).toContainText('ex1-41-N4.delta');
  }
  // A conversion appends a let line with a fresh name and selects the result.
  await palette(page, 'star(N4)', /star\(N4\)/);
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toContainText('let S = star(N4)');
  if (layout === 'phone') await page.getByRole('tab', { name: 'Diagram' }).click();
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^NFA S,/);
});

test('closure constructions from the gallery build and pass their checks', async ({ page }, testInfo) => {
  const layout = testInfo.project.name;
  for (const [query, option, object, kind] of [
    ['union product', /^Examples\s*Theorem 1\.25.*closed under union/, 'M', 'DFA'],
    ['star operation', /^Examples\s*Theorem 1\.49/, 'N', 'NFA'],
  ] as const) {
    await palette(page, query, option);
    await pickObject(page, layout, object);
    if (layout === 'phone') await page.getByRole('tab', { name: 'Diagram' }).click();
    await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', new RegExp(`^${kind} ${object},`));
    await expect(page.getByText(/^(✓ |tests )(\d+)\/\2$/).first()).toBeVisible();
  }
});

test('Convert buttons add the conversion and show its result', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name === 'phone', 'Convert buttons live above the desktop/tablet diagram');
  const convert = page.getByRole('button', { name: /→ DFA/ });
  await expect(convert).toBeVisible();
  // Only the conversions that exist and apply: an NFA offers → DFA, → GNFA, → regex.
  await expect(page.getByRole('button', { name: /→ GNFA/ })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Share' })).toHaveCount(0);
  await expect(page.getByRole('button', { name: 'Export' })).toHaveCount(0);
  await convert.click();
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toContainText('let D2 = dfa(N)');
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^DFA D2,/);
});

// The symbol bar must show a real union sign (a cup), not a letter: Plex Mono's ∪ reads as "u" and
// JetBrains Mono's as "U". The button is screenshotted next to identical buttons holding "U" and
// "u"; the pixels must differ from both. (Compared in the same browser, so no stored baseline is
// needed and the test means the same on macOS and on CI's Linux.)
test('symbol bar shows a real ∪ and a superscript star', async ({ page }, testInfo) => {
  const bar = page.getByTestId('symbol-bar');
  await expect(bar).toBeVisible();
  const union = bar.getByRole('button', { name: /union/ });
  await expect(union).toHaveText('∪');
  await page.evaluate(() => document.fonts.ready);
  expect(await page.evaluate(() => document.fonts.check('16px "Delta Cup"', '∪'))).toBe(true);
  await expect(bar.getByRole('button', { name: /star/ }).locator('sup')).toHaveText('∗');

  const shot = await union.screenshot();
  await testInfo.attach('symbol-bar', { body: await bar.screenshot(), contentType: 'image/png' });
  for (const letter of ['U', 'u']) {
    await union.evaluate((el, l) => {
      const twin = el.cloneNode(true) as HTMLElement;
      twin.textContent = l;
      twin.dataset.twin = 'true';
      el.after(twin);
    }, letter);
    const twin = bar.locator('[data-twin="true"]');
    const other = await twin.screenshot();
    expect(shot.equals(other), `∪ must not look like "${letter}"`).toBe(false);
    await twin.evaluate((el) => el.remove());
  }
});

test('the author credit links to GitHub', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== 'desktop', 'The credit sits in the wide status bar');
  await expect(page.getByRole('link', { name: 'Nandish Sinha' })).toHaveAttribute('href', 'https://github.com/NandishSinha1403');
});

/** Replaces the whole program in the editor. */
async function setCode(page: Page, text: string) {
  await page.getByRole('textbox', { name: 'Delta code' }).click();
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.insertText(text);
}

const SMALL_DFA = `dfa M {
  Q = { p, q }
  Σ = { a }
  δ { p -a-> q
      q -a-> p }
  q0 = p
  F = { p }
}
check M accepts "", "aa"
check M rejects "a"
`;

test('Auto off: edits wait for Execute (Ctrl+Enter), with a stale banner meanwhile', async ({ page }, testInfo) => {
  const layout = testInfo.project.name;
  const auto = page.getByRole('switch', { name: 'Auto' });
  await expect(auto).toHaveAttribute('aria-checked', 'true');
  await auto.click();
  await expect(auto).toHaveAttribute('aria-checked', 'false');

  await setCode(page, SMALL_DFA);
  await expect(page.getByTestId('stale-dot')).toBeVisible();
  if (layout === 'phone') await page.getByRole('tab', { name: 'Diagram' }).click();
  await expect(page.getByTestId('stale-banner')).toContainText('Code changed — press');
  // The old diagram stays until Execute.
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^NFA N,/);

  await page.keyboard.press('Control+Enter');
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^DFA M, 2 states/);
  await expect(page.getByTestId('stale-banner')).toHaveCount(0);
  if (layout === 'phone') await page.getByRole('tab', { name: 'Code' }).click();
  await expect(page.getByTestId('stale-dot')).toHaveCount(0);
  if (layout !== 'phone') await expect(page.getByTestId('executed')).toHaveText(/^Executed in \d+ ms$/);

  // The choice is remembered.
  await page.reload();
  await expect(page.getByRole('switch', { name: 'Auto' })).toHaveAttribute('aria-checked', 'false');
});

/** "The 13th symbol from the end is 1": its DFA has 2¹³ states, ten times over. */
function slowProgram(): string {
  const k = 13;
  const states = Array.from({ length: k + 1 }, (_, i) => `q${i}`).join(', ');
  const arrows = Array.from({ length: k - 1 }, (_, i) => `    q${i + 1} -[0,1]-> q${i + 2}`).join('\n');
  const lets = Array.from({ length: 10 }, (_, i) => `let D${i} = dfa(N)`).join('\n');
  return `nfa N {\n  Q = { ${states} }\n  Σ = { 0, 1 }\n  δ {\n    q0 -[0,1]-> q0\n    q0 -1-> q1\n${arrows}\n  }\n  q0 = q0\n  F = { q${k} }\n}\n${lets}\n`;
}

test('a slow evaluation shows the loader, and Stop cancels it', async ({ page }, testInfo) => {
  test.skip(testInfo.project.name === 'phone', 'The loader sits under the diagram header');
  await setCode(page, slowProgram());
  const loader = page.getByTestId('loader');
  await expect(loader).toBeVisible();
  await expect(loader).toContainText('Evaluating…');
  // The previous diagram stays visible, dimmed.
  await expect(page.locator('[aria-busy="true"]')).toBeVisible();
  await loader.getByRole('button', { name: 'Stop' }).click();
  await expect(loader).toHaveCount(0);
  await expect(page.getByTestId('executed')).toHaveText('Stopped');
  await expect(page.locator('[aria-busy="true"]')).toHaveCount(0);

  // The restarted engine works: a small program evaluates normally.
  await setCode(page, SMALL_DFA);
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^DFA M, 2 states/);
  await expect(page.getByTestId('executed')).toHaveText(/^Executed in \d+ ms$/);
});

test('fast evaluations never flash the loader', async ({ page }) => {
  await expect(page.getByTestId('diagram').or(page.getByRole('tab', { name: 'Diagram' })).first()).toBeVisible();
  let seen = false;
  await page.exposeFunction('sawLoader', () => {
    seen = true;
  });
  await page.evaluate(() => {
    new MutationObserver(() => {
      if (document.querySelector('[data-testid="loader"]')) (window as unknown as { sawLoader: () => void }).sawLoader();
    }).observe(document.body, { childList: true, subtree: true });
  });
  await setCode(page, SMALL_DFA);
  await page.waitForTimeout(800);
  expect(seen).toBe(false);
});
