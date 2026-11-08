// Delta's service worker (03-PRODUCTS.md §6.1, PWA): the Pad works offline after one visit.
// Same-origin GETs are served from the cache when the network fails, and refreshed when it works.
const CACHE = 'delta-v2';

self.addEventListener('install', (event) => {
  event.waitUntil(caches.open(CACHE).then((c) => c.addAll(['./', './manifest.webmanifest', './favicon.svg'])));
  self.skipWaiting();
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim()),
  );
});

self.addEventListener('fetch', (event) => {
  const req = event.request;
  if (req.method !== 'GET' || new URL(req.url).origin !== self.location.origin) return;
  event.respondWith(
    fetch(req)
      .then((res) => {
        if (res.ok) {
          const copy = res.clone();
          caches.open(CACHE).then((c) => c.put(req.mode === 'navigate' ? './' : req, copy));
        }
        return res;
      })
      .catch(() => caches.match(req.mode === 'navigate' ? './' : req).then((r) => r ?? Response.error())),
  );
});
