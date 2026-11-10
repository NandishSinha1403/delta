// The private Delta manual (04-ROADMAP.md, Phase 5c). Its own Starlight site, built from
// chapters/ by scripts/prepare.mjs, deployed to the Pages project `delta-manual` behind
// Cloudflare Access. Never linked from the public site, never indexed, no analytics.
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';

export default defineConfig({
  site: 'https://delta-manual.pages.dev',
  integrations: [
    starlight({
      title: 'Delta manual',
      description: 'How Delta works, how to run and change it, and how to explain it.',
      favicon: '/favicon.svg',
      customCss: ['../site/src/styles/fonts.css', '../site/src/styles/theme.css', './src/manual.css'],
      head: [{ tag: 'meta', attrs: { name: 'robots', content: 'noindex, nofollow, noarchive' } }],
      pagination: true,
      lastUpdated: false,
      components: {
        ThemeProvider: './src/components/DarkTheme.astro',
        ThemeSelect: './src/components/NoThemeSelect.astro',
        Footer: './src/components/Footer.astro',
      },
      sidebar: [
        { label: 'Start', items: ['index', 'pitches'] },
        {
          label: 'The manual',
          items: [
            'idea', 'quick-start', 'life-of-a-program', 'parts', 'data', 'using', 'operating',
            'changing', 'rules', 'security', 'limits', 'next', 'credits',
          ],
        },
        { label: 'Theory deep dives', items: [{ autogenerate: { directory: 'theory' } }] },
        { label: 'For interviews', items: ['language-design', 'architecture', 'war-stories'] },
        { label: 'Appendices', items: ['appendices'] },
      ],
    }),
  ],
});
