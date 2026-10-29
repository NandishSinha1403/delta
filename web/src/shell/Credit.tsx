/** The author credit, shown in the status bar and the examples dialog. */
export function Credit({ className = '' }: { className?: string }) {
  return (
    <span className={className}>
      A project by{' '}
      <a
        href="https://github.com/NandishSinha1403"
        target="_blank"
        rel="noreferrer"
        className="font-medium text-text hover:text-accent hover:underline"
      >
        Nandish Sinha
      </a>
    </span>
  );
}
