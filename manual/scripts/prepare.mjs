// Builds src/content/docs/ from chapters/ (MANUAL-TEMPLATE.md §8, "Keeping it true"):
// - `<!-- excerpt: PATH#NAME -->` becomes the code between `// manual:begin NAME` and
//   `// manual:end NAME` in PATH, with its file, lines and commit. A missing marker fails.
// - `<!-- generated: deltac-help -->` and `<!-- generated: error-codes -->` come from the code.
// - `{{commit}}` and `{{date}}` are the commit described and today's date (DD MMM YYYY).
// - Any secret-shaped string, or the owner's email, fails the build.
import { execFileSync } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const manual = join(here, '..');
const repo = join(manual, '..');
const src = join(manual, 'chapters');
const out = join(manual, 'src/content/docs');

const git = (...args) => execFileSync('git', args, { cwd: repo, encoding: 'utf8' }).trim();
const dirty = git('status', '--porcelain', '--', 'crates', 'web/src', 'manual/chapters') !== '';
const COMMIT = git('rev-parse', '--short', 'HEAD') + (dirty ? '+changes' : '');
const MONTHS = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
const now = new Date();
const DATE = `${String(now.getDate()).padStart(2, '0')} ${MONTHS[now.getMonth()]} ${now.getFullYear()}`;

const errors = [];
const LANG = { rs: 'rust', ts: 'ts', tsx: 'tsx', mjs: 'js', js: 'js', delta: 'delta', sh: 'sh', toml: 'toml' };

function excerpt(path, name) {
  const file = join(repo, path);
  if (!existsSync(file)) {
    errors.push(`excerpt: no file ${path}`);
    return '';
  }
  const lines = readFileSync(file, 'utf8').split('\n');
  // `// manual:begin NAME`, or `{/* manual:begin NAME */}` inside JSX.
  const is = (l, kind) => [`// manual:${kind} ${name}`, `{/* manual:${kind} ${name} */}`].includes(l.trim());
  const begin = lines.findIndex((l) => is(l, 'begin'));
  const end = lines.findIndex((l, i) => i > begin && is(l, 'end'));
  if (begin < 0 || end < 0) {
    errors.push(`excerpt: no marker "${name}" in ${path}`);
    return '';
  }
  const body = lines.slice(begin + 1, end).filter((l) => !/manual:(begin|end) /.test(l));
  const indent = Math.min(...body.filter((l) => l.trim()).map((l) => l.length - l.trimStart().length));
  const code = body.map((l) => l.slice(indent)).join('\n');
  const ext = path.split('.').pop() ?? '';
  const title = `${path} · lines ${begin + 2}–${end} · ${COMMIT}`;
  return '```' + (LANG[ext] ?? '') + ` title="${title}"\n${code}\n` + '```';
}

function generated(kind) {
  if (kind === 'deltac-help') {
    const help = execFileSync('cargo', ['run', '-q', '-p', 'delta-cli', '--', '--help'], { cwd: repo, encoding: 'utf8' });
    return '<!-- Generated from `deltac --help`; do not edit. -->\n\n```text\n' + help.trim() + '\n```';
  }
  if (kind === 'error-codes') {
    const index = readFileSync(join(repo, 'site/src/content/docs/docs/errors/index.md'), 'utf8');
    const rows = [...index.matchAll(/^- \[([A-Z]\d{3})\]\([^)]*\): (.*)$/gm)].map((m) => `| ${m[1]} | ${m[2]} |`);
    return '<!-- Generated from site/src/content/docs/docs/errors/; do not edit. -->\n\n| Code | Meaning |\n|---|---|\n' + rows.join('\n');
  }
  errors.push(`generated: unknown section "${kind}"`);
  return '';
}

// Secret-shaped strings (MANUAL-TEMPLATE §9). The account id is 32 hex digits; tokens are long.
const SECRETS = [
  [/\b[0-9a-f]{32}\b/, 'a 32-hex-digit id (account ids, tokens)'],
  [/Bearer\s+[A-Za-z0-9._~+/-]{20,}/, 'a bearer token'],
  [/\b(gh[pousr]_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,}|AKIA[0-9A-Z]{16})\b/, 'an API key'],
  [/(api[_-]?key|token|secret|password)\s*[:=]\s*["']?[A-Za-z0-9._-]{16,}/i, 'a credential assignment'],
  [/[A-Za-z0-9._%+-]+@(gmail|outlook|yahoo|icloud|proton)\.[a-z]+/i, 'a personal email address'],
];

function scan(name, text) {
  for (const [re, what] of SECRETS) {
    const m = text.match(re);
    if (m) errors.push(`${name}: looks like ${what}: "${m[0].slice(0, 12)}…"`);
  }
}

function walk(dir) {
  return readdirSync(dir).flatMap((f) => {
    const p = join(dir, f);
    return statSync(p).isDirectory() ? walk(p) : [p];
  });
}

rmSync(out, { recursive: true, force: true });
for (const file of walk(src).filter((f) => f.endsWith('.md'))) {
  const rel = relative(src, file);
  let text = readFileSync(file, 'utf8');
  text = text.replace(/<!-- excerpt: ([^#\s]+)#([\w-]+) -->/g, (_, p, n) => excerpt(p, n));
  text = text.replace(/<!-- generated: ([\w-]+) -->/g, (_, k) => generated(k));
  text = text.replaceAll('{{commit}}', COMMIT).replaceAll('{{date}}', DATE);
  scan(rel, text);
  mkdirSync(dirname(join(out, rel)), { recursive: true });
  writeFileSync(join(out, rel), text);
}
writeFileSync(join(manual, 'src/build-info.mjs'), `export const COMMIT = ${JSON.stringify(COMMIT)};\nexport const UPDATED = ${JSON.stringify(DATE)};\n`);
// The site's self-hosted fonts (OFL), so the manual makes no outside requests.
cpSync(join(repo, 'site/public/fonts'), join(manual, 'public/fonts'), { recursive: true });

if (errors.length) {
  console.error(errors.map((e) => `manual: ${e}`).join('\n'));
  process.exit(1);
}
console.log(`manual: ${walk(out).length} pages from chapters/, commit ${COMMIT}`);
