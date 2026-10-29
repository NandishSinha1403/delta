interface Props {
  title: string;
  children: React.ReactNode;
}

/** Placeholder for a view that arrives in a later phase. */
export function EmptyPanel({ title, children }: Props) {
  return (
    <div className="flex h-full min-h-24 flex-col items-center justify-center gap-1 p-6 text-center">
      <p className="text-sm font-medium text-text">{title}</p>
      <p className="max-w-xs text-sm text-muted">{children}</p>
    </div>
  );
}
