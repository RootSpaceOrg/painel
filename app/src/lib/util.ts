// Datas sempre no fuso local. Dia = 'YYYY-MM-DD'.
const pad = (n: number) => String(n).padStart(2, '0');
export const dayKey = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
export const today = () => dayKey(new Date());
export const addDays = (d: Date, n: number) => { const r = new Date(d); r.setDate(r.getDate() + n); return r; };
export const startOfDay = (d: Date) => { const r = new Date(d); r.setHours(0, 0, 0, 0); return r; };
export const parseDay = (s: string) => { const [y, m, d] = s.split('-').map(Number); return new Date(y, m - 1, d); };

// Bitmask de dias: seg=bit0 ... dom=bit6.
export const ALL_DAYS = 127;
export const WEEKDAYS = 31;
export const dayBit = (d: Date) => 1 << ((d.getDay() + 6) % 7);
export const DAY_LETTERS = ['S', 'T', 'Q', 'Q', 'S', 'S', 'D'];
export const DAY_NAMES = ['seg', 'ter', 'qua', 'qui', 'sex', 'sáb', 'dom'];
export function daysLabel(mask: number) {
  if (mask === 0) return 'avulsa';
  if (mask === ALL_DAYS) return 'todo dia';
  if (mask === WEEKDAYS) return 'dias úteis';
  if (mask === 96) return 'fim de semana';
  return DAY_NAMES.filter((_, i) => mask & (1 << i)).join(', ');
}

/** Sequência de dias programados concluídos até hoje. Hoje ainda não feito não quebra a sequência. */
export function streak(log: Set<string>, mask: number, from = new Date()) {
  const m = mask || ALL_DAYS;
  let n = 0;
  for (let i = 0; i < 400; i++) {
    const d = addDays(from, -i);
    if (!(m & dayBit(d))) continue;
    if (log.has(dayKey(d))) n++;
    else if (i > 0) break;
  }
  return n;
}

const fmt = (o: Intl.DateTimeFormatOptions) => new Intl.DateTimeFormat('pt-BR', o);
export const fmtTime = fmt({ hour: '2-digit', minute: '2-digit' });
export const fmtWeekday = fmt({ weekday: 'long' });
export const fmtDayMonth = fmt({ day: 'numeric', month: 'long' });
export const fmtShort = fmt({ weekday: 'short', day: 'numeric', month: 'short' });

export function dayHeading(d: Date) {
  const diff = Math.round((startOfDay(d).getTime() - startOfDay(new Date()).getTime()) / 864e5);
  if (diff === 0) return 'Hoje';
  if (diff === 1) return 'Amanhã';
  if (diff === -1) return 'Ontem';
  return fmtShort.format(d);
}

export function fmtBytes(b: number, digits = 1) {
  const u = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  while (b >= 1024 && i < u.length - 1) { b /= 1024; i++; }
  return `${b.toFixed(i === 0 ? 0 : digits)} ${u[i]}`;
}

export const fmtDuration = (s: number) => {
  s = Math.max(0, Math.floor(s));
  return `${Math.floor(s / 60)}:${pad(s % 60)}`;
};

export function relMinutes(ms: number) {
  const m = Math.round(ms / 60000);
  if (m < 60) return `${m} min`;
  const h = Math.floor(m / 60), r = m % 60;
  return r ? `${h}h${pad(r)}` : `${h}h`;
}

/** LRC "[mm:ss.xx] texto" → linhas ordenadas por tempo (s). Linhas sem tempo são ignoradas. */
export function parseLrc(lrc: string) {
  const out: { t: number; text: string }[] = [];
  for (const line of lrc.split(/\r?\n/)) {
    const tags = [...line.matchAll(/\[(\d+):(\d+(?:\.\d+)?)\]/g)];
    if (!tags.length) continue;
    const text = line.replace(/\[[^\]]*\]/g, '').trim();
    for (const m of tags) out.push({ t: +m[1] * 60 + +m[2], text });
  }
  return out.sort((a, b) => a.t - b.t);
}

/** Índice da linha atual (-1 antes da primeira). */
export function lrcIndex(lines: { t: number }[], pos: number) {
  let lo = 0, hi = lines.length - 1, ans = -1;
  while (lo <= hi) { const mid = (lo + hi) >> 1; if (lines[mid].t <= pos) { ans = mid; lo = mid + 1; } else hi = mid - 1; }
  return ans;
}

/** A sessão de áudio (nome do .exe, ex. "Spotify") é do app que está tocando a mídia (ID do Windows, ex. "Spotify.exe")? */
export function sameApp(mediaId: string, exeStem: string) {
  const norm = (s: string) => s.toLowerCase().replace(/\.exe$/, '').replace(/[^a-z0-9]/g, '');
  const id = norm(mediaId), stem = norm(exeStem);
  if (stem.length < 3 || !id) return false;
  return id === stem || id.includes(stem) || (id.length >= 4 && stem.includes(id));
}
