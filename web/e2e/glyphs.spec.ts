// Callbacks passed to page.evaluate run in the browser, so this file needs the DOM types.
/// <reference lib="dom" />
import { readFileSync } from 'node:fs';
import { expect, test } from '@playwright/test';

// Every symbol Delta code can contain must be drawn by the Pad's own code fonts (IBM Plex Mono and
// its two Noto fallbacks), never by whatever the operating system happens to have. The list is
// shared with delta-export's test, so the Pad and the exports are checked against the same line.
const symbols = readFileSync(new URL('../../crates/delta-export/fonts/symbols.txt', import.meta.url), 'utf8')
  .split('\n')
  .filter((l) => l.trim() !== '' && !l.startsWith('#'))
  .join(' ');
const OURS = ['IBM Plex Mono', 'Noto Sans Math', 'Noto Sans Mono'];

test('the editor draws every Delta symbol with the bundled code fonts', async ({ page }, testInfo) => {
  // Fonts don't depend on the viewport: one layout is enough.
  test.skip(testInfo.project.name !== 'desktop', 'checked once, on desktop');
  await page.goto('/');
  const editor = page.getByRole('textbox', { name: 'Delta code' });
  await editor.click();
  await page.keyboard.press('ControlOrMeta+a');
  await page.keyboard.insertText(`// ${symbols}\n`);
  const line = page.locator('.cm-line').first();
  await expect(line).toContainText('⊔');
  await page.evaluate(() => document.fonts.ready);
  await expect.poll(() => page.evaluate(() => document.fonts.check('14px "Delta Sup"', 'ⁿ'))).toBe(true);

  // Chromium's DevTools protocol reports which actual font files drew a node's text.
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('DOM.enable');
  await cdp.send('CSS.enable');
  const { root } = await cdp.send('DOM.getDocument', { depth: -1 });
  const { nodeId } = await cdp.send('DOM.querySelector', { nodeId: root.nodeId, selector: '.cm-line' });
  const { fonts } = await cdp.send('CSS.getPlatformFontsForNode', { nodeId });
  const strangers = fonts.filter((f) => !f.isCustomFont || !OURS.includes(f.familyName));
  expect(strangers, `drawn with a system font: ${JSON.stringify(strangers)}`).toEqual([]);
  const glyphs = fonts.reduce((n, f) => n + f.glyphCount, 0);
  expect(glyphs).toBeGreaterThanOrEqual([...`// ${symbols}`].length);
  // The fallbacks really are used (Plex lacks Greek and the superscripts).
  expect(fonts.map((f) => f.familyName).sort()).toEqual([...OURS].sort());
});
