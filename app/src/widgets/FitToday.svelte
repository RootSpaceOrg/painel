<script lang="ts">
  // Academia · Hoje: anel de calorias (quanto falta) + anéis de proteína, carbo e gordura; botões de lançar e pesar.
  import { fit, fitUi, fitDay, loadFit, fitTargets } from '../lib/fit.svelte';
  import { dayTotals } from '../lib/fit.ts';
  import { addDays, dayKey, parseDay, dayHeading } from '../lib/util';
  import Ring from '../components/Ring.svelte';
  import Icon from '../components/Icon.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  $effect(() => { loadFit(); });

  const d = $derived(fitDay());
  const tg = $derived(fitTargets());
  const tot = $derived(dayTotals(fit.days[d]));
  const weight = $derived(fit.days[d]?.weight ?? null);
  const left = $derived(tg.kcal - tot.kcal);
  const n = (v: number) => Math.round(v).toLocaleString('pt-BR');
  const kg = (v: number) => v.toLocaleString('pt-BR', { minimumFractionDigits: 1, maximumFractionDigits: 1 });
  const shift = (k: number) => { const x = dayKey(addDays(parseDay(d), k)); fitUi.date = x >= fit.today ? '' : x; };
  const big = $derived(expanded ? 300 : 210);
  const small = $derived(expanded ? 120 : 84);
</script>

{#snippet macro(label: string, v: number, goal: number, color: string)}
  <div class="mac">
    <Ring value={v} {goal} {color} size={small} stroke={expanded ? 10 : 8}>
      <span class="mv mono">{n(v)}</span>
      <span class="mg">de {n(goal)} g</span>
    </Ring>
    <span class="ml">{label}</span>
  </div>
{/snippet}

{#if !fit.ready}
  <div class="empty">Carregando…</div>
{:else}
  <div class="today" class:x={expanded}>
    <div class="nav">
      <button class="ghost icon" aria-label="Dia anterior" onclick={() => shift(-1)}>‹</button>
      <span class="dh">{dayHeading(parseDay(d))}</span>
      <button class="ghost icon" aria-label="Próximo dia" disabled={d >= fit.today} onclick={() => shift(1)}>›</button>
    </div>

    <Ring value={tot.kcal} goal={tg.kcal} size={big} stroke={expanded ? 22 : 16} color="var(--teal)">
      <span class="bl">{left >= 0 ? 'faltam' : 'passou'}</span>
      <span class="bn mono" class:over={left < 0}>{n(Math.abs(left))}</span>
      <span class="bs">{n(tot.kcal)} de {n(tg.kcal)} kcal</span>
    </Ring>

    <div class="macs">
      {@render macro('Proteína', tot.p, tg.protein, 'var(--leaf)')}
      {@render macro('Carbo', tot.c, tg.carbs, 'var(--amber)')}
      {@render macro('Gordura', tot.f, tg.fat, 'var(--blossom)')}
    </div>

    <div class="acts">
      <button class="primary act" onclick={() => (fitUi.log = { mealId: null })}><Icon name="plus" /> Lançar refeição</button>
      <button class="act" onclick={() => (fitUi.weight = true)}>
        <span class="sc">⚖</span>
        {#if weight !== null}<span><b class="mono">{kg(weight)} kg</b> <span class="ok">✓</span></span>{:else}Registrar peso{/if}
      </button>
    </div>
  </div>
{/if}

<style>
  .today { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: space-between; gap: 12px; min-height: 0; }
  .nav { display: flex; align-items: center; gap: 14px; }
  .dh { min-width: 150px; text-align: center; font-size: 15px; color: var(--leaf); text-transform: capitalize; }
  .bl { font-size: 13px; color: var(--muted); }
  .bn { font-size: 52px; font-weight: 300; line-height: 1.05; letter-spacing: -0.02em; }
  .x .bn { font-size: 76px; }
  .bn.over { color: var(--amber); }
  .bs { font-size: 12px; color: var(--faint); margin-top: 2px; }
  .x .bs { font-size: 14px; }
  .macs { display: flex; justify-content: center; gap: 22px; }
  .x .macs { gap: 48px; }
  .mac { display: flex; flex-direction: column; align-items: center; gap: 6px; }
  .mv { font-size: 18px; font-weight: 400; line-height: 1; }
  .x .mv { font-size: 26px; }
  .mg { font-size: 10px; color: var(--faint); margin-top: 2px; }
  .x .mg { font-size: 12px; }
  .ml { font-size: 12px; color: var(--muted); }
  .acts { width: 100%; display: grid; grid-template-columns: 1.3fr 1fr; gap: 10px; }
  .act { display: flex; align-items: center; justify-content: center; gap: 8px; padding: 14px 10px; font-size: 15px; border-radius: 14px; }
  .act :global(svg) { width: 18px; height: 18px; }
  .sc { font-size: 17px; }
  .ok { color: var(--leaf); }
</style>
