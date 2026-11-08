// SEO for every docs page (04-ROADMAP.md, Phase 5): social cards, a preview image when one was
// generated, and JSON-LD built from the page itself (its title, description and its visible FAQ),
// so nothing is written twice.
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { defineRouteMiddleware } from '@astrojs/starlight/route-data';

const SITE = 'https://delta.nandish.dev';

/** Questions and answers under a "## Frequently asked questions" heading (one `###` per question). */
export function faq(body: string): { q: string; a: string }[] {
  const start = body.search(/^## Frequently asked questions\s*$/m);
  if (start < 0) return [];
  const rest = body.slice(start).split('\n').slice(1).join('\n');
  const end = rest.search(/^## /m);
  const section = end < 0 ? rest : rest.slice(0, end);
  return section
    .split(/^### /m)
    .slice(1)
    .map((chunk) => {
      const [q = '', ...a] = chunk.split('\n');
      return { q: plain(q), a: plain(a.join(' ')) };
    })
    .filter((x) => x.q && x.a);
}

/** Markdown to plain text, enough for JSON-LD: links keep their text, code keeps its content. */
function plain(md: string): string {
  return md
    .replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
    .replace(/[`*_]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

export const onRequest = defineRouteMiddleware((context) => {
  const route = context.locals.starlightRoute;
  const { id, entry } = route;
  const url = new URL(context.url.pathname, SITE).href;
  const title = entry.data.title;
  const description = entry.data.description ?? '';
  // Set a <meta>, replacing one Starlight already added with the same name or property.
  const meta = (attrs: Record<string, string>) => {
    const key = attrs.name ? 'name' : 'property';
    const i = route.head.findIndex((h) => h.tag === 'meta' && h.attrs?.[key] === attrs[key]);
    if (i >= 0) route.head.splice(i, 1);
    route.head.push({ tag: 'meta', attrs, content: '' });
  };

  // A generated preview (the page's diagram) if there is one, else the site's card.
  const slug = id.replace(/\/$/, '') || 'index';
  const og = existsSync(join(process.cwd(), 'public', 'og', `${slug}.png`)) ? `/og/${slug}.png` : '/og/default.png';
  const hasCard = existsSync(join(process.cwd(), 'public', og));
  meta({ name: 'twitter:card', content: hasCard ? 'summary_large_image' : 'summary' });
  meta({ name: 'twitter:title', content: title });
  if (description) meta({ name: 'twitter:description', content: description });
  if (hasCard) {
    meta({ property: 'og:image', content: new URL(og, SITE).href });
    meta({ property: 'og:image:width', content: '1200' });
    meta({ property: 'og:image:height', content: '630' });
    meta({ name: 'twitter:image', content: new URL(og, SITE).href });
  }

  const section = id.split('/')[0];
  if (!['tools', 'learn', 'practice'].includes(section)) return;
  const graph: object[] = [
    {
      '@type': 'LearningResource',
      name: title,
      description,
      url,
      inLanguage: 'en',
      educationalLevel: 'University',
      learningResourceType: section === 'tools' ? 'Interactive tool' : section === 'practice' ? 'Solved problem' : 'Concept explanation',
      teaches: entry.data.title,
      about: 'Theory of computation',
      isAccessibleForFree: true,
      provider: { '@type': 'Organization', name: 'Delta', url: SITE },
    },
  ];
  const questions = faq(entry.body ?? '');
  if (questions.length > 0) {
    graph.push({
      '@type': 'FAQPage',
      mainEntity: questions.map(({ q, a }) => ({
        '@type': 'Question',
        name: q,
        acceptedAnswer: { '@type': 'Answer', text: a },
      })),
    });
  }
  route.head.push({
    tag: 'script',
    attrs: { type: 'application/ld+json' },
    content: JSON.stringify({ '@context': 'https://schema.org', '@graph': graph }),
  });
});
