import { useState } from 'react';
import { usePresentStore } from '../state/present';
import { embedSnippet, presentLink, shareLink } from '../state/share';
import { useUiStore } from '../state/ui';

/** Share: copy a link that holds the program, or an `<iframe>` embed of it. */
export function ShareButton() {
  const [open, setOpen] = useState(false);
  const [copied, setCopied] = useState<string | null>(null);
  const copy = async (what: 'link' | 'embed' | 'present') => {
    const source = useUiStore.getState().source;
    const text =
      what === 'link'
        ? await shareLink(source)
        : what === 'present'
          ? await presentLink(source, usePresentStore.getState().what)
          : await embedSnippet(source);
    try {
      await navigator.clipboard.writeText(text);
      setCopied(what === 'embed' ? 'Embed copied' : 'Link copied');
    } catch {
      window.prompt('Copy this:', text);
    }
    if (what === 'link') window.history.replaceState(null, '', text);
    window.setTimeout(() => setCopied(null), 1600);
    setOpen(false);
  };
  return (
    <div className="relative shrink-0">
      <button
        type="button"
        aria-haspopup="menu"
        aria-expanded={open}
        onClick={() => setOpen(!open)}
        className="h-8 rounded-md border border-border-strong px-3 text-xs text-text hover:bg-surface-2"
      >
        {copied ?? 'Share'}
      </button>
      {open && (
        <div role="menu" className="absolute right-0 z-30 mt-1 w-44 rounded-md border border-border bg-surface p-1 shadow-lg">
          <button type="button" role="menuitem" onClick={() => void copy('link')} className="block w-full rounded px-2 py-1.5 text-left text-xs hover:bg-surface-2">
            Copy link
          </button>
          <button type="button" role="menuitem" onClick={() => void copy('embed')} className="block w-full rounded px-2 py-1.5 text-left text-xs hover:bg-surface-2">
            Copy embed
          </button>
          <button type="button" role="menuitem" onClick={() => void copy('present')} className="block w-full rounded px-2 py-1.5 text-left text-xs hover:bg-surface-2">
            Copy presentation link
          </button>
        </div>
      )}
    </div>
  );
}

/** "explain off" or the profile in use, from the program's `use explain "…"`. */
export function ProfileBadge() {
  const source = useUiStore((s) => s.source);
  const explain = /use\s+explain\s+"([^"]+)"/.exec(source)?.[1];
  const notation = /use\s+notation\s+"([^"]+)"/.exec(source)?.[1] ?? 'default';
  return (
    <span
      data-testid="profile-badge"
      className="hidden shrink-0 rounded-full border border-border-strong px-2 py-1 font-mono text-[11px] text-text-2 lg:inline"
    >
      notation {notation} · explain {explain ?? 'off'}
    </span>
  );
}
