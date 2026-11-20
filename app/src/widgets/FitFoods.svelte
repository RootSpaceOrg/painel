<script lang="ts">
  // Academia · Alimentos: banco de alimentos (por 100 g/ml) e refeições cadastradas para lançar com um clique.
  import { fit, loadFit, saveFoods, saveTemplates, uid } from '../lib/fit.svelte';
  import { itemFor, dayTotals, type Food, type Template } from '../lib/fit.ts';
  import { foodSearch, type FoodHit } from '../lib/api';
  import Icon from '../components/Icon.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  $effect(() => { loadFit(); });

  const norm = (s: string) => s.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();
  const n = (v: number) => Math.round(v).toLocaleString('pt-BR');
  const foodById = (id: string) => fit.foods.find(f => f.id === id);
  const tplTotals = (t: Template) =>
    dayTotals({ weight: null, meals: [{ id: '', name: '', items: t.items.flatMap(i => { const f = foodById(i.foodId); return f ? [itemFor(f, i.grams)] : []; }) }] });

  // ---------- banco ----------
  let filter = $state('');
  const shown = $derived(fit.foods.filter(f => !filter.trim() || norm(f.name).includes(norm(filter.trim()))));
  type FoodForm = { id?: string; name: string; kcal: string; p: string; c: string; f: string; pname: string; pgrams: string };
  const blank = (): FoodForm => ({ name: '', kcal: '', p: '', c: '', f: '', pname: '', pgrams: '' });
  let form = $state<FoodForm>(blank());
  let err = $state('');
  const num = (s: string) => { const x = parseFloat(String(s).replace(',', '.')); return Number.isFinite(x) && x >= 0 ? x : NaN; };
  function editFood(f: Food) {
    form = { id: f.id, name: f.name, kcal: String(f.kcal), p: String(f.p), c: String(f.c), f: String(f.f), pname: f.portion?.name ?? '', pgrams: f.portion ? String(f.portion.grams) : '' };
  }
  function saveFood() {
    err = '';
    const [kcal, p, c, f] = [num(form.kcal), num(form.p || '0'), num(form.c || '0'), num(form.f || '0')];
    if (!form.name.trim() || Number.isNaN(kcal) || [p, c, f].some(Number.isNaN)) { err = 'Preencha nome e calorias (valores por 100 g).'; return; }
    const pg = num(form.pgrams);
    const food: Food = {
      id: form.id ?? uid(), name: form.name.trim(), kcal: Math.round(kcal), p, c, f,
      portion: form.pname.trim() && pg > 0 ? { name: form.pname.trim(), grams: pg } : null,
      source: form.id ? foodById(form.id)?.source ?? 'manual' : 'manual',
    };
    const i = fit.foods.findIndex(x => x.id === food.id);
    if (i >= 0) fit.foods[i] = food; else fit.foods.unshift(food);
    saveFoods();
    form = blank();
  }
  function delFood(f: Food) {
    const used = fit.templates.filter(t => t.items.some(i => i.foodId === f.id)).map(t => t.name);
    if (!confirm(used.length ? `"${f.name}" está em: ${used.join(', ')}. Excluir mesmo assim?` : `Excluir "${f.name}"?`)) return;
    fit.foods = fit.foods.filter(x => x.id !== f.id);
    saveFoods();
  }

  // ---------- Open Food Facts ----------
  let oq = $state('');
  let hits = $state<FoodHit[]>([]);
  let searching = $state(false);
  let oerr = $state('');
  async function search() {
    if (!oq.trim()) return;
    searching = true; oerr = ''; hits = [];
    try { hits = await foodSearch(oq.trim()); if (!hits.length) oerr = 'Nada encontrado com calorias informadas.'; }
    catch (e) { oerr = `Não deu para buscar: ${e}`; }
    searching = false;
  }
  function importHit(h: FoodHit) {
    const name = h.brand ? `${h.name} (${h.brand})` : h.name;
    fit.foods.unshift({ id: uid(), name, kcal: h.kcal, p: h.p, c: h.c, f: h.f, portion: null, source: 'Open Food Facts' });
    saveFoods();
    hits = hits.filter(x => x !== h);
  }

  // ---------- refeições cadastradas ----------
  let tpl = $state<{ id?: string; name: string; items: { foodId: string; grams: number }[] }>({ name: '', items: [] });
  let tq = $state('');
  let tg = $state(100);
  const tmatches = $derived(tq.trim() ? fit.foods.filter(f => norm(f.name).includes(norm(tq.trim()))).slice(0, 6) : []);
  const tplDraftTotals = $derived(tplTotals({ id: '', name: '', items: tpl.items }));
  function saveTpl() {
    if (!tpl.name.trim() || !tpl.items.length) return;
    const t: Template = { id: tpl.id ?? uid(), name: tpl.name.trim(), items: tpl.items.map(i => ({ ...i })) };
    const i = fit.templates.findIndex(x => x.id === t.id);
    if (i >= 0) fit.templates[i] = t; else fit.templates.push(t);
    saveTemplates();
    tpl = { name: '', items: [] };
  }
  function delTpl(t: Template) {
    if (!confirm(`Excluir a refeição "${t.name}"?`)) return;
    fit.templates = fit.templates.filter(x => x.id !== t.id);
    saveTemplates();
  }
</script>

{#if !fit.ready}
  <div class="empty">Carregando…</div>
{:else if !expanded}
  <div class="mini">
    <div class="faint small">{fit.foods.length} alimentos · {fit.templates.length} refeições cadastradas</div>
    <ul class="scroll">
      {#each fit.templates as t (t.id)}
        {@const s = tplTotals(t)}
        <li><span class="ellipsis">{t.name}</span><span class="mono faint">{n(s.kcal)} kcal · {n(s.p)}g P</span></li>
      {:else}
        <li class="faint">Abra para cadastrar refeições (ex.: "Café da manhã") e lançar com um clique na aba Hoje.</li>
      {/each}
    </ul>
  </div>
{:else}
  <div class="full">
    <!-- banco -->
    <section class="col">
      <div class="row between"><span class="label">Banco de alimentos · por 100 g</span><span class="faint small">{fit.foods.length}</span></div>
      <input bind:value={filter} placeholder="Filtrar…" aria-label="Filtrar alimentos" />
      <ul class="list scroll">
        {#each shown as f (f.id)}
          <li>
            <div class="grow">
              <div class="ellipsis">{f.name}</div>
              <div class="faint small mono">{f.kcal} kcal · P {f.p} · C {f.c} · G {f.f}{f.portion ? ` · ${f.portion.name} = ${f.portion.grams} g` : ''} · {f.source}</div>
            </div>
            <button class="ghost icon sm" aria-label="Editar {f.name}" onclick={() => editFood(f)}><Icon name="edit" /></button>
            <button class="ghost icon sm danger" aria-label="Excluir {f.name}" onclick={() => delFood(f)}><Icon name="trash" /></button>
          </li>
        {/each}
      </ul>
    </section>

    <section class="col">
      <div class="glass pane col">
        <span class="label">{form.id ? 'Editar alimento' : 'Novo alimento'} · por 100 g</span>
        <input bind:value={form.name} placeholder="Nome" aria-label="Nome do alimento" />
        <div class="macros">
          <input class="mono" inputmode="decimal" bind:value={form.kcal} placeholder="kcal" aria-label="Calorias por 100 g" />
          <input class="mono" inputmode="decimal" bind:value={form.p} placeholder="Proteína" aria-label="Proteína por 100 g" />
          <input class="mono" inputmode="decimal" bind:value={form.c} placeholder="Carbo" aria-label="Carboidrato por 100 g" />
          <input class="mono" inputmode="decimal" bind:value={form.f} placeholder="Gordura" aria-label="Gordura por 100 g" />
        </div>
        <div class="row">
          <input class="grow" bind:value={form.pname} placeholder="Porção (opcional, ex.: 1 scoop)" aria-label="Nome da porção" />
          <input class="mono g" inputmode="decimal" bind:value={form.pgrams} placeholder="g" aria-label="Gramas da porção" />
        </div>
        {#if err}<span class="err">{err}</span>{/if}
        <div class="row">
          <button class="primary" onclick={saveFood}>{form.id ? 'Salvar' : 'Adicionar ao banco'}</button>
          {#if form.id}<button class="ghost" onclick={() => (form = blank())}>Cancelar</button>{/if}
        </div>
      </div>
      <div class="glass pane col">
        <span class="label">Buscar produto (Open Food Facts)</span>
        <form class="row" onsubmit={e => { e.preventDefault(); search(); }}>
          <input class="grow" bind:value={oq} placeholder="ex.: whey, iogurte grego, pasta de amendoim" aria-label="Buscar produto" />
          <button type="submit" disabled={searching}>{searching ? 'Buscando…' : 'Buscar'}</button>
        </form>
        {#if oerr}<span class="faint small">{oerr}</span>{/if}
        <ul class="list scroll hits">
          {#each hits as h}
            <li>
              <div class="grow"><div class="ellipsis">{h.name}{h.brand ? ` · ${h.brand}` : ''}</div>
                <div class="faint small mono">{h.kcal} kcal · P {h.p} · C {h.c} · G {h.f}</div></div>
              <button class="chip" onclick={() => importHit(h)}>+ Banco</button>
            </li>
          {/each}
        </ul>
        <span class="faint small">Dados de uma base aberta e colaborativa: confira com o rótulo.</span>
      </div>
    </section>

    <!-- refeições cadastradas -->
    <section class="col">
      <span class="label">Refeições cadastradas</span>
      <ul class="list scroll tpls">
        {#each fit.templates as t (t.id)}
          {@const s = tplTotals(t)}
          <li>
            <div class="grow">
              <div class="ellipsis">{t.name} <span class="faint mono small">· {n(s.kcal)} kcal · P {n(s.p)}g</span></div>
              <div class="faint small">{t.items.map(i => `${foodById(i.foodId)?.name ?? '(removido)'} ${i.grams}g`).join(' · ')}</div>
            </div>
            <button class="ghost icon sm" aria-label="Editar {t.name}" onclick={() => (tpl = { id: t.id, name: t.name, items: t.items.map(i => ({ ...i })) })}><Icon name="edit" /></button>
            <button class="ghost icon sm danger" aria-label="Excluir {t.name}" onclick={() => delTpl(t)}><Icon name="trash" /></button>
          </li>
        {:else}<li class="faint small">Nenhuma ainda.</li>{/each}
      </ul>
      <div class="glass pane col">
        <span class="label">{tpl.id ? 'Editar refeição' : 'Nova refeição'}</span>
        <input bind:value={tpl.name} placeholder="Nome (ex.: Café da manhã)" aria-label="Nome da refeição" />
        <div class="row">
          <input class="grow" bind:value={tq} placeholder="Alimento do banco…" aria-label="Buscar alimento" />
          <input class="mono g" type="number" min="1" bind:value={tg} aria-label="Gramas" /><span class="faint">g</span>
        </div>
        {#each tmatches as f (f.id)}
          <div class="row hit">
            <span class="ellipsis grow">{f.name}</span>
            <button class="chip" onclick={() => tpl.items.push({ foodId: f.id, grams: tg })}>+ {tg} g</button>
            {#if f.portion}<button class="chip" onclick={() => tpl.items.push({ foodId: f.id, grams: f.portion!.grams })}>+ {f.portion.name}</button>{/if}
          </div>
        {/each}
        {#each tpl.items as it, i}
          <div class="row hit">
            <span class="ellipsis grow">{foodById(it.foodId)?.name ?? '(removido)'} · {it.grams} g</span>
            <button class="ghost icon sm" aria-label="Tirar" onclick={() => tpl.items.splice(i, 1)}><Icon name="x" /></button>
          </div>
        {/each}
        <div class="row between">
          <span class="mono small">{n(tplDraftTotals.kcal)} kcal · P {n(tplDraftTotals.p)} · C {n(tplDraftTotals.c)} · G {n(tplDraftTotals.f)}</span>
          <span class="row">
            {#if tpl.id}<button class="ghost" onclick={() => (tpl = { name: '', items: [] })}>Cancelar</button>{/if}
            <button class="primary" disabled={!tpl.name.trim() || !tpl.items.length} onclick={saveTpl}>Salvar</button>
          </span>
        </div>
      </div>
    </section>
  </div>
{/if}

<style>
  .mini { height: 100%; display: flex; flex-direction: column; gap: 6px; min-height: 0; }
  .mini ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; flex: 1; min-height: 0; }
  .mini li { display: flex; justify-content: space-between; gap: 8px; font-size: 13px; padding: 3px 0; border-bottom: 1px solid var(--line); }
  .small { font-size: 11.5px; }
  .between { justify-content: space-between; }
  .full { height: 100%; display: grid; grid-template-columns: 1.1fr 1fr 1fr; gap: 22px; min-height: 0; }
  .full section { min-height: 0; gap: 10px; overflow-y: auto; }
  .pane { padding: 14px; gap: 8px; }
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; min-height: 0; }
  .list li { display: flex; align-items: center; gap: 6px; padding: 6px 0; border-bottom: 1px solid var(--line); font-size: 13px; }
  .hits { max-height: 220px; }
  .macros { display: grid; grid-template-columns: repeat(4, 1fr); gap: 6px; }
  .g { width: 64px; }
  .hit { font-size: 12.5px; gap: 6px; }
  .icon.sm { width: 26px; height: 26px; }
  .icon.sm :global(svg) { width: 13px; height: 13px; }
  .chip { font-size: 11.5px; padding: 3px 10px; border-radius: 999px; white-space: nowrap; }
  .err { color: var(--danger); font-size: 12px; }
</style>
