// The front page and docs (03-PRODUCTS.md §1, §2, §4). The Pad is built separately (web/) and
// served at /pad/; every Delta code block here links to it.
import { unified } from '@astrojs/markdown-remark';
import starlight from '@astrojs/starlight';
import { defineConfig } from 'astro/config';
import { beaconScript } from './src/lib/analytics.mjs';
import { remarkOpenInPad } from './src/lib/pad-link.mjs';


export default defineConfig({
  site: 'https://delta.nandish.dev',
  markdown: { processor: unified({ remarkPlugins: [remarkOpenInPad] }) },
  integrations: [
    starlight({
      title: 'Delta',
      description: 'The language of computation: machines, languages and constructions as code.',
      logo: { src: './src/assets/logo.svg', replacesTitle: false },
      favicon: '/favicon.svg',
      social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/NandishSinha1403/delta' }],
      customCss: ['./src/styles/theme.css'],
      // JSON-LD, social cards and preview images for every page (src/route-data.ts).
      routeMiddleware: './src/route-data.ts',
      head: beaconScript ? [beaconScript] : [],
      components: {
        ThemeProvider: './src/components/DarkTheme.astro',
        ThemeSelect: './src/components/NoThemeSelect.astro',
      },
      sidebar: [
        { label: 'Start', items: [{ label: 'Docs home', link: '/docs/' }, { label: 'Open the Pad ↗', link: '/pad/' }, { label: 'Download deltac', link: '/download/' }] },
        { label: 'Tools', collapsed: true, items: [{ autogenerate: { directory: 'tools' } }] },
        { label: 'Learn', collapsed: true, items: [{ autogenerate: { directory: 'learn' } }] },
        { label: 'Practice', collapsed: true, items: [{ autogenerate: { directory: 'practice' } }] },
        { label: 'Tour', items: [{ autogenerate: { directory: 'docs/tour' } }] },
        { label: 'Reference', items: ['docs/library', 'docs/models', 'docs/languages', 'docs/notation'] },
        { label: 'Cookbook', items: [{ autogenerate: { directory: 'docs/cookbook' } }] },
        { label: 'Errors', items: [{ autogenerate: { directory: 'docs/errors' } }] },
        { label: 'Terminal', items: ['docs/terminal'] },
        { label: 'AI', items: ['docs/ai'] },
      ],
    }),
  ],
});
