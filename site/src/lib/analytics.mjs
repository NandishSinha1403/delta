// Cloudflare Web Analytics (no cookies, no personal data). The site token is public: it appears in
// every page's HTML. Empty = no snippet. Set it from Cloudflare → Analytics & Logs → Web Analytics.
export const BEACON_TOKEN = '';

export const beaconScript = BEACON_TOKEN
  ? {
      tag: 'script',
      attrs: {
        defer: true,
        src: 'https://static.cloudflareinsights.com/beacon.min.js',
        'data-cf-beacon': JSON.stringify({ token: BEACON_TOKEN }),
      },
    }
  : null;
