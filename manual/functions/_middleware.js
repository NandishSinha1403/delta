// Password gate for the private manual (a Cloudflare Pages Function that runs before every
// request). The password is the Pages secret MANUAL_PASS, never in the repo.
// - Without a valid cookie: a small password page (401). POST the right password → a 30-day
//   Secure, HttpOnly cookie and a redirect back to the page asked for. Wrong → the page again.
// - The cookie holds HMAC-SHA256(MANUAL_PASS, "delta-manual/v1"), not the password, so changing
//   the secret signs everyone out. Comparisons are constant-time (on SHA-256 digests).
// - No secret configured → fail closed (500), never open.

const COOKIE = 'delta_manual';
const MAX_AGE = 30 * 24 * 60 * 60;
// Served without a password: the login page's fonts and icon, and robots.txt (Disallow: /).
const PUBLIC = [/^\/fonts\//, /^\/favicon\.svg$/, /^\/robots\.txt$/];
const NOINDEX = 'noindex, nofollow, noarchive';

const enc = new TextEncoder();

async function sha256(text) {
  return new Uint8Array(await crypto.subtle.digest('SHA-256', enc.encode(text)));
}

/** Constant-time string equality: compares fixed-length digests, every byte, always. */
export async function same(a, b) {
  const [x, y] = await Promise.all([sha256(a), sha256(b)]);
  let diff = 0;
  for (let i = 0; i < x.length; i++) diff |= x[i] ^ y[i];
  return diff === 0;
}

/** The cookie value for a password. */
export async function token(pass) {
  const key = await crypto.subtle.importKey('raw', enc.encode(pass), { name: 'HMAC', hash: 'SHA-256' }, false, ['sign']);
  const mac = new Uint8Array(await crypto.subtle.sign('HMAC', key, enc.encode('delta-manual/v1')));
  return [...mac].map((b) => b.toString(16).padStart(2, '0')).join('');
}

function cookieValue(request) {
  const header = request.headers.get('Cookie') ?? '';
  for (const part of header.split(';')) {
    const [k, ...v] = part.trim().split('=');
    if (k === COOKIE) return v.join('=');
  }
  return '';
}

/** Only same-site paths, so the redirect after login can't send anyone elsewhere. */
function safePath(p) {
  return typeof p === 'string' && p.startsWith('/') && !p.startsWith('//') && !p.includes('\\') ? p : '/';
}

function esc(s) {
  return s.replace(/&/g, '&amp;').replace(/"/g, '&quot;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function page(status, wrong, next) {
  const html = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="robots" content="${NOINDEX}">
<title>Delta manual</title>
<link rel="icon" href="/favicon.svg">
<style>
@font-face { font-family: 'Schibsted Grotesk'; font-weight: 400 900; font-display: swap; src: url('/fonts/schibsted-grotesk-latin-wght-normal.woff2') format('woff2-variations'); }
@font-face { font-family: 'Commit Mono'; font-weight: 400; font-display: swap; src: url('/fonts/commit-mono-latin-400-normal.woff2') format('woff2'); }
*, *::before, *::after { box-sizing: border-box; }
html, body { margin: 0; height: 100%; background: #000; color: #ededed; }
body { display: grid; place-items: center; padding: 20px; font: 16px/1.6 'Schibsted Grotesk', system-ui, sans-serif; }
form { width: 100%; max-width: 360px; padding: 28px; background: #0a0a0a; border-radius: 12px; box-shadow: 0 0 0 1px rgba(255,255,255,.145); }
.mark { display: block; margin-bottom: 18px; }
h1 { margin: 0 0 4px; font-weight: 500; font-size: 28px; letter-spacing: -0.04em; color: #fff; }
p { margin: 0 0 20px; color: #999; font-size: 14px; }
label { display: block; margin-bottom: 6px; font: 12px 'Commit Mono', ui-monospace, monospace; text-transform: uppercase; color: #999; }
input { width: 100%; padding: 10px 12px; background: #000; color: #fff; border: 0; border-radius: 0; box-shadow: 0 0 0 1px rgba(255,255,255,.145); font: 15px 'Commit Mono', ui-monospace, monospace; }
input:focus { outline: none; box-shadow: 0 0 0 1px #52a8ff; }
button { margin-top: 16px; width: 100%; padding: 10px 12px; background: #fff; color: #121212; border: 0; border-radius: 0; font: 500 15px 'Schibsted Grotesk', system-ui, sans-serif; cursor: pointer; }
button:hover { background: #e6e6e6; box-shadow: 0 0 0 4px rgba(255,255,255,.08); }
.wrong { margin: 12px 0 0; display: inline-flex; gap: 6px; align-items: center; padding: 4px 10px; background: #1f1f1f; border-radius: 100px; font: 12px 'Commit Mono', ui-monospace, monospace; text-transform: uppercase; color: #ededed; }
.wrong::before { content: ''; width: 6px; height: 6px; border-radius: 50%; background: #ededed; }
</style>
</head>
<body>
<form method="post" action="${esc(next)}">
<svg class="mark" width="40" height="24" viewBox="0 0 40 24" aria-hidden="true"><path d="M2 12h14" stroke="#fff" stroke-width="1.5"/><path d="M13 8l4 4-4 4" fill="none" stroke="#fff" stroke-width="1.5"/><circle cx="28" cy="12" r="10" fill="none" stroke="#fff" stroke-width="1.5"/><circle cx="28" cy="12" r="6.5" fill="none" stroke="#fff" stroke-width="1.5"/></svg>
<h1>Delta manual</h1>
<p>Private. Enter the password to continue.</p>
<label for="password">Password</label>
<input id="password" name="password" type="password" autocomplete="current-password" autofocus required>
${wrong ? '<span class="wrong" role="alert">Wrong password</span>' : ''}
<button type="submit">Open the manual</button>
</form>
</body>
</html>`;
  return new Response(html, {
    status,
    headers: {
      'Content-Type': 'text/html; charset=utf-8',
      'Cache-Control': 'no-store',
      'X-Robots-Tag': NOINDEX,
      'Referrer-Policy': 'no-referrer',
      'X-Frame-Options': 'DENY',
    },
  });
}

export async function onRequest(context) {
  const { request, env, next } = context;
  const url = new URL(request.url);
  if (PUBLIC.some((re) => re.test(url.pathname))) return next();

  const pass = env.MANUAL_PASS;
  if (typeof pass !== 'string' || pass.length === 0) {
    return new Response('The manual is not configured.', { status: 500, headers: { 'Cache-Control': 'no-store', 'X-Robots-Tag': NOINDEX } });
  }
  const expected = await token(pass);

  if (await same(cookieValue(request), expected)) {
    const res = await next();
    const out = new Response(res.body, res);
    out.headers.set('X-Robots-Tag', NOINDEX);
    out.headers.set('Cache-Control', 'private, no-store');
    return out;
  }

  const here = safePath(url.pathname + url.search);
  if (request.method === 'POST') {
    const form = await request.formData().catch(() => null);
    const given = form?.get('password');
    if (typeof given === 'string' && (await same(given, pass))) {
      return new Response(null, {
        status: 303,
        headers: {
          Location: here,
          'Set-Cookie': `${COOKIE}=${expected}; Max-Age=${MAX_AGE}; Path=/; Secure; HttpOnly; SameSite=Lax`,
          'Cache-Control': 'no-store',
        },
      });
    }
    return page(401, true, here);
  }
  return page(401, false, here);
}
