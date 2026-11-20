// node scripts/fit.check.ts — cálculos da aba Academia
import assert from 'node:assert';
import { targets, itemFor, dayTotals, weightTrend, adherence, maintenanceCheck, phaseReview, weekVolume, movingAvg, defaultProfile, type Days } from '../src/lib/fit.ts';
import { addDays, dayKey, parseDay } from '../src/lib/util.ts';

const END = '2026-11-20'; // sexta
const day = (i: number) => dayKey(addDays(parseDay(END), i));
const p = { ...defaultProfile('2026-11-01'), weight: 80, proteinGkg: 2, fatGkg: 1 };

// metas: descobrindo a manutenção → come em 33 × peso, sem o ajuste do objetivo
let t = targets({ ...p, goal: 'cut', adjust: 10 });
assert.deepEqual([t.estimated, t.maintenance, t.kcal, t.protein, t.fat], [true, 2640, 2640, 160, 80]);
assert.equal(t.carbs, Math.round((2640 - 160 * 4 - 80 * 9) / 4)); // 320
// manutenção confirmada → aplica o objetivo e as revisões; proteína não muda
t = targets({ ...p, goal: 'cut', adjust: 10, maintenance: 2800 });
assert.deepEqual([t.estimated, t.kcal, t.protein], [false, 2520, 160]);
assert.equal(targets({ ...p, goal: 'bulk', adjust: 5, maintenance: 2800, review: 5 }).kcal, Math.round(2800 * 1.05 * 1.05));
assert.equal(targets({ ...p, goal: 'maintain', maintenance: 2800 }).kcal, 2800);
// estimativa ajustada na descoberta substitui 33 × peso, ainda sem aplicar o objetivo
assert.deepEqual([targets({ ...p, goal: 'cut', estimate: 2500 }).kcal, targets({ ...p, goal: 'cut', estimate: 2500 }).estimated], [2500, true]);

// alimentos
const ovo = itemFor({ name: 'Ovo', kcal: 146, p: 13.3, c: 0.6, f: 9.5 }, 50);
assert.deepEqual([ovo.kcal, ovo.p, ovo.f], [73, 6.7, 4.8]);

// dias sintéticos: semana anterior 80,0 kg; semana atual 80,1 kg; comendo 2700
const days: Days = {};
for (let i = -13; i <= 0; i++)
  days[day(i)] = { weight: i <= -7 ? 80.0 : 80.1, meals: [{ id: 'm', name: 'Dia', items: [{ name: 'x', grams: 0, kcal: 2700, p: 165, c: 0, f: 0 }] }], training: {} };
const tr = weightTrend(days, END);
assert.deepEqual([tr.cur, tr.prev, tr.delta, tr.nCur, tr.nPrev], [80.1, 80.0, 0.1, 7, 7]);

// manutenção estável (±0,2) → confirma o que comeu de fato (2700)
let mc = maintenanceCheck(days, END, p);
assert.ok(mc.ready && mc.stable && mc.measured === 2700 && mc.suggested === 2700);
// peso subiu 0,3 kg/semana → sugere descer 0,3 × 7700 / 7 = 330 kcal/dia
for (let i = -6; i <= 0; i++) days[day(i)].weight = 80.3;
mc = maintenanceCheck(days, END, p);
assert.ok(mc.ready && !mc.stable && mc.suggested === 2700 - 330, `sugestão ${mc.ready && mc.suggested}`);
// subiu 0,5 kg → seria −550, mas o ajuste é limitado a 15% (−405)
for (let i = -6; i <= 0; i++) days[day(i)].weight = 80.5;
mc = maintenanceCheck(days, END, p);
assert.ok(mc.ready && !mc.stable && mc.suggested === 2700 - 405, `limite ${mc.ready && mc.suggested}`);
// poucas pesagens → ainda não dá para concluir
assert.equal(maintenanceCheck({ [END]: days[END] }, END, p).ready, false);

// revisão: cortando, peso subiu → -5%; manutenção ainda não confirmada → não revisa
assert.equal(phaseReview(days, END, { ...p, goal: 'cut', maintenance: 2700, phaseStart: day(-14) }).due, true);
const rv = phaseReview(days, END, { ...p, goal: 'cut', maintenance: 2700, phaseStart: day(-14) });
assert.ok(rv.due && rv.pct === -5);
assert.equal(phaseReview(days, END, { ...p, goal: 'cut', phaseStart: day(-14) }).due, false);
assert.equal(phaseReview(days, END, { ...p, goal: 'cut', maintenance: 2700, phaseStart: day(-5) }).due, false, 'antes de 14 dias');
const bulk = phaseReview(days, END, { ...p, goal: 'bulk', maintenance: 2700, phaseStart: day(-20) });
assert.ok(bulk.due && bulk.pct === 0, 'ganhando e peso subiu: mantém');

// adesão: meta 2640 → 2700 está dentro de ±10%; proteína 165 ≥ 95% de 160
const ad = adherence(days, END, targets(p));
assert.deepEqual([ad.kcalHit, ad.protHit, ad.logged, ad.avgMeals], [7, 7, 7, 1]);

// volume: semana de segunda (16/11) a domingo; treino fora da semana não conta
days[day(-4)].training = { Peito: 4, Costas: 5 }; // segunda 16
days[day(-2)].training = { Peito: 4 };            // quarta 18
days[day(-6)].training = { Peito: 10 };           // sábado anterior: fora
const v = weekVolume(days, END);
assert.deepEqual(v.Peito, { sets: 8, freq: 2 });
assert.deepEqual(v.Costas, { sets: 5, freq: 1 });
assert.deepEqual(v.Ombros, { sets: 0, freq: 0 });

// média móvel e totais
const ma = movingAvg(days, END, 3);
assert.equal(ma.length, 3);
assert.equal(ma[2].avg, 80.5);
assert.equal(dayTotals(undefined).meals, 0);
console.log('fit ok');
