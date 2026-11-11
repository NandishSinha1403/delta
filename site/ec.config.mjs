// Expressive Code (the docs' code blocks): Delta's grammar and a theme from the site palette.
import { defineEcConfig, ExpressiveCodeTheme } from '@astrojs/starlight/expressive-code';
import { readFileSync } from 'node:fs';

const grammar = JSON.parse(readFileSync(new URL('./src/lib/delta.tmLanguage.json', import.meta.url), 'utf8'));
const theme = ExpressiveCodeTheme.fromJSONString(readFileSync(new URL('./src/lib/delta-dark.theme.json', import.meta.url), 'utf8'));

export default defineEcConfig({
  themes: [theme],
  shiki: { langs: [grammar] },
  styleOverrides: {
    borderRadius: '12px',
    borderColor: 'rgba(255, 255, 255, 0.145)',
    codeFontFamily: "'Commit Mono', ui-monospace, Menlo, monospace",
    codeFontSize: '13.5px',
    codeBackground: '#0a0a0a',
  },
});
