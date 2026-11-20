// Estado da aba Academia: perfil, banco de alimentos, refeições cadastradas e registros diários (kv 'fit:*').
import { kvGet, kvSet, kvPrefix } from './api';
import { today } from './util';
import { defaultProfile, emptyDay, SEED_FOODS, type Days, type DayLog, type Food, type FitProfile, type Template } from './fit.ts';

export const fit = $state({
  ready: false,
  today: today(),
  profile: defaultProfile(today()) as FitProfile,
  foods: [] as Food[],
  templates: [] as Template[],
  days: {} as Days,
});

export const uid = () => Math.random().toString(36).slice(2, 10);

let loading: Promise<void> | null = null;
/** Carrega tudo uma vez (os widgets da aba chamam ao montar). */
export function loadFit() {
  return (loading ??= (async () => {
    const [p, f, t, d] = await Promise.all([kvGet('fit:profile'), kvGet('fit:foods'), kvGet('fit:templates'), kvPrefix('fit:day:')]);
    try { if (p) Object.assign(fit.profile, JSON.parse(p)); } catch { /* perfil corrompido: padrão */ }
    try { fit.foods = f ? JSON.parse(f) : SEED_FOODS.map(x => ({ ...x })); } catch { fit.foods = SEED_FOODS.map(x => ({ ...x })); }
    try { fit.templates = t ? JSON.parse(t) : []; } catch { fit.templates = []; }
    const days: Days = {};
    for (const [k, v] of d) try { days[k.slice('fit:day:'.length)] = JSON.parse(v); } catch { /* dia corrompido: ignora */ }
    fit.days = days;
    fit.ready = true;
    setInterval(() => { if (today() !== fit.today) fit.today = today(); }, 30_000); // vira o dia
  })());
}

export const saveProfile = () => kvSet('fit:profile', JSON.stringify(fit.profile));
export const saveFoods = () => kvSet('fit:foods', JSON.stringify(fit.foods));
export const saveTemplates = () => kvSet('fit:templates', JSON.stringify(fit.templates));

/** Altera o registro de um dia (cria se não existir) e salva. */
export function editDay(date: string, fn: (d: DayLog) => void) {
  fit.days[date] ??= emptyDay();
  fn(fit.days[date]);
  kvSet(`fit:day:${date}`, JSON.stringify(fit.days[date]));
}
