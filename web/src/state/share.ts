// Share links and embeds (03-PRODUCTS.md §1, §5): the program lives in the URL hash, compressed,
// so sharing needs no server. `#code=…` opens it in the Pad; `&embed=1` shows the embeddable Pad.

/** Base64url without padding. */
function toBase64Url(bytes: Uint8Array): string {
  let s = '';
  for (const b of bytes) s += String.fromCharCode(b);
  return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

function fromBase64Url(s: string): Uint8Array {
  const b64 = s.replace(/-/g, '+').replace(/_/g, '/');
  const bin = atob(b64 + '='.repeat((4 - (b64.length % 4)) % 4));
  return Uint8Array.from(bin, (c) => c.charCodeAt(0));
}

async function pipe(bytes: Uint8Array, stream: CompressionStream | DecompressionStream): Promise<Uint8Array> {
  const out = new Blob([bytes as BlobPart]).stream().pipeThrough(stream);
  return new Uint8Array(await new Response(out).arrayBuffer());
}

/** The program as a `code=` value: deflate, then base64url. */
export async function encodeSource(source: string): Promise<string> {
  const packed = await pipe(new TextEncoder().encode(source), new CompressionStream('deflate-raw'));
  return toBase64Url(packed);
}

export async function decodeSource(code: string): Promise<string> {
  const bytes = await pipe(fromBase64Url(code), new DecompressionStream('deflate-raw'));
  return new TextDecoder().decode(bytes);
}

export interface HashOptions {
  code: string | null;
  embed: boolean;
  /** Which output panel an embed shows: diagram, tests, console or lang. */
  panel: string | null;
  readonly: boolean;
}

export function parseHash(hash: string): HashOptions {
  const params = new URLSearchParams(hash.replace(/^#/, ''));
  return {
    code: params.get('code'),
    embed: params.get('embed') === '1',
    panel: params.get('panel'),
    readonly: params.get('readonly') === '1',
  };
}

/** The base URL of the Pad (this page, without its hash). */
function base(): string {
  return `${window.location.origin}${window.location.pathname}`;
}

export async function shareLink(source: string): Promise<string> {
  return `${base()}#code=${await encodeSource(source)}`;
}

export async function embedSnippet(source: string): Promise<string> {
  const url = `${base()}#code=${await encodeSource(source)}&embed=1&panel=diagram&run=1`;
  return `<iframe src="${url}" width="100%" height="420" loading="lazy" title="Delta example"></iframe>`;
}
