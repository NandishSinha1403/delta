// SEO checks on the built site (04-ROADMAP.md, Phase 5): every page has a unique <title>, a
// unique meta description and a canonical URL; JSON-LD parses; robots.txt and the sitemap exist.
// Run after `astro build`: node scripts/check-seo.mjs [dist]
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

const dist = process.argv[2] ?? 'dist';
const pages = [];
const walk = (dir) => {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    // The Pad is an app, not a page to index; 404.html is not indexed either.
    if (p === join(dist, 'pad')) continue;
    if (statSync(p).isDirectory()) walk(p);
    else if (name === 'index.html') pages.push(p);
  }
};
walk(dist);

const errors = [];
const seen = { title: new Map(), description: new Map(), canonical: new Map() };
const attr = (html, re) => html.match(re)?.[1]?.trim();
for (const page of pages) {
  const html = readFileSync(page, 'utf8');
  const found = {
    title: attr(html, /<title>([^<]*)<\/title>/),
    description: attr(html, /<meta name="description" content="([^"]*)"/),
    canonical: attr(html, /<link rel="canonical" href="([^"]*)"/),
  };
  for (const [key, value] of Object.entries(found)) {
    if (!value) errors.push(`${page}: no ${key}`);
    else if (seen[key].has(value)) errors.push(`${page}: same ${key} as ${seen[key].get(value)}: ${value}`);
    else seen[key].set(value, page);
  }
  for (const m of html.matchAll(/<script type="application\/ld\+json">([^<]*)<\/script>/g)) {
    try {
      JSON.parse(m[1]);
    } catch {
      errors.push(`${page}: JSON-LD does not parse`);
    }
  }
}
for (const f of ['robots.txt', 'sitemap-index.xml']) {
  if (!existsSync(join(dist, f))) errors.push(`missing ${f}`);
}

if (errors.length > 0) {
  console.error(errors.join('\n'));
  console.error(`SEO check: ${errors.length} problem(s) in ${pages.length} pages`);
  process.exit(1);
}
console.log(`SEO check: ${pages.length} pages, titles, descriptions and canonicals all unique`);
