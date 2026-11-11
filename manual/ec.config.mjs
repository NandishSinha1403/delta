// Code blocks look like the public docs' (same grammar and theme, read from site/).
import { defineEcConfig, ExpressiveCodeTheme } from '@astrojs/starlight/expressive-code';
import { readFileSync } from 'node:fs';

const grammar = JSON.parse(readFileSync(new URL('../site/src/lib/delta.tmLanguage.json', import.meta.url), 'utf8'));
const theme = ExpressiveCodeTheme.fromJSONString(
  readFileSync(new URL('../site/src/lib/delta-dark.theme.json', import.meta.url), 'utf8'),
);

export default defineEcConfig({
  themes: [theme],
  shiki: { langs: [grammar] },
  styleOverrides: {
    borderRadius: '12px',
    borderColor: 'rgba(255, 255, 255, 0.145)',
    codeFontFamily: "'IBM Plex Mono', 'Delta Math', 'Delta Sup', ui-monospace, Menlo, monospace",
    codeFontSize: '13px',
    codeBackground: '#0a0a0a',
  },
});
