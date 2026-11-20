<script lang="ts">
  // Academia · Comido: lançamentos do dia selecionado (o mesmo do card Hoje). Tocar abre para editar.
  import { fit, fitUi, fitDay, loadFit } from '../lib/fit.svelte';
  import { dayTotals } from '../lib/fit.ts';
  import Icon from '../components/Icon.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  $effect(() => { loadFit(); });

  const d = $derived(fitDay());
  const meals = $derived([...(fit.days[d]?.meals ?? [])].reverse());
  const tot = $derived(dayTotals(fit.days[d]));
  const n = (v: number) => Math.round(v).toLocaleString('pt-BR');
  const sum = (m: (typeof meals)[number]) => dayTotals({ weight: null, meals: [m] });
</script>

{#if !fit.ready}
  <div class="empty">Carregando…</div>
{:else}
  <div class="eaten" class:x={expanded}>
    {#if !meals.length}
      <div class="none">
        <span class="faint">Nada lançado {d === fit.today ? 'hoje' : 'neste dia'}.</span>
        <button class="primary" onclick={() => (fitUi.log = { mealId: null })}><Icon name="plus" /> Lançar refeição</button>
      </div>
    {:else}
      <ul class="scroll">
        {#each meals as m (m.id)}
          {@const s = sum(m)}
          <li>
            <button class="item" onclick={() => (fitUi.log = { mealId: m.id })} title="Editar">
              <span class="grow">
                <span class="nm ellipsis">{m.name}</span>
                <span class="sub ellipsis">{m.items.map(i => i.name).join(' · ')}</span>
              </span>
              <span class="kc mono">{n(s.kcal)}<small> kcal</small></span>
              <span class="pr mono">{n(s.p)}<small> g P</small></span>
            </button>
          </li>
        {/each}
      </ul>
      <div class="total">
        <span class="faint">Total</span>
        <span class="mono"><b>{n(tot.kcal)}</b> kcal · <b>{n(tot.p)}</b> g P · {n(tot.c)} g C · {n(tot.f)} g G</span>
      </div>
    {/if}
  </div>
{/if}

<style>
  .eaten { height: 100%; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  .none { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 12px; }
  .none button { display: inline-flex; align-items: center; gap: 6px; border-radius: 999px; padding: 8px 16px; }
  .none :global(svg) { width: 15px; height: 15px; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; flex: 1; min-height: 0; }
  .item { width: 100%; display: flex; align-items: center; gap: 12px; padding: 8px 10px; text-align: left; border-radius: 10px; background: rgba(190, 240, 225, 0.04); border-color: transparent; }
  .item:hover { border-color: var(--line-strong); }
  .grow { display: flex; flex-direction: column; min-width: 0; }
  .nm { font-size: 14px; font-weight: 500; }
  .x .nm { font-size: 16px; }
  .sub { font-size: 11.5px; color: var(--faint); }
  .kc { font-size: 16px; width: 76px; text-align: right; }
  .pr { font-size: 13px; color: var(--leaf); width: 60px; text-align: right; }
  small { font-size: 10.5px; color: var(--muted); }
  .total { display: flex; justify-content: space-between; gap: 10px; font-size: 12.5px; padding-top: 6px; border-top: 1px solid var(--line); }
</style>
