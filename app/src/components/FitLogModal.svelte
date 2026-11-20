<script lang="ts">
  // Lançar (ou editar) uma refeição: marca vários alimentos, ajusta quantidade (g, porção caseira, − / +),
  // total ao vivo; depois de lançar, pode salvar como refeição pronta. Também gerencia o banco ("Meus alimentos").
  import { fit, fitUi, fitDay, editDay, uid, saveFoods, saveTemplates } from '../lib/fit.svelte';
  import { itemFor, dayTotals, recentFoodIds, type Food, type Item, type Portion } from '../lib/fit.ts';
  import { foodSearch, type FoodHit } from '../lib/api';
  import { dayHeading, parseDay } from '../lib/util';
  import Icon from './Icon.svelte';

  const d = fitDay();
  const editing = fitUi.log?.mealId ? fit.days[d]?.meals.find(m => m.id === fitUi.log!.mealId) ?? null : null;
  const close = () => (fitUi.log = null);
  const n = (v: number) => Math.round(v).toLocaleString('pt-BR');
  const norm = (s: string) => s.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();
  const foodById = (id?: string) => (id ? fit.foods.find(f => f.id === id) : undefined);

  // ---------- refeição em montagem ----------
  // unit: -1 = gramas; i ≥ 0 = índice da porção caseira. fixed: item antigo sem alimento no banco (só remover).
  type Line = { key: string; foodId?: string; unit: number; qty: number; fixed?: Item };
  const lineFromItem = (it: Item): Line =>
    foodById(it.foodId) ? { key: uid(), foodId: it.foodId, unit: -1, qty: it.grams } : { key: uid(), unit: -1, qty: it.grams, fixed: it };
  let lines = $state<Line[]>(editing ? editing.items.map(lineFromItem) : []);
  let name = $state(editing?.name ?? '');

  const gramsOf = (l: Line) => { const f = foodById(l.foodId); return l.unit >= 0 && f?.portions[l.unit] ? l.qty * f.portions[l.unit].grams : l.qty; };
  const itemOf = (l: Line): Item | null => {
    if (l.fixed) return l.fixed;
    const f = foodById(l.foodId);
    return f ? itemFor(f, Math.round(gramsOf(l))) : null;
  };
  const items = $derived(lines.map(itemOf).filter((x): x is Item => !!x));
  const tot = $derived(dayTotals({ weight: null, meals: [{ id: '', name: '', items }] }));
  const autoName = $derived(items.map(i => i.name.split(' ')[0]).slice(0, 3).join(' + ') || 'Refeição');

  const selected = (f: Food) => lines.some(l => l.foodId === f.id);
  function toggle(f: Food) {
    const i = lines.findIndex(l => l.foodId === f.id);
    if (i >= 0) lines.splice(i, 1);
    else lines.push(f.portions.length ? { key: uid(), foodId: f.id, unit: 0, qty: 1 } : { key: uid(), foodId: f.id, unit: -1, qty: 100 });
  }
  const step = (l: Line, dir: number) => { const s = l.unit < 0 ? 10 : 1; l.qty = Math.max(0, Math.round((l.qty + dir * s) * 10) / 10); };
  function setUnit(l: Line, unit: number) {
    const g = gramsOf(l);
    l.unit = unit;
    l.qty = unit < 0 ? Math.round(g) : 1; // porção caseira começa em 1; voltar para gramas mantém o peso
  }

  // ---------- lançar / salvar ----------
  let done = $state(false);
  let tplName = $state('');
  let tplSaved = $state(false);
  function launch() {
    if (!items.length) return;
    const meal = { id: editing?.id ?? uid(), name: name.trim() || autoName, items };
    editDay(d, x => {
      const i = x.meals.findIndex(m => m.id === meal.id);
      if (i >= 0) x.meals[i] = meal; else x.meals.push(meal);
    });
    if (editing) close();
    else { done = true; tplName = meal.name; }
  }
  function removeMeal() {
    if (!editing || !confirm(`Apagar "${editing.name}" deste dia?`)) return;
    editDay(d, x => { x.meals = x.meals.filter(m => m.id !== editing.id); });
    close();
  }
  const savable = $derived(lines.filter(l => l.foodId && !l.fixed));
  function saveTemplate() {
    if (!tplName.trim() || !savable.length) return;
    fit.templates.push({ id: uid(), name: tplName.trim(), items: savable.map(l => ({ foodId: l.foodId!, grams: Math.round(gramsOf(l)) })) });
    saveTemplates();
    tplSaved = true;
  }
  function another() { lines = []; name = ''; done = false; tplSaved = false; }

  // ---------- abas ----------
  let tab = $state<'foods' | 'saved' | 'manage'>('foods');
  let q = $state('');
  const recent = $derived(recentFoodIds(fit.days, d));
  const grid = $derived(
    fit.foods
      .filter(f => !q.trim() || norm(f.name).includes(norm(q.trim())))
      .sort((a, b) => {
        const ra = recent.indexOf(a.id), rb = recent.indexOf(b.id);
        return (ra < 0 ? 999 : ra) - (rb < 0 ? 999 : rb) || a.name.localeCompare(b.name);
      }),
  );

  // busca online (Open Food Facts, gratuito) e cadastro na hora
  let hits = $state<FoodHit[]>([]);
  let searching = $state(false);
  let searchMsg = $state('');
  async function searchOnline() {
    if (!q.trim()) return;
    searching = true; searchMsg = ''; hits = [];
    try { hits = await foodSearch(q.trim()); if (!hits.length) searchMsg = 'Nada encontrado online. Cadastre abaixo.'; }
    catch { searchMsg = 'Sem conexão com a busca online. Cadastre abaixo.'; }
    searching = false;
  }
  function addFood(food: Food, select = true) {
    fit.foods.unshift(food);
    saveFoods();
    if (select) toggle(food);
  }
  const importHit = (h: FoodHit) => {
    addFood({ id: uid(), name: h.brand ? `${h.name} (${h.brand})` : h.name, kcal: h.kcal, p: h.p, c: h.c, f: h.f, portions: [], source: 'Open Food Facts' });
    hits = hits.filter(x => x !== h);
  };

  // formulário de alimento (novo ou edição em "Meus alimentos")
  type FoodForm = { id?: string; name: string; kcal: string; p: string; c: string; f: string; portions: { name: string; grams: string }[] };
  const blankForm = (nm = ''): FoodForm => ({ name: nm, kcal: '', p: '', c: '', f: '', portions: [] });
  let form = $state<FoodForm | null>(null);
  let formErr = $state('');
  const num = (s: string) => { const x = parseFloat(String(s).replace(',', '.')); return Number.isFinite(x) && x >= 0 ? x : NaN; };
  function editFood(f: Food) {
    form = { id: f.id, name: f.name, kcal: String(f.kcal), p: String(f.p), c: String(f.c), f: String(f.f), portions: f.portions.map(p => ({ name: p.name, grams: String(p.grams) })) };
  }
  function saveForm() {
    if (!form) return;
    formErr = '';
    const [kcal, p, c, f] = [num(form.kcal), num(form.p || '0'), num(form.c || '0'), num(form.f || '0')];
    if (!form.name.trim() || Number.isNaN(kcal) || [p, c, f].some(Number.isNaN)) { formErr = 'Preencha o nome e as calorias (por 100 g).'; return; }
    const portions: Portion[] = form.portions.filter(x => x.name.trim() && num(x.grams) > 0).map(x => ({ name: x.name.trim(), grams: num(x.grams) }));
    const food: Food = { id: form.id ?? uid(), name: form.name.trim(), kcal: Math.round(kcal), p, c, f, portions, source: form.id ? foodById(form.id)?.source ?? 'manual' : 'manual' };
    if (form.id) { const i = fit.foods.findIndex(x => x.id === form!.id); fit.foods[i] = food; saveFoods(); }
    else addFood(food, tab === 'foods');
    form = null;
  }
  function delFood(f: Food) {
    const used = fit.templates.filter(t => t.items.some(i => i.foodId === f.id)).map(t => t.name);
    if (!confirm(used.length ? `"${f.name}" está em: ${used.join(', ')}. Excluir mesmo assim?` : `Excluir "${f.name}" do banco?`)) return;
    fit.foods = fit.foods.filter(x => x.id !== f.id);
    lines = lines.filter(l => l.foodId !== f.id);
    saveFoods();
  }

  // refeições salvas
  const tplKcal = (id: string) =>
    (fit.templates.find(t => t.id === id)?.items ?? []).reduce((a, i) => { const f = foodById(i.foodId); return a + (f ? itemFor(f, i.grams).kcal : 0); }, 0);
  function useTemplate(id: string) {
    const t = fit.templates.find(x => x.id === id);
    if (!t) return;
    for (const i of t.items) if (foodById(i.foodId)) lines.push({ key: uid(), foodId: i.foodId, unit: -1, qty: i.grams });
    if (!name.trim()) name = t.name;
    tab = 'foods';
  }
  function renameTemplate(id: string) {
    const t = fit.templates.find(x => x.id === id);
    const nm = t && prompt('Novo nome da refeição:', t.name);
    if (t && nm?.trim()) { t.name = nm.trim(); saveTemplates(); }
  }
  function delTemplate(id: string) {
    const t = fit.templates.find(x => x.id === id);
    if (!t || !confirm(`Excluir a refeição salva "${t.name}"?`)) return;
    fit.templates = fit.templates.filter(x => x.id !== id);
    saveTemplates();
  }

  function onKey(e: KeyboardEvent) { if (e.key === 'Escape') { if (form) form = null; else close(); } }
</script>

<svelte:window onkeydown={onKey} />

{#snippet foodForm()}
  {#if form}
    <div class="form glass">
      <span class="label">{form.id ? 'Editar alimento' : 'Cadastrar alimento'} · valores por 100 g</span>
      <input bind:value={form.name} placeholder="Nome" aria-label="Nome do alimento" />
      <div class="macros">
        <label><span>kcal</span><input class="mono" inputmode="decimal" bind:value={form.kcal} /></label>
        <label><span>Proteína</span><input class="mono" inputmode="decimal" bind:value={form.p} /></label>
        <label><span>Carbo</span><input class="mono" inputmode="decimal" bind:value={form.c} /></label>
        <label><span>Gordura</span><input class="mono" inputmode="decimal" bind:value={form.f} /></label>
      </div>
      <span class="label">Porções caseiras</span>
      {#each form.portions as pt, i}
        <div class="row">
          <input class="grow" bind:value={pt.name} placeholder="ex.: colher de sopa" aria-label="Nome da porção" />
          <input class="mono gr" inputmode="decimal" bind:value={pt.grams} placeholder="g" aria-label="Gramas da porção" />
          <button class="ghost icon sm" aria-label="Tirar porção" onclick={() => form!.portions.splice(i, 1)}><Icon name="x" /></button>
        </div>
      {/each}
      <button class="ghost addp" onclick={() => form!.portions.push({ name: '', grams: '' })}><Icon name="plus" /> porção</button>
      {#if formErr}<span class="err">{formErr}</span>{/if}
      <div class="row">
        <button class="primary" onclick={saveForm}>{form.id ? 'Salvar' : 'Cadastrar e usar'}</button>
        <button class="ghost" onclick={() => (form = null)}>Cancelar</button>
      </div>
    </div>
  {/if}
{/snippet}

<div class="overlay" role="presentation" onpointerdown={e => e.target === e.currentTarget && close()}>
  <div class="panel glass" role="dialog" aria-modal="true" aria-label={editing ? 'Editar refeição' : 'Lançar refeição'}>
    <header>
      <span class="label">{editing ? 'Editar refeição' : 'Lançar refeição'} · {dayHeading(parseDay(d))}</span>
      <button class="ghost icon" aria-label="Fechar" onclick={close}><Icon name="x" /></button>
    </header>

    <div class="body">
      <!-- esquerda: escolher -->
      <section class="pick">
        <div class="tabs">
          <button class:on={tab === 'foods'} onclick={() => (tab = 'foods')}>Alimentos</button>
          <button class:on={tab === 'saved'} onclick={() => (tab = 'saved')}>Refeições salvas <span class="cnt">{fit.templates.length}</span></button>
          <button class:on={tab === 'manage'} onclick={() => { tab = 'manage'; form = null; }}>Meus alimentos</button>
        </div>

        {#if tab === 'foods'}
          <div class="row">
            <input class="grow" bind:value={q} placeholder="Buscar alimento…" aria-label="Buscar alimento" />
            {#if q.trim()}<button onclick={searchOnline} disabled={searching}>{searching ? 'Buscando…' : 'Buscar online'}</button>{/if}
          </div>
          {#if form}{@render foodForm()}{/if}
          <div class="grid scroll">
            {#each grid as f (f.id)}
              <button class="tile" class:sel={selected(f)} aria-pressed={selected(f)} onclick={() => toggle(f)}>
                {#if selected(f)}<span class="ck"><Icon name="check" /></span>{/if}
                <span class="tn">{f.name}</span>
                <span class="tk mono">{f.kcal} kcal/100 g</span>
              </button>
            {/each}
            {#if q.trim()}
              <button class="tile new" onclick={() => (form = blankForm(q.trim()))}><Icon name="plus" /><span class="tn">Cadastrar “{q.trim()}”</span></button>
            {/if}
          </div>
          {#if hits.length || searchMsg}
            <div class="online">
              <span class="label">Online · Open Food Facts (confira com o rótulo)</span>
              {#if searchMsg}<span class="faint small">{searchMsg}</span>{/if}
              <div class="hits scroll">
                {#each hits as h}
                  <button class="hit" onclick={() => importHit(h)}>
                    <span class="grow ellipsis">{h.name}{h.brand ? ` · ${h.brand}` : ''}</span>
                    <span class="mono faint">{h.kcal} kcal · P {h.p}</span>
                    <span class="addh">+ usar</span>
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        {:else if tab === 'saved'}
          <div class="saved scroll">
            {#each fit.templates as t (t.id)}
              <button class="sv" onclick={() => useTemplate(t.id)}>
                <span class="grow"><span class="tn">{t.name}</span>
                  <span class="faint small ellipsis">{t.items.map(i => foodById(i.foodId)?.name ?? '?').join(' · ')}</span></span>
                <span class="mono">{n(tplKcal(t.id))} kcal</span>
              </button>
            {:else}
              <p class="faint">Nenhuma refeição salva ainda. Monte uma, lance e toque em “Salvar como refeição pronta”.</p>
            {/each}
          </div>
        {:else}
          <div class="manage scroll">
            {#if form}{@render foodForm()}{:else}<button class="ghost addp" onclick={() => (form = blankForm())}><Icon name="plus" /> Novo alimento</button>{/if}
            <span class="label">Alimentos · por 100 g</span>
            {#each fit.foods as f (f.id)}
              <div class="mrow">
                <span class="grow"><span class="ellipsis">{f.name}</span>
                  <span class="faint small mono">{f.kcal} kcal · P {f.p} · C {f.c} · G {f.f}{f.portions.length ? ` · ${f.portions.map(p => `${p.name} ${p.grams} g`).join(', ')}` : ''}</span></span>
                <button class="ghost icon sm" aria-label="Editar {f.name}" onclick={() => editFood(f)}><Icon name="edit" /></button>
                <button class="ghost icon sm danger" aria-label="Excluir {f.name}" onclick={() => delFood(f)}><Icon name="trash" /></button>
              </div>
            {/each}
            <span class="label">Refeições salvas</span>
            {#each fit.templates as t (t.id)}
              <div class="mrow">
                <span class="grow ellipsis">{t.name} <span class="faint mono small">· {n(tplKcal(t.id))} kcal</span></span>
                <button class="ghost icon sm" aria-label="Renomear {t.name}" onclick={() => renameTemplate(t.id)}><Icon name="edit" /></button>
                <button class="ghost icon sm danger" aria-label="Excluir {t.name}" onclick={() => delTemplate(t.id)}><Icon name="trash" /></button>
              </div>
            {:else}<span class="faint small">Nenhuma.</span>{/each}
          </div>
        {/if}
      </section>

      <!-- direita: a refeição -->
      <section class="meal">
        {#if done}
          <div class="done">
            <span class="big">✓</span>
            <b>Lançado: {n(tot.kcal)} kcal · {n(tot.p)} g de proteína</b>
            {#if savable.length && !tplSaved}
              <span class="faint">Quer salvar para lançar com um toque depois?</span>
              <div class="row"><input class="grow" bind:value={tplName} aria-label="Nome da refeição pronta" />
                <button class="primary" onclick={saveTemplate}>Salvar como refeição pronta</button></div>
            {:else if tplSaved}<span class="ok">Refeição salva ✓</span>{/if}
            <div class="row">
              <button onclick={another}>Lançar outra</button>
              <button class="ghost" onclick={close}>Fechar</button>
            </div>
          </div>
        {:else}
          <input class="name" bind:value={name} placeholder={autoName} aria-label="Nome da refeição" />
          <ul class="lines scroll">
            {#each lines as l, i (l.key)}
              {@const f = foodById(l.foodId)}
              {@const it = itemOf(l)}
              <li>
                <div class="lt"><span class="ellipsis">{it?.name ?? '?'}</span>
                  <button class="ghost icon sm" aria-label="Tirar" onclick={() => lines.splice(i, 1)}><Icon name="x" /></button></div>
                {#if l.fixed}
                  <span class="faint small">lançamento antigo · {n(l.fixed.kcal)} kcal</span>
                {:else}
                  <div class="qty">
                    <button class="st" aria-label="Menos" onclick={() => step(l, -1)}>−</button>
                    <input class="mono qv" inputmode="decimal" value={String(l.qty).replace('.', ',')} aria-label="Quantidade"
                      onchange={e => { const v = num(e.currentTarget.value); if (!Number.isNaN(v)) l.qty = v; }} />
                    <button class="st" aria-label="Mais" onclick={() => step(l, 1)}>+</button>
                    <select value={l.unit} onchange={e => setUnit(l, +e.currentTarget.value)} aria-label="Unidade">
                      <option value={-1}>g</option>
                      {#each f?.portions ?? [] as p, pi}<option value={pi}>{p.name} ({p.grams} g)</option>{/each}
                    </select>
                    <span class="lk mono">{n(it?.kcal ?? 0)} kcal</span>
                  </div>
                {/if}
              </li>
            {:else}
              <li class="hint faint">Toque nos alimentos à esquerda para montar a refeição.</li>
            {/each}
          </ul>
          <div class="totals">
            <div><span class="tv mono">{n(tot.kcal)}</span><span class="tl">kcal</span></div>
            <div><span class="tv mono p">{n(tot.p)}</span><span class="tl">g proteína</span></div>
            <div><span class="tv mono sm2">{n(tot.c)}</span><span class="tl">g carbo</span></div>
            <div><span class="tv mono sm2">{n(tot.f)}</span><span class="tl">g gordura</span></div>
          </div>
          <div class="row end">
            {#if editing}<button class="ghost danger" onclick={removeMeal}>Apagar</button>{/if}
            <button class="primary go" disabled={!items.length} onclick={launch}>{editing ? 'Salvar alterações' : 'Lançar'}</button>
          </div>
        {/if}
      </section>
    </div>
  </div>
</div>

<style>
  .overlay { position: fixed; inset: 0; z-index: 70; display: grid; padding: 22px; background: rgba(3, 12, 14, 0.55); animation: fade 0.15s; }
  .panel { display: flex; flex-direction: column; min-height: 0; padding: 16px 22px 20px; background: var(--glass-strong); }
  header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
  .body { flex: 1; min-height: 0; display: grid; grid-template-columns: 1.35fr 1fr; gap: 24px; }
  section { min-height: 0; display: flex; flex-direction: column; gap: 10px; }
  .tabs { display: flex; gap: 6px; }
  .tabs button { border-radius: 999px; padding: 6px 14px; font-size: 13px; }
  .tabs .on { color: var(--ink); background: var(--teal); border-color: transparent; }
  .cnt { font-size: 11px; opacity: 0.7; margin-left: 2px; }
  .small { font-size: 11.5px; }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(130px, 1fr)); gap: 8px; align-content: start; flex: 1; min-height: 0; }
  .tile { position: relative; display: flex; flex-direction: column; align-items: flex-start; justify-content: space-between; gap: 6px; min-height: 76px; padding: 10px 12px; text-align: left; border-radius: 12px; }
  .tile.sel { background: rgba(127, 224, 208, 0.16); border-color: rgba(127, 224, 208, 0.55); }
  .tile:active { transform: scale(0.97); }
  .tn { font-size: 13.5px; font-weight: 500; line-height: 1.25; }
  .tk { font-size: 11px; color: var(--muted); }
  .ck { position: absolute; top: 8px; right: 8px; width: 20px; height: 20px; border-radius: 50%; background: var(--teal); color: var(--ink); display: grid; place-items: center; }
  .ck :global(svg) { width: 12px; height: 12px; }
  .tile.new { border-style: dashed; color: var(--teal); justify-content: center; align-items: center; text-align: center; }
  .tile.new :global(svg) { width: 16px; height: 16px; }
  .online { display: flex; flex-direction: column; gap: 6px; }
  .hits { max-height: 170px; display: flex; flex-direction: column; gap: 4px; }
  .hit { display: flex; align-items: center; gap: 10px; text-align: left; padding: 8px 10px; font-size: 12.5px; }
  .addh { color: var(--teal); font-size: 12px; white-space: nowrap; }
  .saved, .manage { display: flex; flex-direction: column; gap: 6px; flex: 1; min-height: 0; }
  .sv { display: flex; align-items: center; gap: 12px; text-align: left; padding: 12px 14px; border-radius: 12px; }
  .sv .grow, .mrow .grow { display: flex; flex-direction: column; min-width: 0; }
  .mrow { display: flex; align-items: center; gap: 6px; padding: 6px 0; border-bottom: 1px solid var(--line); font-size: 13px; }
  .form { padding: 14px; display: flex; flex-direction: column; gap: 8px; background: rgba(0, 0, 0, 0.25); }
  .macros { display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px; }
  .macros label { display: flex; flex-direction: column; gap: 3px; font-size: 11px; color: var(--muted); }
  .gr { width: 70px; }
  .addp { align-self: flex-start; display: inline-flex; align-items: center; gap: 6px; font-size: 12.5px; }
  .addp :global(svg) { width: 13px; height: 13px; }
  .icon.sm { width: 26px; height: 26px; }
  .icon.sm :global(svg) { width: 13px; height: 13px; }
  .err { color: var(--danger); font-size: 12px; }
  /* refeição */
  .name { font-size: 16px; }
  .lines { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 8px; flex: 1; min-height: 0; }
  .lines li { padding: 10px 12px; border-radius: 12px; background: rgba(190, 240, 225, 0.05); display: flex; flex-direction: column; gap: 8px; }
  .lines li.hint { background: none; padding: 18px 0; }
  .lt { display: flex; justify-content: space-between; align-items: center; gap: 8px; font-size: 14px; font-weight: 500; }
  .qty { display: flex; align-items: center; gap: 6px; }
  .st { width: 34px; height: 34px; padding: 0; border-radius: 50%; font-size: 18px; flex: none; }
  .qv { width: 64px; text-align: center; font-size: 16px; }
  .qty select { flex: 1; min-width: 0; font-size: 12.5px; }
  .lk { font-size: 13px; color: var(--muted); width: 72px; text-align: right; flex: none; }
  .totals { display: grid; grid-template-columns: 1.3fr 1.1fr 1fr 1fr; gap: 8px; padding: 12px 0 4px; border-top: 1px solid var(--line); }
  .totals > div { display: flex; flex-direction: column; }
  .tv { font-size: 34px; font-weight: 300; line-height: 1; }
  .tv.p { color: var(--leaf); }
  .tv.sm2 { font-size: 22px; color: var(--muted); }
  .tl { font-size: 11px; color: var(--faint); margin-top: 3px; }
  .end { justify-content: flex-end; }
  .go { padding: 12px 28px; font-size: 16px; border-radius: 14px; }
  .done { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; text-align: center; }
  .done .big { font-size: 48px; color: var(--leaf); line-height: 1; }
  .done .row { width: 100%; justify-content: center; }
  .ok { color: var(--leaf); }
  @keyframes fade { from { opacity: 0; } }
</style>
