// Callbacks passed to page.evaluate run in the browser, so this file needs the DOM types.
/// <reference lib="dom" />
import { expect, test, type Page } from '@playwright/test';

// Runs at the three layout sizes from docs/plan/05-PLAYGROUND-UX.md §2 (see playwright.config.ts).
// The page opens on the tour sample (samples/01-tour.delta): a DFA EndsAB, a lang L and an NFA N.

/** Shows an inspector view, wherever it lives in the current layout. */
async function openView(page: Page, layout: string, view: 'Tuple' | 'Steps' | 'Run' | 'Problems' | 'Lang' | 'Console') {
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

/** The app layout a project tests: phone, tablet or desktop (playwright.config.ts metadata). */
function layoutOf(info: { project: { metadata: Record<string, unknown> } }): string {
  return String(info.project.metadata.layout);
}

test.beforeEach(async ({ page }) => {
  await page.goto('/');
});

test('shell renders the tour with its tuple and diagram', async ({ page }, testInfo) => {
  const layout = layoutOf(testInfo); // 'phone' | 'tablet' | 'desktop'
  await expect(page.locator(`[data-layout="${layout}"]`)).toBeVisible();
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toBeVisible();
  if (layout === 'phone') {
    await expect(page.getByRole('toolbar', { name: 'Insert book symbols' })).toBeVisible();
    await page.getByRole('tab', { name: 'Diagram' }).click();
  } else {
    await expect(page.getByRole('navigation', { name: 'Open files' })).toContainText('tour.delta');
  }
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^DFA EndsAB, 3 states, start q0, accept q2/);

  await openView(page, layout, 'Tuple');
  const tuple = page.getByTestId('tuple');
  await expect(tuple.getByRole('math', { name: 'EndsAB = (Q, Σ, δ, q0, F)' })).toBeVisible();
  await expect(tuple.getByRole('math', { name: 'Q = {q0, q1, q2}' })).toBeVisible();
  await expect(tuple.getByRole('math', { name: 'F = {q2}' })).toBeVisible();
});



// the subset construction, then gets the regular expression.
// An NFA becomes a DFA, and the student scrubs the subset construction's steps.
test('subset construction: open the sample and scrub its steps', async ({ page }, testInfo) => {
  const layout = layoutOf(testInfo);
  await palette(page, 'subset', /^Examples\s*Subset construction/);
  await pickObject(page, layout, 'D');
  await openView(page, layout, 'Steps');
  await expect(page.getByTestId('step-counter')).toHaveText(/^Result · \d+ steps$/);
  await page.getByRole('button', { name: 'Start from step 1' }).click();
  await expect(page.getByTestId('step-counter')).toHaveText(/^Step 1 of \d+$/);
  await expect(page.getByTestId('step-message')).toContainText('ε-closure');
  await page.getByRole('button', { name: 'Next step' }).click();
  await expect(page.getByTestId('step-counter')).toHaveText(/^Step 2 of \d+$/);
});

test('runs EndsAB step by step: tape, timeline, sentence, verdict', async ({ page }, testInfo) => {
  const layout = layoutOf(testInfo);
  await openView(page, layout, 'Run');
  const input = page.getByRole('textbox', { name: 'Input for EndsAB' });
  await input.fill('bab');
  await page.getByRole('button', { name: 'Run', exact: true }).click();
  await page.getByRole('button', { name: 'Step forward' }).click();
  await expect(page.getByTestId('run-message')).toContainText('Read b');
  await expect(page.getByTestId('run-explain')).toHaveText('Reading b moves to q0.');
  for (let i = 0; i < 2; i++) await page.getByRole('button', { name: 'Step forward' }).click();
  await expect(page.getByTestId('verdict')).toHaveAttribute('data-status', 'accepted');

  await input.fill('ba');
  await page.getByRole('button', { name: 'Run', exact: true }).click();
  for (let i = 0; i < 2; i++) await page.getByRole('button', { name: 'Step forward' }).click();
  await expect(page.getByTestId('verdict')).toHaveAttribute('data-status', 'rejected');
});

test('an empty input shows an ε cell before ⊣', async ({ page }, testInfo) => {
  const layout = layoutOf(testInfo);
  await openView(page, layout, 'Run');
  await page.getByRole('textbox', { name: 'Input for EndsAB' }).fill('');
  await page.getByRole('button', { name: 'Run', exact: true }).click();
  const eps = page.getByTestId('empty-input');
  await expect(eps).toHaveText('ε');
  await expect(eps).toHaveAttribute('title', 'empty input — no symbols to read');
  await expect(page.getByTestId('verdict')).toHaveAttribute('data-status', 'rejected');
});

test('reports a problem as the code is edited', async ({ page }, testInfo) => {
  const layout = layoutOf(testInfo);
  const editor = page.getByRole('textbox', { name: 'Delta code' });
  await editor.click();
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.insertText('dfa M {\n  states q1\n  alphabet 0\n  start q1\n  accept q1\n  q1 -0-> q9\n}\n');
  await openView(page, layout, 'Problems');
  await expect(page.getByRole('list', { name: 'Problems' })).toContainText('q9');
});

test('the command palette opens samples and applies conversions', async ({ page }, testInfo) => {
  const layout = layoutOf(testInfo);
  await palette(page, 'subset', /^Examples\s*Subset construction/);
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toContainText('nfa N');
  if (layout !== 'phone') {
    await expect(page.getByRole('navigation', { name: 'Open files' })).toContainText('03-subset-construction.delta');
  }
  // A conversion appends a let line with a fresh name and selects the result.
  await palette(page, 'star(N)', /star\(N\)/);
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toContainText('let S = star(N)');
  if (layout === 'phone') await page.getByRole('tab', { name: 'Diagram' }).click();
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^NFA S,/);
});

test('samples from the gallery build and pass their checks', async ({ page }, testInfo) => {
  const layout = layoutOf(testInfo);
  for (const [query, option, object, kind] of [
    ['product', /^Examples\s*Product construction/, 'Both', 'DFA'],
    ['pushdown', /^Examples\s*Pushdown automaton/, 'P', 'PDA'],
  ] as const) {
    await palette(page, query, option);
    await pickObject(page, layout, object);
    if (layout === 'phone') await page.getByRole('tab', { name: 'Diagram' }).click();
    await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', new RegExp(`^${kind} ${object},`));
    await expect(page.getByText(/^(✓ |tests )(\d+)\/\2$/).first()).toBeVisible();
  }
});

test('Convert buttons add the conversion and show its result', async ({ page }, testInfo) => {
  test.skip(layoutOf(testInfo) === 'phone', 'Convert buttons live above the desktop/tablet diagram');
  // Only the conversions that apply: a DFA offers minimize and → regex, not → DFA.
  const convert = page.getByRole('button', { name: 'minimize', exact: true });
  await expect(convert).toBeVisible();
  await expect(page.getByRole('button', { name: /→ regex/ })).toBeVisible();
  await expect(page.getByRole('button', { name: /→ DFA/ })).toHaveCount(0);
  await convert.click();
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toContainText('let M = minimize(EndsAB)');
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^DFA M, 3 states/);
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
  test.skip(layoutOf(testInfo) !== 'desktop', 'The credit sits in the wide status bar');
  await expect(page.getByRole('link', { name: 'Nandish Sinha' })).toHaveAttribute('href', 'https://github.com/NandishSinha1403');
});

/** Replaces the whole program in the editor. */
async function setCode(page: Page, text: string) {
  await page.getByRole('textbox', { name: 'Delta code' }).click();
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.insertText(text);
}

const SMALL_DFA = `dfa M {
  alphabet a
  start p
  accept p
  p -a-> q
  q -a-> p
}
check M accepts "", "aa"
check M rejects "a"
`;

test('Auto off: edits wait for Execute (Ctrl+Enter), with a stale banner meanwhile', async ({ page }, testInfo) => {
  const layout = layoutOf(testInfo);
  const auto = page.getByRole('switch', { name: 'Auto' });
  await expect(auto).toHaveAttribute('aria-checked', 'true');
  await auto.click();
  await expect(auto).toHaveAttribute('aria-checked', 'false');

  await setCode(page, SMALL_DFA);
  await expect(page.getByTestId('stale-dot')).toBeVisible();
  if (layout === 'phone') await page.getByRole('tab', { name: 'Diagram' }).click();
  await expect(page.getByTestId('stale-banner')).toContainText('Code changed — press');
  // The old diagram stays until Execute.
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^DFA EndsAB,/);

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
  const arrows = Array.from({ length: k - 1 }, (_, i) => `  q${i + 1} -[0,1]-> q${i + 2}`).join('\n');
  const lets = Array.from({ length: 10 }, (_, i) => `let D${i} = determinize(N)`).join('\n');
  return `nfa N {\n  states ${states}\n  alphabet 0, 1\n  start q0\n  accept q${k}\n  q0 -[0,1]-> q0\n  q0 -1-> q1\n${arrows}\n}\n${lets}\n`;
}

test('a slow evaluation shows the loader, and Stop cancels it', async ({ page }, testInfo) => {
  test.skip(layoutOf(testInfo) === 'phone', 'The loader sits under the diagram header');
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



// with one box per sub-expression and dashed ε-arrows, and no DFA unless the code asks for one.
test('the Lang view shows a language’s class and sample words', async ({ page }, testInfo) => {
  const layout = layoutOf(testInfo);
  await pickObject(page, layout, 'L');
  await openView(page, layout, 'Lang');
  await expect(page.getByTestId('lang-class')).toHaveText('Decidable');
  await expect(page.getByRole('list', { name: 'Class lattice' })).toContainText('Regular');
  await expect(page.getByRole('region', { name: 'Language' })).toContainText('ab');
});

/** A share link's `code=` value, computed in the page (deflate-raw, base64url). */
async function shareCode(page: Page, text: string): Promise<string> {
  return page.evaluate(async (src) => {
    const stream = new Blob([new TextEncoder().encode(src)]).stream().pipeThrough(new CompressionStream('deflate-raw'));
    const bytes = new Uint8Array(await new Response(stream).arrayBuffer());
    let bin = '';
    for (const b of bytes) bin += String.fromCharCode(b);
    return btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  }, text);
}

test('a share link opens its program, and embed mode shows the embeddable Pad', async ({ page }, testInfo) => {
  const layout = layoutOf(testInfo);
  const code = await shareCode(page, SMALL_DFA);
  await page.goto(`/#code=${code}`);
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toContainText('dfa M');
  if (layout === 'phone') await page.getByRole('tab', { name: 'Diagram' }).click();
  await expect(page.getByTestId('diagram')).toHaveAttribute('aria-label', /^DFA M, 2 states/);

  await page.goto(`/#code=${code}&embed=1&panel=tests`);
  await page.reload();
  await expect(page.locator('[data-layout="embed"]')).toBeVisible();
  await expect(page.getByTestId('tests-summary')).toHaveText('3 of 3 passed');
});

test('the Pad is installable and works offline after one visit', async ({ page, context }, testInfo) => {
  test.skip(layoutOf(testInfo) !== 'desktop', 'One offline check is enough');
  const manifest = await page.request.get('/manifest.webmanifest');
  expect(manifest.ok()).toBe(true);
  expect((await manifest.json()).icons).toHaveLength(3);
  // Wait for the service worker to control the page and for the engine to have loaded once.
  await page.evaluate(async () => {
    await navigator.serviceWorker.ready;
  });
  await page.reload();
  await expect(page.getByTestId('error-count')).toBeVisible();
  await expect(page.getByTestId('executed')).toHaveText(/^Executed in \d+ ms$/);
  await context.setOffline(true);
  await page.reload();
  await expect(page.getByRole('textbox', { name: 'Delta code' })).toContainText('dfa EndsAB');
  await expect(page.getByTestId('executed')).toHaveText(/^Executed in \d+ ms$/);
  await context.setOffline(false);
});
