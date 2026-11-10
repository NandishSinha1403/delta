// Tests the password gate (functions/_middleware.js) with Node's own Request/Response.
import assert from 'node:assert/strict';
import { onRequest, same, token } from '../functions/_middleware.js';

const env = { MANUAL_PASS: 'test-pass-123' };
const ok = new Response('<h1>manual</h1>', { headers: { 'Content-Type': 'text/html' } });
const call = (req, e = env) => onRequest({ request: req, env: e, next: async () => ok.clone() });

// No cookie: the password page, not the manual.
let res = await call(new Request('https://m.example/theory/01-subset/'));
assert.equal(res.status, 401);
let html = await res.text();
assert.match(html, /type="password"/);
assert.doesNotMatch(html, /<h1>manual<\/h1>/);
assert.equal(res.headers.get('X-Robots-Tag'), 'noindex, nofollow, noarchive');

// Wrong password: the page again, with the message, no cookie.
const post = (pw, path = '/theory/01-subset/') =>
  new Request('https://m.example' + path, { method: 'POST', body: new URLSearchParams({ password: pw }) });
res = await call(post('nope'));
assert.equal(res.status, 401);
assert.match(await res.text(), /Wrong password/);
assert.equal(res.headers.get('Set-Cookie'), null);

// Right password: a 30-day Secure, HttpOnly cookie that isn't the password, and a redirect back.
res = await call(post('test-pass-123'));
assert.equal(res.status, 303);
assert.equal(res.headers.get('Location'), '/theory/01-subset/');
const cookie = res.headers.get('Set-Cookie');
assert.match(cookie, /Max-Age=2592000/);
assert.match(cookie, /; Secure; HttpOnly; SameSite=Lax/);
assert.doesNotMatch(cookie, /test-pass-123/);

// With the cookie: the manual.
const value = cookie.split(';')[0];
res = await call(new Request('https://m.example/', { headers: { Cookie: `other=1; ${value}` } }));
assert.equal(res.status, 200);
assert.equal(await res.text(), '<h1>manual</h1>');
assert.equal(res.headers.get('X-Robots-Tag'), 'noindex, nofollow, noarchive');

// A forged or old cookie, or one for another password, is refused.
res = await call(new Request('https://m.example/', { headers: { Cookie: 'delta_manual=abc' } }));
assert.equal(res.status, 401);
res = await call(new Request('https://m.example/', { headers: { Cookie: `delta_manual=${await token('old')}` } }));
assert.equal(res.status, 401);

// Fonts, icon and robots.txt are public; nothing else is.
for (const p of ['/fonts/commit-mono-latin-400-normal.woff2', '/favicon.svg', '/robots.txt']) {
  assert.equal((await call(new Request('https://m.example' + p))).status, 200, p);
}
assert.equal((await call(new Request('https://m.example/sitemap-index.xml'))).status, 401);
assert.equal((await call(new Request('https://m.example/pagefind/pagefind.js'))).status, 401);

// No secret: fail closed.
assert.equal((await call(new Request('https://m.example/'), {})).status, 500);

// Redirect target stays on this site, and the form action is escaped.
res = await call(post('test-pass-123', '//evil.example/'));
assert.equal(res.headers.get('Location'), '/');
html = await (await call(new Request('https://m.example/a?x="><script>'))).text();
assert.doesNotMatch(html, /"><script>/);

assert.equal(await same('a', 'a'), true);
assert.equal(await same('a', 'ab'), false);
console.log('manual: password gate tests pass');
