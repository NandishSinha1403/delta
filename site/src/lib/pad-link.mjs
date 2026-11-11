// Pad links: the program deflated (raw) and base64url-encoded in the URL hash, exactly as the
// Pad's own share links (web/src/state/share.ts) encode it.
import { deflateRawSync } from 'node:zlib';

export function padCode(source) {
  return deflateRawSync(Buffer.from(source, 'utf8')).toString('base64url');
}

export function padUrl(source, extra = '') {
  return `/pad/#code=${padCode(source)}${extra}`;
}

/** remark plugin: an "Open in the Pad" link after every runnable ```delta block (and a live Pad
 * after a ```delta embed block). */
export function remarkOpenInPad() {
  return (tree) => {
    const walk = (node) => {
      if (!node.children) return;
      for (let i = 0; i < node.children.length; i++) {
        const child = node.children[i];
        if (child.type === 'code' && child.lang === 'delta') {
          const meta = child.meta ?? '';
          const label = /error|outcome/.test(meta) ? 'Try it in the Pad' : 'Open in the Pad';
          const added = [
            {
              type: 'html',
              value: `<p class="open-pad"><a href="${padUrl(child.value)}">${label} ↗</a></p>`,
            },
          ];
          // ```delta embed: the same program, live, in an embedded Pad (loaded when scrolled to).
          if (/\bembed\b/.test(meta)) {
            added.unshift({
              type: 'html',
              value: `<div class="pad-embed"><iframe title="The program above, live in the Delta Pad" src="${padUrl(child.value, '&embed=1&panel=diagram&run=1')}" loading="lazy"></iframe></div>`,
            });
          }
          node.children.splice(i + 1, 0, ...added);
          child.meta = null;
          i += added.length;
        } else {
          walk(child);
        }
      }
    };
    walk(tree);
  };
}
