// A few line icons drawn inline (no icon library): 18 × 18, stroke in the current text color.
const PATHS = {
  files: 'M5 3h6l4 4v11H5z M11 3v4h4',
  book: 'M4 4h5a3 3 0 0 1 3 3v11a2 2 0 0 0-2-2H4z M16 4h-2a2 2 0 0 0-2 2 M16 4v12h-4',
  convert: 'M4 7h11l-3-3 M16 13H5l3 3',
  tests: 'M4 4h12v12H4z M7 10l2 2 4-4',
  settings:
    'M10 7a3 3 0 1 0 0 6 3 3 0 0 0 0-6z M10 2v2 M10 16v2 M2 10h2 M16 10h2 M4.3 4.3l1.4 1.4 M14.3 14.3l1.4 1.4 M4.3 15.7l1.4-1.4 M14.3 5.7l1.4-1.4',
  search: 'M8.5 3a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11z M12.5 12.5L17 17',
  moon: 'M15 12.5A6.5 6.5 0 0 1 7.5 5a6.5 6.5 0 1 0 7.5 7.5z',
  sun: 'M10 6a4 4 0 1 0 0 8 4 4 0 0 0 0-8z M10 1.5v2 M10 16.5v2 M1.5 10h2 M16.5 10h2 M4 4l1.4 1.4 M14.6 14.6L16 16 M4 16l1.4-1.4 M14.6 5.4L16 4',
} as const;

export type IconName = keyof typeof PATHS;

export function Icon({ name, size = 18 }: { name: IconName; size?: number }) {
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 20 20"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.6}
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d={PATHS[name]} />
    </svg>
  );
}
