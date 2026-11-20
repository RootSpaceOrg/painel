<script lang="ts">
  // Academia · Hoje: quanto falta de calorias/proteína (números grandes), peso com − / + e lançamentos de um toque.
  import { fit, loadFit, editDay, uid, fitTargets } from '../lib/fit.svelte';
  import { dayTotals, itemFor, lastWeight, recentItems, type Food, type Item } from '../lib/fit.ts';
  import { addDays, dayKey, parseDay, dayHeading } from '../lib/util';
  import Icon from '../components/Icon.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  $effect(() => { loadFit(); });

  let date = $state(''); // '' = hoje (acompanha a virada do dia)
  const d = $derived(date || fit.today);
  const log = $derived(fit.days[d]);
  const tg = $derived(fitTargets());
  const tot = $derived(dayTotals(log));
  const n = (v: number) => Math.round(v).toLocaleString('pt-BR');
  const kg = (v: number) => v.toLocaleString('pt-BR', { minimumFractionDigits: 1, maximumFractionDigits: 1 });
  const pct = (a: number, b: number) => (b ? Math.min(100, (a / b) * 100) : 0);
  const shift = (k: number) => { const x = dayKey(addDays(parseDay(d), k)); date = x === fit.today ? '' : x; };

  // ---------- peso: − / + partem do último peso registrado ----------
  const saved = $derived(log?.weight ?? null);
  const shownWeight = $derived(saved ?? lastWeight(fit.days, d) ?? fit.profile.weight);
  const setWeight = (v: number) => editDay(d, x => { x.weight = Math.round(Math.min(400, Math.max(20, v)) * 10) / 10; });
  let typing = $state(false);
  function typed(v: string) {
    typing = false;
    const x = parseFloat(v.replace(',', '.'));
    if (Number.isFinite(x) && x > 20 && x < 400) setWeight(x);
  }

  // ---------- lançar ----------
  const add = (name: string, items: Item[]) => { if (items.length) editDay(d, x => x.meals.push({ id: uid(), name, items })); };
  const tplItems = (id: string) =>
    (fit.templates.find(t => t.id === id)?.items ?? []).flatMap(i => { const f = fit.foods.find(x => x.id === i.foodId); return f ? [itemFor(f, i.grams)] : []; });
  const tplKcal = (id: string) => tplItems(id).reduce((a, i) => a + i.kcal, 0);
  const recents = $derived(recentItems(fit.days, d, expanded ? 10 : 6));
  const remove = (id: string) => editDay(d, x => { x.meals = x.meals.filter(m => m.id !== id); });

  // "+ Outro": do banco (toque no alimento → toque nas gramas) ou só calorias
  let other = $state<null | 'bank' | 'kcal'>(null);
  let q = $state('');
  let picked = $state<Food | null>(null);
  let customG = $state('');
  let man = $state({ name: '', kcal: '', p: '' });
  const norm = (s: string) => s.normalize('NFD').replace(/[̀-ͯ]/g, '').toLowerCase();
  const results = $derived(fit.foods.filter(f => !q.trim() || norm(f.name).includes(norm(q.trim()))).slice(0, 8));
  const close = () => { other = null; q = ''; picked = null; customG = ''; man = { name: '', kcal: '', p: '' }; };
  function addGrams(g: number) {
    if (!picked || !(g > 0)) return;
    const it = itemFor(picked, g);
    add(picked.name, [it]);
    close();
  }
  function addKcal() {
    const kcal = Math.round(+man.kcal.replace(',', '.'));
    if (!(kcal > 0)) return;
    const p = +(man.p || '0').replace(',', '.') || 0;
    const name = man.name.trim() || 'Lançamento';
    add(name, [{ name, grams: 0, kcal, p, c: 0, f: 0 }]);
    close();
  }
</script>

{#snippet big(label: string, eaten: number, goal: number, unit: string, cls: string)}
  {@const left = goal - eaten}
  <div class="big {cls}" class:over={left < 0}>
    <span class="label">{label}</span>
    <span class="bl">{left >= 0 ? 'faltam' : 'passou'}</span>
    <span class="bn mono">{n(Math.abs(left))}<small>{unit}</small></span>
    <div class="bar"><i style="width:{pct(eaten, goal)}%"></i></div>
    <span class="bs">{n(eaten)} de {n(goal)}{unit}</span>
  </div>
{/snippet}

{#if !fit.ready}
  <div class="empty">Carregando…</div>
{:else}
  <div class="fit" class:x={expanded}>
    {#if expanded}
      <div class="row datenav">
        <button class="ghost icon" aria-label="Dia anterior" onclick={() => shift(-1)}>‹</button>
        <span class="dh">{dayHeading(parseDay(d))}</span>
        <button class="ghost icon" aria-label="Próximo dia" disabled={d >= fit.today} onclick={() => shift(1)}>›</button>
      </div>
    {/if}

    <div class="bigs">
      {@render big('Calorias', tot.kcal, tg.kcal, '', 'kc')}
      {@render big('Proteína', tot.p, tg.protein, ' g', 'pr')}
    </div>

    <!-- peso -->
    <div class="weight" class:unsaved={saved === null}>
      <span class="label">Peso {d === fit.today ? 'hoje' : 'do dia'}</span>
      <button class="step" aria-label="Menos 100 g" onclick={() => setWeight(shownWeight - 0.1)}>−</button>
      {#if typing}
        <!-- svelte-ignore a11y_autofocus -->
        <input class="wv mono" inputmode="decimal" value={kg(shownWeight)} autofocus aria-label="Peso (kg)"
          onblur={e => typed(e.currentTarget.value)} onkeydown={e => e.key === 'Enter' && e.currentTarget.blur()} />
      {:else}
        <button class="wv mono" title="Clique para digitar" onclick={() => (typing = true)}>{kg(shownWeight)}<small> kg</small></button>
      {/if}
      <button class="step" aria-label="Mais 100 g" onclick={() => setWeight(shownWeight + 0.1)}>+</button>
      {#if saved === null}
        <button class="primary reg" onclick={() => setWeight(shownWeight)}>Registrar</button>
      {:else}
        <span class="ok">✓ salvo</span>
      {/if}
    </div>

    <!-- adicionar -->
    {#if !other}
      <div class="adds">
        {#each fit.templates as t (t.id)}
          <button class="add tpl" onclick={() => add(t.name, tplItems(t.id))}>
            <span class="an ellipsis">{t.name}</span><span class="ak mono">{n(tplKcal(t.id))} kcal</span>
          </button>
        {/each}
        {#each recents as it}
          <button class="add" onclick={() => add(it.name, [{ ...it }])} title="Lançar de novo">
            <span class="an ellipsis">{it.name}</span><span class="ak mono">{it.grams ? `${it.grams} g · ` : ''}{n(it.kcal)} kcal</span>
          </button>
        {/each}
        <button class="add more" onclick={() => (other = 'bank')}><Icon name="plus" /> Outro</button>
      </div>
    {:else}
      <div class="other glass">
        <div class="row between">
          <div class="row seg">
            <button class:on={other === 'bank'} onclick={() => { other = 'bank'; picked = null; }}>Do banco</button>
            <button class:on={other === 'kcal'} onclick={() => (other = 'kcal')}>Só calorias</button>
          </div>
          <button class="ghost icon" aria-label="Cancelar" onclick={close}><Icon name="x" /></button>
        </div>
        {#if other === 'bank' && !picked}
          <!-- svelte-ignore a11y_autofocus -->
          <input bind:value={q} placeholder="Buscar alimento…" aria-label="Buscar alimento" autofocus />
          <div class="picks scroll">
            {#each results as f (f.id)}
              <button class="pick" onclick={() => (picked = f)}><span class="ellipsis">{f.name}</span><span class="mono faint">{f.kcal} kcal/100 g</span></button>
            {:else}<span class="faint">Nada no banco. Cadastre em "Alimentos e refeições".</span>{/each}
          </div>
        {:else if other === 'bank' && picked}
          <div class="pname">{picked.name} <span class="faint mono">· {picked.kcal} kcal/100 g</span></div>
          <div class="grams">
            {#if picked.portion}<button class="g primary" onclick={() => addGrams(picked!.portion!.grams)}>{picked.portion.name}<small>{picked.portion.grams} g</small></button>{/if}
            {#each [50, 100, 150, 200, 300] as g}
              <button class="g" onclick={() => addGrams(g)}>{g} g<small>{n(itemFor(picked, g).kcal)} kcal</small></button>
            {/each}
            <form class="g custom" onsubmit={e => { e.preventDefault(); addGrams(+customG); }}>
              <input class="mono" inputmode="numeric" bind:value={customG} placeholder="g" aria-label="Outra quantidade em gramas" /><button type="submit">ok</button>
            </form>
          </div>
          <button class="ghost back" onclick={() => (picked = null)}>‹ outro alimento</button>
        {:else}
          <form class="kcal" onsubmit={e => { e.preventDefault(); addKcal(); }}>
            <input class="mono bigin" inputmode="numeric" bind:value={man.kcal} placeholder="kcal" aria-label="Calorias" />
            <input class="mono bigin" inputmode="numeric" bind:value={man.p} placeholder="proteína g" aria-label="Proteína (g)" />
            <input bind:value={man.name} placeholder="o quê (opcional)" aria-label="Descrição" />
            <button class="primary" type="submit">Adicionar</button>
          </form>
        {/if}
      </div>
    {/if}

    <!-- lançados -->
    <div class="label">Lançado {d === fit.today ? 'hoje' : 'no dia'} · {tot.meals}</div>
    <ul class="list scroll">
      {#each [...(log?.meals ?? [])].reverse() as m (m.id)}
        {@const mt = dayTotals({ weight: null, meals: [m] })}
        <li>
          <span class="ellipsis grow">{m.name}</span>
          <span class="mono faint">{n(mt.kcal)} kcal · {n(mt.p)} g</span>
          <button class="ghost icon del" aria-label="Apagar {m.name}" onclick={() => remove(m.id)}><Icon name="x" /></button>
        </li>
      {:else}<li class="faint">Nada lançado ainda. Toque num botão acima.</li>{/each}
    </ul>
  </div>
{/if}

<style>
  .fit { height: 100%; display: flex; flex-direction: column; gap: 12px; min-height: 0; }
  .datenav { justify-content: center; gap: 16px; }
  .dh { font-size: 16px; color: var(--leaf); text-transform: capitalize; min-width: 140px; text-align: center; }

  /* números grandes */
  .bigs { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
  .big { display: flex; flex-direction: column; gap: 2px; padding: 12px 14px; border-radius: 14px; background: rgba(190, 240, 225, 0.05); border: 1px solid var(--line); min-width: 0; }
  .bl { font-size: 12px; color: var(--muted); margin-top: 4px; }
  .bn { font-size: 46px; font-weight: 300; line-height: 1; letter-spacing: -0.02em; }
  .x .bn { font-size: 64px; }
  .bn small { font-size: 18px; color: var(--muted); }
  .bs { font-size: 12px; color: var(--faint); }
  .bar { height: 6px; border-radius: 3px; background: rgba(190, 240, 225, 0.1); overflow: hidden; margin: 8px 0 4px; }
  .bar i { display: block; height: 100%; border-radius: 3px; transition: width 0.4s; }
  .kc .bar i { background: linear-gradient(90deg, var(--teal), var(--leaf)); }
  .pr .bar i { background: var(--leaf); }
  .big.over .bl, .big.over .bn { color: var(--amber); }
  .big.over .bar i { background: var(--amber); }

  /* peso */
  .weight { display: flex; align-items: center; gap: 10px; padding: 8px 12px; border-radius: 14px; background: rgba(190, 240, 225, 0.05); border: 1px solid var(--line); }
  .weight .label { flex: 1; }
  .step { width: 40px; height: 40px; padding: 0; border-radius: 50%; font-size: 22px; line-height: 1; }
  .wv { font-size: 30px; font-weight: 300; background: none; border: 0; padding: 0 4px; min-width: 120px; text-align: center; }
  input.wv { width: 120px; border: 1px solid var(--line-strong); border-radius: 10px; }
  .wv small { font-size: 14px; color: var(--muted); }
  .unsaved .wv { color: var(--muted); }
  .reg { border-radius: 999px; }
  .ok { color: var(--leaf); font-size: 13px; min-width: 70px; }

  /* botões de lançar */
  .adds { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 8px; }
  .add { display: flex; flex-direction: column; align-items: flex-start; gap: 2px; padding: 10px 12px; border-radius: 12px; text-align: left; min-width: 0; }
  .add.tpl { background: rgba(127, 224, 208, 0.1); border-color: rgba(127, 224, 208, 0.3); }
  .add:active { transform: scale(0.97); }
  .an { font-size: 13.5px; font-weight: 500; max-width: 100%; }
  .ak { font-size: 11.5px; color: var(--muted); }
  .add.more { flex-direction: row; align-items: center; justify-content: center; gap: 6px; color: var(--teal); border-style: dashed; }
  .add.more :global(svg) { width: 15px; height: 15px; }

  /* + Outro */
  .other { padding: 12px; display: flex; flex-direction: column; gap: 10px; background: var(--glass-strong); }
  .between { justify-content: space-between; }
  .seg button { padding: 5px 14px; border-radius: 999px; font-size: 13px; }
  .seg .on { color: var(--ink); background: var(--teal); border-color: transparent; }
  .picks { display: flex; flex-direction: column; gap: 4px; max-height: 210px; }
  .pick { display: flex; justify-content: space-between; gap: 10px; text-align: left; padding: 9px 12px; }
  .pname { font-size: 15px; font-weight: 500; }
  .grams { display: grid; grid-template-columns: repeat(auto-fill, minmax(92px, 1fr)); gap: 8px; }
  .g { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 10px 6px; font-size: 16px; border-radius: 12px; }
  .g small { font-size: 11px; color: var(--muted); }
  .g.primary small { color: var(--teal); }
  .custom { flex-direction: row; padding: 4px; gap: 4px; border: 1px solid var(--line); }
  .custom input { width: 100%; text-align: center; }
  .custom button { padding: 4px 8px; font-size: 12px; }
  .back { align-self: flex-start; font-size: 12px; }
  .kcal { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .bigin { font-size: 22px; text-align: center; }
  .kcal input:not(.bigin) { grid-column: 1 / -1; }
  .kcal button { grid-column: 1 / -1; padding: 10px; }

  /* lançados */
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 2px; flex: 1; min-height: 0; }
  .list li { display: flex; align-items: center; gap: 10px; font-size: 13.5px; padding: 6px 0; border-bottom: 1px solid var(--line); }
  .del { width: 26px; height: 26px; }
  .del :global(svg) { width: 13px; height: 13px; }
</style>
