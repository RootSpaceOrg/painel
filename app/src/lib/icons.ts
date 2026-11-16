import { appIcon } from './api';

// Ícone do executável → data URL (PNG), uma vez por caminho. '' = sem ícone.
const cache = new Map<string, Promise<string>>();

export function iconFor(exe: string): Promise<string> {
  if (!exe) return Promise.resolve('');
  let p = cache.get(exe);
  if (!p) {
    p = appIcon(exe)
      .then(ic => {
        if (!ic) return '';
        const bytes = Uint8ClampedArray.from(atob(ic.rgba), c => c.charCodeAt(0));
        const c = document.createElement('canvas');
        c.width = ic.w;
        c.height = ic.h;
        c.getContext('2d')!.putImageData(new ImageData(bytes, ic.w, ic.h), 0, 0);
        return c.toDataURL();
      })
      .catch(() => '');
    cache.set(exe, p);
  }
  return p;
}
