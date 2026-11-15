import { spInit, spRefresh } from './spotify.svelte';
import { kvGet, kvSet, onMedia, onSys, mediaArt, wallpaperGet, type MediaState, type SysSample } from './api';

export type WidgetType = 'clock' | 'media' | 'todo' | 'habits' | 'agenda' | 'hardware' | 'mixer';
export interface Placed { id: string; type: WidgetType; x: number; y: number; w: number; h: number }
export interface Tab { id: string; name: string; widgets: Placed[] }

export const COLS = 12;
export const ROWS = 8;

const uid = () => Math.random().toString(36).slice(2, 9);
const w = (type: WidgetType, x: number, y: number, ww: number, h: number): Placed => ({ id: uid(), type, x, y, w: ww, h });

// Pensado para 1366x768: 12x8 células.
const defaultTabs = (): Tab[] => [
  {
    id: uid(), name: 'Início', widgets: [
      w('clock', 0, 0, 4, 3), w('media', 4, 0, 5, 3), w('mixer', 9, 0, 3, 3),
      w('todo', 0, 3, 4, 5), w('habits', 4, 3, 4, 5), w('agenda', 8, 3, 4, 5),
    ],
  },
  { id: uid(), name: 'Sistema', widgets: [w('hardware', 0, 0, 8, 8), w('mixer', 8, 0, 4, 5), w('clock', 8, 5, 4, 3)] },
];

export const ui = $state({
  tabs: defaultTabs(),
  active: 0,
  editing: false,
  expanded: null as Placed | null,
  settings: false,
});

export const prefs = $state({ dim: 0.35, blur: 18, seconds: false, wallUrl: '' });

export const live = $state({
  media: null as MediaState | null,
  art: '',
  artColor: '', // cor da capa, clareada para ler bem no escuro ('' = usa --teal)
  sys: null as SysSample | null,
  hist: { cpu: [] as number[], gpu: [] as number[], ram: [] as number[], rx: [] as number[], tx: [] as number[] },
});

export const newWidget = (type: WidgetType, x: number, y: number, ww: number, h: number) => w(type, x, y, ww, h);
export const newTab = (name: string): Tab => ({ id: uid(), name, widgets: [] });

export const saveLayout = () => kvSet('layout', JSON.stringify(ui.tabs));
export const savePrefs = () => kvSet('ui', JSON.stringify({ dim: prefs.dim, blur: prefs.blur, seconds: prefs.seconds }));

export async function loadWallpaper() {
  if (prefs.wallUrl) URL.revokeObjectURL(prefs.wallUrl);
  const buf = await wallpaperGet();
  prefs.wallUrl = buf.byteLength ? URL.createObjectURL(new Blob([buf])) : '';
}

// Média da capa em 16x16, puxada para a cor mais viva e clara (estilo player One UI).
async function artColor(src: string) {
  try {
    const img = new Image();
    img.src = src;
    await img.decode();
    const c = document.createElement('canvas');
    c.width = c.height = 16;
    const g = c.getContext('2d')!;
    g.drawImage(img, 0, 0, 16, 16);
    const d = g.getImageData(0, 0, 16, 16).data;
    // pesa pixels saturados: a média simples vira cinza
    let r = 0, gg = 0, b = 0, wsum = 0;
    for (let i = 0; i < d.length; i += 4) {
      const mx = Math.max(d[i], d[i + 1], d[i + 2]), mn = Math.min(d[i], d[i + 1], d[i + 2]);
      const w = 1 + ((mx - mn) / 255) * 8;
      r += d[i] * w; gg += d[i + 1] * w; b += d[i + 2] * w; wsum += w;
    }
    const [h, s] = rgbToHsl(r / wsum, gg / wsum, b / wsum);
    return `hsl(${Math.round(h)} ${Math.round(Math.min(Math.max(s, 0.35), 0.75) * 100)}% 72%)`;
  } catch { return ''; }
}
function rgbToHsl(r: number, g: number, b: number): [number, number] {
  r /= 255; g /= 255; b /= 255;
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b), l = (mx + mn) / 2, d = mx - mn;
  if (!d) return [0, 0];
  const s = d / (1 - Math.abs(2 * l - 1));
  const h = mx === r ? ((g - b) / d) % 6 : mx === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [(h * 60 + 360) % 360, s];
}

const HIST = 90; // 3 min de amostras a cada 2s
const push = (a: number[], v: number) => { a.push(v); if (a.length > HIST) a.shift(); };

export async function boot() {
  const [layout, p] = await Promise.all([kvGet('layout'), kvGet('ui')]);
  try { if (layout) ui.tabs = JSON.parse(layout); } catch { /* layout corrompido: fica o padrão */ }
  try { if (p) Object.assign(prefs, JSON.parse(p)); } catch { /* idem */ }
  loadWallpaper();

  spInit();
  let artKey = '', track = '', spTimer = 0;
  onMedia(async s => {
    live.media = s;
    // troca de faixa → reavalia playlist do Spotify (com folga para a API refletir a troca)
    if (`${s.title}|${s.artist}` !== track) {
      track = `${s.title}|${s.artist}`;
      clearTimeout(spTimer);
      spTimer = setTimeout(spRefresh, 1200);
    }
    if (s.art_key !== artKey) {
      artKey = s.art_key;
      live.art = s.art_key ? await mediaArt() : '';
      live.artColor = live.art ? await artColor(live.art) : '';
    }
  });
  onSys(s => {
    live.sys = s;
    const h = live.hist;
    push(h.cpu, s.cpu);
    push(h.gpu, s.gpu?.util ?? 0);
    push(h.ram, (s.ram_used / s.ram_total) * 100);
    push(h.rx, s.net_rx);
    push(h.tx, s.net_tx);
  });
}
