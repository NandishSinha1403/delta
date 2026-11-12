// "Open file…": the browser's file picker, then the file opens in a tab. Nothing is uploaded; the
// file is read here and kept in this browser like every other tab.
import { useUiStore } from './ui';

/** Asks for a .delta or .jff file and opens it. Resolves when the file is open (or none was picked). */
export function openFromDevice(): Promise<void> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.delta,.jff,text/plain,application/xml';
    input.addEventListener('change', () => {
      const file = input.files?.[0];
      if (!file) return resolve();
      void file.text().then((text) => {
        useUiStore.getState().openLocal(file.name, text);
        resolve();
      });
    });
    input.addEventListener('cancel', () => resolve());
    input.click();
  });
}
