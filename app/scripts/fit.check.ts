// node scripts/fit.check.ts — cálculos da aba Academia
import assert from 'node:assert';
import { targets, itemFor, dayTotals, weightTrend, adherence, movingAvg, lastWeight, recentItems, recentFoodIds, firstWeight, migrateFood, defaultProfile, type Days } from '../src/lib/fit.ts';
import { addDays, dayKey, parseDay } from '../src/lib/util.ts';

const END = '2026-11-20';
const day = (i: number) => dayKey(addDays(parseDay(END), i));
const p = { ...defaultProfile(), weight: 80, proteinGkg: 2, fatGkg: 1 };

// metas: manutenção 33 × peso, objetivo ±ajuste, carbo = o que sobra
let t = targets({ ...p, goal: 'cut', adjust: 10 });
assert.deepEqual([t.maintenance, t.kcal, t.protein, t.fat], [2640, 2376, 160, 80]);
assert.equal(t.carbs, Math.round((2376 - 160 * 4 - 80 * 9) / 4));
assert.equal(targets({ ...p, goal: 'bulk', adjust: 5 }).kcal, 2772);
assert.equal(targets({ ...p, goal: 'maintain' }).kcal, 2640);
// manutenção informada substitui 33 × peso
assert.equal(targets({ ...p, goal: 'cut', adjust: 10, maintenance: 3000 }).kcal, 2700);
// peso das contas pode vir da média das pesagens
t = targets(p, 90);
assert.deepEqual([t.maintenance, t.protein, t.weight], [2970, 180, 90]);

// alimentos
const ovo = itemFor({ name: 'Ovo', kcal: 146, p: 13.3, c: 0.6, f: 9.5 }, 50);
assert.deepEqual([ovo.kcal, ovo.p, ovo.f], [73, 6.7, 4.8]);

// 14 dias: semana anterior 80,0 kg, atual 80,1 kg; comendo 2700 kcal, 165 g de proteína
const it = (name: string, grams: number, kcal: number, p: number) => ({ name, grams, kcal, p, c: 0, f: 0 });
const days: Days = {};
for (let i = -13; i <= 0; i++)
  days[day(i)] = { weight: i <= -7 ? 80.0 : 80.1, meals: [{ id: 'm', name: 'Dia', items: [it('Base', 0, 2700, 165)] }] };
const tr = weightTrend(days, END);
assert.deepEqual([tr.cur, tr.prev, tr.delta, tr.nCur, tr.nPrev], [80.1, 80.0, 0.1, 7, 7]);

// adesão: meta 2640 → 2700 dentro de ±10%; proteína 165 ≥ 95% de 160
const ad = adherence(days, END, targets(p));
assert.deepEqual([ad.kcalHit, ad.protHit, ad.logged, ad.avgMeals], [7, 7, 7, 1]);

// média móvel
const ma = movingAvg(days, END, 3);
assert.equal(ma.length, 3);
assert.equal(ma[2].avg, 80.1);
assert.equal(dayTotals(undefined).meals, 0);

// último peso: pula dias sem pesagem e ignora o futuro
days[END].weight = null;
days[day(1)] = { weight: 99, meals: [] };
assert.equal(lastWeight(days, END), 80.1);
assert.equal(lastWeight({}, END), null);

// recentes: mais novo primeiro, sem repetir nome+gramas
days[day(-1)].meals.push({ id: 'a', name: 'Lanche', items: [it('Frango', 150, 239, 48), it('Arroz', 150, 192, 4)] });
days[END].meals.push({ id: 'b', name: 'Almoço', items: [it('Frango', 150, 239, 48)] });
const rec = recentItems(days, END);
assert.deepEqual(rec.slice(0, 3).map(x => x.name), ['Frango', 'Base', 'Arroz']);
assert.equal(rec.filter(x => x.name === 'Frango').length, 1);
// itens guardam de qual alimento vieram (para editar depois)
assert.equal(itemFor({ id: 'ovo', name: 'Ovo', kcal: 146, p: 13.3, c: 0.6, f: 9.5 }, 100).foodId, 'ovo');
days[END].meals.push({ id: 'c', name: 'Ovo', items: [{ ...it('Ovo', 100, 146, 13), foodId: 'ovo' }] });
days[day(-2)].meals.push({ id: 'd', name: 'x', items: [{ ...it('Arroz', 100, 128, 2), foodId: 'arroz' }, { ...it('Ovo', 50, 73, 7), foodId: 'ovo' }] });
assert.deepEqual(recentFoodIds(days, END), ['ovo', 'arroz']);
// primeiro peso
assert.deepEqual(firstWeight(days), { date: day(-13), weight: 80 });
assert.equal(firstWeight({}), null);
// migração: porção única antiga vira lista
const old = { id: 'x', name: 'X', kcal: 1, p: 0, c: 0, f: 0, source: '', portion: { name: 'un', grams: 10 } } as any;
assert.deepEqual(migrateFood(old).portions, [{ name: 'un', grams: 10 }]);
assert.equal('portion' in migrateFood(old), false);
assert.deepEqual(migrateFood({ ...old, portion: null }).portions, []);
console.log('fit ok');
