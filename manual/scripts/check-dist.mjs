// After the build: every page says noindex, nothing loads from another origin, no page links
// to the public site's analytics, and no secret-shaped string made it into the HTML.
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';

const dist = new URL('../dist/', import.meta.url).pathname;
const walk = (d) => readdirSync(d).flatMap((f) => (statSync(join(d, f)).isDirectory() ? walk(join(d, f)) : [join(d, f)]));
const pages = walk(dist).filter((f) => f.endsWith('.html'));
const problems = [];
for (const p of pages) {
  const html = readFileSync(p, 'utf8');
  if (!/<meta name="robots" content="noindex, nofollow, noarchive"/.test(html)) problems.push(`${p}: no noindex meta`);
  // Things that load: scripts, images, iframes, and links other than canonical/alternate/sitemap.
  const tags = html.match(/<(script|img|iframe|link)\b[^>]*>/g) ?? [];
  const outside = tags.find(
    (t) => /(src|href)="(https?:)?\/\//.test(t) && !/rel="(canonical|alternate|sitemap)"/.test(t),
  );
  if (outside) problems.push(`${p}: loads from another origin: ${outside.slice(0, 80)}`);
  if (/cloudflareinsights|beacon\.min\.js/.test(html)) problems.push(`${p}: analytics beacon`);
  if (/\b[0-9a-f]{32}\b/.test(html.replace(/<[^>]+>/g, ' '))) problems.push(`${p}: 32-hex-digit string in text`);
}
if (problems.length) {
  console.error(problems.join('\n'));
  process.exit(1);
}
console.log(`manual: ${pages.length} pages checked (noindex, no outside requests, no secrets)`);
