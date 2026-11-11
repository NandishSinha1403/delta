// What the engine is asked to run for the active file. A Delta file runs as it is, with every open
// JFLAP file (.jff) available to `import "x.jff"` by its tab name. A .jff tab runs a two-line
// program that imports and shows it, so opening a JFLAP file shows its machine.
import { useUiStore } from './ui';

export interface Program {
  src: string;
  /** Open .jff files by tab name, for `import`. */
  files: Record<string, string>;
  /** The active file is a .jff, so `src` was generated (its offsets aren't the editor's). */
  generated: boolean;
}

/** True for a JFLAP file's name. */
export function isJflap(name: string): boolean {
  return /\.jff$/i.test(name);
}

/** A Delta name for a file stem: letters, digits and _, starting with a letter. */
export function nameForFile(file: string): string {
  const stem = file.replace(/\.[^.]*$/, '').replace(/[^A-Za-z0-9_]/g, '_');
  return /^[A-Za-z_]/.test(stem) ? stem : `M_${stem}`;
}

/** The program for `source`, the active file's text. */
export function programOf(source: string): Program {
  const { files, activeId } = useUiStore.getState();
  const open: Record<string, string> = {};
  for (const f of files) if (isJflap(f.name)) open[f.name] = f.source;
  const active = files.find((f) => f.id === activeId);
  if (active && isJflap(active.name)) {
    open[active.name] = source;
    const name = nameForFile(active.name);
    return { src: `import "${active.name}" as ${name}\nshow ${name}\n`, files: open, generated: true };
  }
  return { src: source, files: open, generated: false };
}
