// Academia: tipos e cálculos (puros, testados em scripts/fit.check.ts).
// Método: manutenção ≈ 33 kcal/kg por 1–2 semanas; confirmada, aplica o objetivo (±5–10%).
// Proteína 1,6–2,2 g/kg fixa em todas as fases; gordura ~1 g/kg; carboidrato = o que sobra.
import { addDays, dayKey, parseDay } from './util.ts';

export type Goal = 'cut' | 'maintain' | 'bulk';
export const GOAL_LABEL: Record<Goal, string> = { cut: 'Cortar', maintain: 'Manter', bulk: 'Ganhar' };

export interface FitProfile {
  weight: number;              // kg, informado
  goal: Goal;
  adjust: number;              // % do objetivo (5–10)
  proteinGkg: number;          // 1,6–2,2
  fatGkg: number;              // ~1
  maintenance: number | null;  // kcal medida e confirmada; null = descobrindo
  estimate: number | null;     // estimativa ajustada durante a descoberta (null = 33 × peso)
  review: number;              // % acumulado das revisões de fase (ex.: -5)
  targetWeight: number | null; // linha da meta no gráfico
  phaseStart: string;          // 'YYYY-MM-DD' início da fase atual (revisão a cada 14 dias)
}
export const defaultProfile = (today: string): FitProfile => ({
  weight: 75, goal: 'maintain', adjust: 7, proteinGkg: 1.8, fatGkg: 1, maintenance: null, estimate: null, review: 0, targetWeight: null, phaseStart: today,
});

export interface Food { id: string; name: string; kcal: number; p: number; c: number; f: number; portion: { name: string; grams: number } | null; source: string } // por 100 g/ml
export interface Item { name: string; grams: number; kcal: number; p: number; c: number; f: number }
export interface Meal { id: string; name: string; items: Item[] }
export interface Template { id: string; name: string; items: { foodId: string; grams: number }[] }
export interface DayLog { weight: number | null; meals: Meal[]; training: Record<string, number> } // grupo → séries
export const emptyDay = (): DayLog => ({ weight: null, meals: [], training: {} });

export const GROUPS = ['Peito', 'Costas', 'Ombros', 'Bíceps', 'Tríceps', 'Quadríceps', 'Posterior', 'Glúteos', 'Panturrilha', 'Abdômen'];
export const SETS_MIN = 5, SETS_MAX = 12, FREQ_MIN = 2, FREQ_MAX = 3;

const r = (x: number) => Math.round(x);
const r1 = (x: number) => Math.round(x * 10) / 10;

// ---------- metas ----------
export function targets(p: FitProfile) {
  const estimated = p.maintenance === null;
  const maintenance = p.maintenance ?? p.estimate ?? r(33 * p.weight);
  const sign = p.goal === 'cut' ? -1 : p.goal === 'bulk' ? 1 : 0;
  // descobrindo a manutenção: come na manutenção estimada; o objetivo só entra depois de confirmar
  const kcal = estimated ? maintenance : r(maintenance * (1 + (sign * p.adjust) / 100) * (1 + p.review / 100));
  const protein = r(p.proteinGkg * p.weight);
  const fat = r(p.fatGkg * p.weight);
  const carbs = Math.max(0, r((kcal - protein * 4 - fat * 9) / 4));
  return { estimated, maintenance, kcal, protein, fat, carbs };
}
export type Targets = ReturnType<typeof targets>;

// ---------- alimentos ----------
export function itemFor(food: Pick<Food, 'name' | 'kcal' | 'p' | 'c' | 'f'>, grams: number): Item {
  const k = grams / 100;
  return { name: food.name, grams, kcal: r(food.kcal * k), p: r1(food.p * k), c: r1(food.c * k), f: r1(food.f * k) };
}
export function dayTotals(d: DayLog | undefined) {
  const t = { kcal: 0, p: 0, c: 0, f: 0, meals: d?.meals.length ?? 0 };
  for (const m of d?.meals ?? []) for (const i of m.items) { t.kcal += i.kcal; t.p += i.p; t.c += i.c; t.f += i.f; }
  return { ...t, kcal: r(t.kcal), p: r1(t.p), c: r1(t.c), f: r1(t.f) };
}

// ---------- janelas de dias ----------
export type Days = Record<string, DayLog>; // 'YYYY-MM-DD' → registro
/** Os `n` dias terminando em `end` (inclusive), do mais antigo ao mais novo. */
export function lastDays(days: Days, end: string, n = 7) {
  const e = parseDay(end);
  return Array.from({ length: n }, (_, i) => { const k = dayKey(addDays(e, i - n + 1)); return { date: k, log: days[k] }; });
}
const avg = (xs: number[]) => (xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : null);
const weights = (w: { log?: DayLog }[]) => w.map(x => x.log?.weight).filter((x): x is number => typeof x === 'number');

/** Média móvel de 7 dias do peso (tira o ruído do dia a dia), para cada dia da janela. */
export function movingAvg(days: Days, end: string, n = 14) {
  return lastDays(days, end, n).map(({ date }) => {
    const ws = weights(lastDays(days, date, 7));
    return { date, weight: days[date]?.weight ?? null, avg: ws.length ? r1(avg(ws)!) : null };
  });
}

/** Média de peso dos últimos 7 dias vs. os 7 anteriores. */
export function weightTrend(days: Days, end: string) {
  const cur = weights(lastDays(days, end, 7));
  const prev = weights(lastDays(days, dayKey(addDays(parseDay(end), -7)), 7));
  const a = avg(cur), b = avg(prev);
  return { cur: a === null ? null : r1(a), prev: b === null ? null : r1(b), nCur: cur.length, nPrev: prev.length, delta: a !== null && b !== null ? r1(a - b) : null };
}

/** Adesão nos últimos 7 dias: calorias dentro de ±10% da meta, proteína ≥ 95% da meta, média de refeições. */
export function adherence(days: Days, end: string, t: Targets) {
  let kcalHit = 0, protHit = 0, logged = 0, meals = 0;
  for (const { log } of lastDays(days, end, 7)) {
    const s = dayTotals(log);
    if (!s.meals) continue;
    logged++;
    meals += s.meals;
    if (Math.abs(s.kcal - t.kcal) <= t.kcal * 0.1) kcalHit++;
    if (s.p >= t.protein * 0.95) protHit++;
  }
  return { kcalHit, protHit, logged, avgMeals: logged ? r1(meals / logged) : 0 };
}

export const KCAL_PER_KG = 7700; // energia aproximada de 1 kg de peso corporal

/** Achar a manutenção: precisa de ≥4 pesagens em cada uma das 2 semanas. Estável = ±0,2 kg. */
export function maintenanceCheck(days: Days, end: string, p: FitProfile) {
  const tr = weightTrend(days, end);
  if (tr.nCur < 4 || tr.nPrev < 4 || tr.delta === null) return { ready: false as const, trend: tr };
  const eaten = lastDays(days, end, 14).map(x => dayTotals(x.log)).filter(s => s.meals).map(s => s.kcal);
  const current = targets(p).maintenance;
  const measured = eaten.length >= 7 ? r(avg(eaten)!) : current; // o que você comeu de fato nessas 2 semanas
  if (Math.abs(tr.delta) <= 0.2) return { ready: true as const, stable: true, trend: tr, measured, suggested: measured };
  // peso mudou: ajusta pelo equivalente energético da variação semanal, limitado a ±15%
  const change = Math.max(-0.15 * measured, Math.min(0.15 * measured, (-tr.delta * KCAL_PER_KG) / 7));
  return { ready: true as const, stable: false, trend: tr, measured, suggested: r(measured + change) };
}

/** Revisão da fase a cada 14 dias: se o peso não andou para o lado do objetivo, sugere ±5% nas calorias. */
export function phaseReview(days: Days, end: string, p: FitProfile) {
  const since = Math.round((parseDay(end).getTime() - parseDay(p.phaseStart).getTime()) / 864e5);
  const tr = weightTrend(days, end);
  if (p.maintenance === null || since < 14 || tr.delta === null || tr.nCur < 4 || tr.nPrev < 4) return { due: false as const, since, trend: tr };
  const d = tr.delta;
  let pct = 0;
  if (p.goal === 'cut' && d > -0.1) pct = -5;
  else if (p.goal === 'bulk' && d < 0.05) pct = 5;
  else if (p.goal === 'maintain' && Math.abs(d) > 0.3) pct = d > 0 ? -5 : 5;
  return { due: true as const, since, trend: tr, pct };
}

/** Volume da semana (segunda a domingo de `end`): séries e frequência por grupo. */
export function weekVolume(days: Days, end: string) {
  const e = parseDay(end);
  const monday = addDays(e, -((e.getDay() + 6) % 7));
  const out: Record<string, { sets: number; freq: number }> = {};
  for (const g of GROUPS) out[g] = { sets: 0, freq: 0 };
  for (let i = 0; i < 7; i++) {
    const log = days[dayKey(addDays(monday, i))];
    for (const [g, sets] of Object.entries(log?.training ?? {})) {
      if (!sets) continue;
      out[g] ??= { sets: 0, freq: 0 };
      out[g].sets += sets;
      out[g].freq++;
    }
  }
  return out;
}

// ---------- banco inicial (por 100 g; valores aproximados da Tabela TACO — edite à vontade) ----------
const t = (name: string, kcal: number, p: number, c: number, f: number, portion: Food['portion'] = null): Food =>
  ({ id: `taco-${name}`, name, kcal, p, c, f, portion, source: 'TACO (aprox.)' });
export const SEED_FOODS: Food[] = [
  t('Arroz branco cozido', 128, 2.5, 28.1, 0.2),
  t('Arroz integral cozido', 124, 2.6, 25.8, 1.0),
  t('Feijão carioca cozido', 76, 4.8, 13.6, 0.5),
  t('Peito de frango grelhado', 159, 32.0, 0, 2.5),
  t('Patinho bovino grelhado', 219, 35.9, 0, 7.3),
  t('Ovo cozido', 146, 13.3, 0.6, 9.5, { name: '1 ovo', grams: 50 }),
  t('Batata-doce cozida', 77, 0.6, 18.4, 0.1),
  t('Aveia em flocos', 394, 13.9, 66.6, 8.5),
  t('Pão francês', 300, 8.0, 58.6, 3.1, { name: '1 pão', grams: 50 }),
  t('Banana prata', 98, 1.3, 26.0, 0.1),
  t('Abacate', 96, 1.2, 6.0, 8.4),
  t('Azeite de oliva', 884, 0, 0, 100, { name: '1 colher de sopa', grams: 8 }),
  t('Castanha-do-pará', 643, 14.5, 15.1, 63.5),
];
