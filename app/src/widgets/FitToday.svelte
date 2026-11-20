<script lang="ts">
  // Academia · Hoje: calorias/macros do dia vs. meta, peso da manhã, refeições e treino.
  import { fit, loadFit, editDay, uid } from '../lib/fit.svelte';
  import { targets, dayTotals, itemFor, GROUPS, type Food, type Item } from '../lib/fit.ts';
  import { addDays, dayKey, parseDay, dayHeading } from '../lib/util';
  import Icon from '../components/Icon.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  $effect(() => { loadFit(); });

  let date = $state(''); // '' = hoje (acompanha a virada do dia)
  const d = $derived(date || fit.today);
  const log = $derived(fit.days[d]);
  const tg = $derived(targets(fit.profile));
  const tot = $derived(dayTotals(log));
  const pct = (a: number, b: number) => (b ? Math.min(100, (a / b) * 100) : 0);
  const fmt = (n: number) => Math.round(n).toLocaleString('pt-BR');
  const trained = $derived(Object.entries(log?.training ?? {}).filter(([, n]) => n > 0));
  const goodMeals = $derived(tot.meals >= 3 && tot.meals <= 4);
  const shift = (n: number) => { const k = dayKey(addDays(parseDay(d), n)); date = k === fit.today ? '' : k; };

  function setWeight(v: string) {
    const n = parseFloat(v.replace(',', '.'));
    editDay(d, x => { x.weight = Number.isFinite(n) && n > 20 && n < 400 ? Math.round(n * 10) / 10 : null; });
  }

  // ---------- refeições ----------
  let tplId = $state('');
  function logTemplate(id: string) {
    const t = fit.templates.find(x => x.id === id);
    if (!t) return;
    const items = t.items.flatMap(i => { const f = fit.foods.find(x => x.id === i.foodId); return f ? [itemFor(f, i.grams)] : []; });
    editDay(d, x => x.meals.push({ id: uid(), name: t.name, items }));
    tplId = '';
  }
  const delMeal = (id: string) => editDay(d, x => { x.meals = x.meals.filter(m => m.id !== id); });

  // montar refeição (expandido)
  let draft = $state<{ name: string; items: Item[] }>({ name: '', items: [] });
  let q = $state('');
  let grams = $state(100);
  const norm = (s: string) => s.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();
  const matches = $derived(q.trim() ? fit.foods.filter(f => norm(f.name).includes(norm(q.trim()))).slice(0, 8) : []);
  const addFood = (f: Food, g: number) => { if (g > 0) draft.items.push(itemFor(f, g)); };
  let manual = $state({ name: '', kcal: '', p: '' });
  function addManual() {
    const kcal = +manual.kcal, p = +(manual.p || 0);
    if (!(kcal > 0)) return;
    draft.items.push({ name: manual.name.trim() || 'Lançamento manual', grams: 0, kcal: Math.round(kcal), p, c: 0, f: 0 });
    manual = { name: '', kcal: '', p: '' };
  }
  const draftTot = $derived(dayTotals({ weight: null, training: {}, meals: [{ id: '', name: '', items: draft.items }] }));
  function saveMeal() {
    if (!draft.items.length) return;
    const name = draft.name.trim() || `Refeição ${(log?.meals.length ?? 0) + 1}`;
    editDay(d, x => x.meals.push({ id: uid(), name, items: draft.items }));
    draft = { name: '', items: [] };
  }

  // ---------- treino ----------
  const sets = (g: string) => log?.training[g] ?? 0;
  const addSets = (g: string, n: number) => editDay(d, x => { x.training[g] = Math.max(0, (x.training[g] ?? 0) + n); });
</script>

{#snippet bar(label: string, v: number, goal: number, unit: string, color: string)}
  <div class="mac">
    <div class="ml"><span>{label}</span><span class="mono">{fmt(v)}<span class="faint">/{fmt(goal)}{unit}</span></span></div>
    <div class="track"><i style="width:{pct(v, goal)}%;background:{color}"></i></div>
  </div>
{/snippet}

{#snippet summary(big: boolean)}
  <div class="kcal" class:big>
    <span class="mono kv">{fmt(tot.kcal)}</span><span class="faint"> / {fmt(tg.kcal)} kcal</span>
    {#if tg.estimated}<span class="tag" title="Comendo na manutenção estimada até confirmar (aba Plano)">descobrindo</span>{/if}
  </div>
  <div class="track main" class:over={tot.kcal > tg.kcal * 1.1}><i style="width:{pct(tot.kcal, tg.kcal)}%"></i></div>
  {@render bar('Proteína', tot.p, tg.protein, 'g', 'var(--leaf)')}
  <div class="two">
    {@render bar('Carbo', tot.c, tg.carbs, 'g', 'var(--amber)')}
    {@render bar('Gordura', tot.f, tg.fat, 'g', 'var(--blossom)')}
  </div>
{/snippet}

{#snippet weightRow()}
  <label class="wrow">
    <span class="label">Peso</span>
    {#key d}<input class="mono" inputmode="decimal" placeholder="—" value={log?.weight ?? ''} onchange={e => setWeight(e.currentTarget.value)} aria-label="Peso da manhã (kg)" />{/key}
    <span class="faint">kg</span>
  </label>
{/snippet}

{#if !fit.ready}
  <div class="empty">Carregando…</div>
{:else if !expanded}
  <div class="today">
    {@render summary(false)}
    <div class="row gap">
      {@render weightRow()}
      <span class="chip" class:ok={goodMeals} title="3 ou 4 refeições já é um dia bom">{tot.meals} {tot.meals === 1 ? 'refeição' : 'refeições'}{goodMeals ? ' ✓' : ''}</span>
    </div>
    <ul class="meals scroll">
      {#each log?.meals ?? [] as m (m.id)}
        <li><span class="ellipsis">{m.name}</span><span class="mono faint">{fmt(dayTotals({ weight: null, training: {}, meals: [m] }).kcal)} kcal</span></li>
      {:else}<li class="faint">Nenhuma refeição lançada.</li>{/each}
    </ul>
    {#if fit.templates.length}
      <div class="row gap">
        <select class="grow" bind:value={tplId} aria-label="Refeição cadastrada">
          <option value="">Lançar refeição cadastrada…</option>
          {#each fit.templates as t}<option value={t.id}>{t.name}</option>{/each}
        </select>
        <button class="primary" disabled={!tplId} onclick={() => logTemplate(tplId)}>Lançar</button>
      </div>
    {/if}
    <div class="trained">
      {#each trained as [g, n]}<span class="chip ok">{g} · {n}</span>{:else}<span class="faint">Sem treino registrado hoje.</span>{/each}
    </div>
  </div>
{:else}
  <div class="full">
    <section class="col">
      <div class="row datenav">
        <button class="ghost icon" aria-label="Dia anterior" onclick={() => shift(-1)}>‹</button>
        <span class="dh">{dayHeading(parseDay(d))}</span>
        <button class="ghost icon" aria-label="Próximo dia" disabled={d >= fit.today} onclick={() => shift(1)}>›</button>
      </div>
      {@render summary(true)}
      {@render weightRow()}
      <span class="label">Treino · séries</span>
      <div class="groups">
        {#each GROUPS as g}
          <div class="grp" class:on={sets(g) > 0}>
            <span class="ellipsis">{g}</span>
            <button class="ghost icon sm" aria-label="Menos uma série de {g}" disabled={!sets(g)} onclick={() => addSets(g, -1)}>−</button>
            <span class="mono n">{sets(g)}</span>
            <button class="ghost icon sm" aria-label="Mais uma série de {g}" onclick={() => addSets(g, 1)}>+</button>
          </div>
        {/each}
      </div>
    </section>

    <section class="col">
      <div class="row between"><span class="label">Refeições do dia</span><span class="chip" class:ok={goodMeals}>{tot.meals} {goodMeals ? '✓' : ''}</span></div>
      {#if fit.templates.length}
        <div class="tpls">
          {#each fit.templates as t}<button class="chip btn" onclick={() => logTemplate(t.id)}><Icon name="plus" /> {t.name}</button>{/each}
        </div>
      {/if}
      <ul class="mlist scroll">
        {#each log?.meals ?? [] as m (m.id)}
          {@const mt = dayTotals({ weight: null, training: {}, meals: [m] })}
          <li>
            <div class="row between"><b class="ellipsis">{m.name}</b>
              <span class="row"><span class="mono faint">{fmt(mt.kcal)} kcal · {fmt(mt.p)}g P</span>
                <button class="ghost icon danger sm" aria-label="Apagar {m.name}" onclick={() => delMeal(m.id)}><Icon name="trash" /></button></span>
            </div>
            <div class="faint small">{m.items.map(i => (i.grams ? `${i.name} ${i.grams}g` : i.name)).join(' · ')}</div>
          </li>
        {:else}<li class="faint">Nenhuma refeição lançada neste dia.</li>{/each}
      </ul>
    </section>

    <section class="col glass pane">
      <span class="label">Nova refeição</span>
      <input bind:value={draft.name} placeholder="Nome (ex.: Café da manhã)" aria-label="Nome da refeição" />
      <div class="row gap">
        <input class="grow" bind:value={q} placeholder="Buscar no banco de alimentos…" aria-label="Buscar alimento" />
        <input class="g mono" type="number" min="1" bind:value={grams} aria-label="Gramas" /><span class="faint">g</span>
      </div>
      {#each matches as f (f.id)}
        <div class="hit">
          <span class="ellipsis grow">{f.name} <span class="faint">· {f.kcal} kcal/100g</span></span>
          <button class="chip btn" onclick={() => addFood(f, grams)}>+ {grams} g</button>
          {#if f.portion}<button class="chip btn" onclick={() => addFood(f, f.portion!.grams)}>+ {f.portion.name}</button>{/if}
        </div>
      {/each}
      <details>
        <summary class="faint small">Lançar só calorias e proteína (ex.: comeu fora)</summary>
        <div class="row gap mt">
          <input class="grow" bind:value={manual.name} placeholder="O quê" aria-label="Descrição" />
          <input class="g mono" inputmode="numeric" bind:value={manual.kcal} placeholder="kcal" aria-label="Calorias" />
          <input class="g mono" inputmode="numeric" bind:value={manual.p} placeholder="P g" aria-label="Proteína (g)" />
          <button onclick={addManual}>+</button>
        </div>
      </details>
      <ul class="draft scroll">
        {#each draft.items as it, i}
          <li class="row between"><span class="ellipsis">{it.name}{it.grams ? ` · ${it.grams} g` : ''}</span>
            <span class="row"><span class="mono faint">{it.kcal} kcal</span>
              <button class="ghost icon sm" aria-label="Tirar {it.name}" onclick={() => draft.items.splice(i, 1)}><Icon name="x" /></button></span></li>
        {/each}
      </ul>
      <div class="row between">
        <span class="mono small">{fmt(draftTot.kcal)} kcal · P {fmt(draftTot.p)} · C {fmt(draftTot.c)} · G {fmt(draftTot.f)}</span>
        <button class="primary" disabled={!draft.items.length} onclick={saveMeal}>Lançar refeição</button>
      </div>
    </section>
  </div>
{/if}

<style>
  .today { height: 100%; display: flex; flex-direction: column; gap: 10px; min-height: 0; }
  .kcal { font-size: 13px; }
  .kv { font-size: 26px; font-weight: 300; }
  .big .kv { font-size: 34px; }
  .tag { margin-left: 8px; font-size: 10.5px; padding: 1px 8px; border-radius: 999px; background: rgba(232, 194, 124, 0.15); color: var(--amber); }
  .track { height: 5px; border-radius: 3px; background: rgba(190, 240, 225, 0.1); overflow: hidden; }
  .track i { display: block; height: 100%; border-radius: 3px; transition: width 0.4s; }
  .track.main { height: 7px; }
  .track.main i { background: linear-gradient(90deg, var(--teal), var(--leaf)); }
  .track.main.over i { background: var(--amber); }
  .mac { display: flex; flex-direction: column; gap: 4px; min-width: 0; }
  .ml { display: flex; justify-content: space-between; font-size: 12px; color: var(--muted); }
  .ml .mono { color: var(--text); }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .gap { gap: 8px; }
  .wrow { display: flex; align-items: center; gap: 8px; }
  .wrow input { width: 72px; text-align: right; }
  .chip { font-size: 11.5px; padding: 3px 10px; border-radius: 999px; background: rgba(190, 240, 225, 0.07); color: var(--muted); white-space: nowrap; border: 1px solid transparent; }
  .chip.ok { color: var(--leaf); background: rgba(207, 224, 142, 0.12); }
  .chip.btn { cursor: pointer; display: inline-flex; align-items: center; gap: 4px; }
  .chip.btn:hover { border-color: var(--line-strong); }
  .chip :global(svg) { width: 12px; height: 12px; }
  .meals { list-style: none; margin: 0; padding: 0; flex: 1; min-height: 0; display: flex; flex-direction: column; gap: 4px; }
  .meals li { display: flex; justify-content: space-between; gap: 8px; font-size: 13px; padding: 4px 0; border-bottom: 1px solid var(--line); }
  .trained { display: flex; flex-wrap: wrap; gap: 6px; font-size: 12px; }
  select { font-size: 12.5px; }
  /* expandido */
  .full { height: 100%; display: grid; grid-template-columns: 1fr 1.1fr 1.1fr; gap: 24px; min-height: 0; }
  .full section { min-height: 0; gap: 10px; }
  .pane { padding: 16px; overflow-y: auto; }
  .datenav { justify-content: space-between; }
  .dh { font-size: 15px; color: var(--leaf); text-transform: capitalize; }
  .groups { display: grid; grid-template-columns: 1fr 1fr; gap: 4px 12px; overflow-y: auto; }
  .grp { display: flex; align-items: center; gap: 4px; font-size: 12.5px; color: var(--muted); }
  .grp > span:first-child { flex: 1; }
  .grp.on { color: var(--text); }
  .grp .n { width: 18px; text-align: center; }
  .icon.sm { width: 24px; height: 24px; font-size: 15px; }
  .icon.sm :global(svg) { width: 13px; height: 13px; }
  .between { justify-content: space-between; }
  .tpls { display: flex; flex-wrap: wrap; gap: 6px; }
  .mlist, .draft { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; min-height: 0; }
  .mlist { flex: 1; }
  .mlist li { padding: 8px 0; border-bottom: 1px solid var(--line); }
  .small { font-size: 12px; }
  .g { width: 64px; }
  .hit { display: flex; align-items: center; gap: 6px; font-size: 12.5px; }
  .mt { margin-top: 6px; }
  .draft { flex: 1; font-size: 12.5px; }
  details summary { cursor: pointer; }
</style>
