// Academia: tipos e cálculos (puros, testados em scripts/fit.check.ts).
// Meta simples: manutenção = 33 kcal/kg (ou a sua, se souber), objetivo ±5–10%;
// proteína 1,6–2,2 g/kg; gordura ~1 g/kg; carboidrato = o que sobra.
import { addDays, dayKey, parseDay } from './util.ts';

export type Goal = 'cut' | 'maintain' | 'bulk';
export const GOAL_LABEL: Record<Goal, string> = { cut: 'Cortar', maintain: 'Manter', bulk: 'Ganhar' };

export interface FitProfile {
  weight: number;              // kg informado; as contas usam a média das pesagens quando houver
  goal: Goal;
  adjust: number;              // % do objetivo (5–10)
  proteinGkg: number;          // 1,6–2,2
  fatGkg: number;              // ~1
  maintenance: number | null;  // kcal, se você souber; null = 33 × peso
  targetWeight: number | null; // linha da meta no gráfico
}
export const defaultProfile = (): FitProfile => ({ weight: 75, goal: 'maintain', adjust: 7, proteinGkg: 1.8, fatGkg: 1, maintenance: null, targetWeight: null });

export interface Portion { name: string; grams: number } // medida caseira (ex.: colher de sopa = 25 g)
export interface Food { id: string; name: string; kcal: number; p: number; c: number; f: number; portions: Portion[]; source: string } // por 100 g/ml
export interface Item { foodId?: string; name: string; grams: number; kcal: number; p: number; c: number; f: number } // foodId: permite editar depois
export interface Meal { id: string; name: string; items: Item[] } // um lançamento (refeição cadastrada, alimento ou só kcal)
export interface Template { id: string; name: string; items: { foodId: string; grams: number }[] }
export interface DayLog { weight: number | null; meals: Meal[] }
export const emptyDay = (): DayLog => ({ weight: null, meals: [] });

const r = (x: number) => Math.round(x);
const r1 = (x: number) => Math.round(x * 10) / 10;

// ---------- metas ----------
/** `weight`: peso usado nas contas (média das pesagens, se houver; senão o do perfil). */
export function targets(p: FitProfile, weight = p.weight) {
  const maintenance = p.maintenance ?? r(33 * weight);
  const sign = p.goal === 'cut' ? -1 : p.goal === 'bulk' ? 1 : 0;
  const kcal = r(maintenance * (1 + (sign * p.adjust) / 100));
  const protein = r(p.proteinGkg * weight);
  const fat = r(p.fatGkg * weight);
  const carbs = Math.max(0, r((kcal - protein * 4 - fat * 9) / 4));
  return { maintenance, kcal, protein, fat, carbs, weight };
}
export type Targets = ReturnType<typeof targets>;

// ---------- alimentos ----------
export function itemFor(food: Pick<Food, 'name' | 'kcal' | 'p' | 'c' | 'f'> & { id?: string }, grams: number): Item {
  const k = grams / 100;
  return { foodId: food.id, name: food.name, grams, kcal: r(food.kcal * k), p: r1(food.p * k), c: r1(food.c * k), f: r1(food.f * k) };
}

/** Alimentos salvos antes das porções múltiplas tinham `portion` (uma só). */
export function migrateFood(f: Food & { portion?: Portion | null }): Food {
  const { portion, ...rest } = f;
  return { ...rest, portions: f.portions ?? (portion ? [portion] : []) };
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

/** Ids de alimentos usados recentemente (mais recente primeiro): ordenam a grade do modal. */
export function recentFoodIds(days: Days, end: string, lookback = 30) {
  const out: string[] = [];
  for (const { log } of lastDays(days, end, lookback).reverse())
    for (const m of [...(log?.meals ?? [])].reverse())
      for (const it of m.items) if (it.foodId && !out.includes(it.foodId)) out.push(it.foodId);
  return out;
}

/** Peso no primeiro registro (para "desde o início"). */
export function firstWeight(days: Days): { date: string; weight: number } | null {
  const k = Object.keys(days).filter(d => typeof days[d]?.weight === 'number').sort()[0];
  return k ? { date: k, weight: days[k].weight! } : null;
}

/** Último peso registrado até `end` (para o − / + já partir dele). */
export function lastWeight(days: Days, end: string): number | null {
  const k = Object.keys(days).filter(d => d <= end && typeof days[d]?.weight === 'number').sort().pop();
  return k ? days[k].weight : null;
}

/** Itens lançados recentemente (sem repetir nome+gramas), do mais novo ao mais antigo: viram botões de um toque. */
export function recentItems(days: Days, end: string, n = 6, lookback = 21) {
  const seen = new Set<string>(), out: Item[] = [];
  for (const { log } of lastDays(days, end, lookback).reverse())
    for (const m of [...(log?.meals ?? [])].reverse())
      for (const it of m.items) {
        const k = `${it.name}|${it.grams}`;
        if (seen.has(k)) continue;
        seen.add(k);
        out.push(it);
        if (out.length >= n) return out;
      }
  return out;
}

// ---------- banco inicial (por 100 g; valores aproximados da Tabela TACO e porções caseiras médias — edite à vontade) ----------
const t = (name: string, kcal: number, p: number, c: number, f: number, portions: Portion[] = []): Food =>
  ({ id: `taco-${name}`, name, kcal, p, c, f, portions, source: 'TACO (aprox.)' });
const P = (name: string, grams: number): Portion => ({ name, grams });
export const SEED_FOODS: Food[] = [
  t('Arroz branco cozido', 128, 2.5, 28.1, 0.2, [P('colher de sopa', 25), P('escumadeira', 90)]),
  t('Arroz integral cozido', 124, 2.6, 25.8, 1.0, [P('colher de sopa', 25), P('escumadeira', 90)]),
  t('Feijão carioca cozido', 76, 4.8, 13.6, 0.5, [P('concha', 80), P('colher de sopa', 20)]),
  t('Peito de frango grelhado', 159, 32.0, 0, 2.5, [P('filé', 100)]),
  t('Patinho bovino grelhado', 219, 35.9, 0, 7.3, [P('bife', 100)]),
  t('Ovo cozido', 146, 13.3, 0.6, 9.5, [P('ovo', 50)]),
  t('Batata-doce cozida', 77, 0.6, 18.4, 0.1, [P('unidade média', 130)]),
  t('Aveia em flocos', 394, 13.9, 66.6, 8.5, [P('colher de sopa', 15)]),
  t('Pão francês', 300, 8.0, 58.6, 3.1, [P('pão', 50)]),
  t('Banana prata', 98, 1.3, 26.0, 0.1, [P('unidade', 60)]),
  t('Abacate', 96, 1.2, 6.0, 8.4, [P('colher de sopa', 30)]),
  t('Azeite de oliva', 884, 0, 0, 100, [P('colher de sopa', 13)]),
  t('Castanha-do-pará', 643, 14.5, 15.1, 63.5, [P('unidade', 4)]),
];
