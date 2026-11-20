<script lang="ts">
  // Academia · Peso: média de 7 dias (número grande), variação, dias na meta e gráfico.
  import { fit, loadFit, fitTargets } from '../lib/fit.svelte';
  import { movingAvg, weightTrend, adherence, dayTotals, lastDays } from '../lib/fit.ts';
  import { parseDay, fmtShort } from '../lib/util';

  let { expanded = false }: { expanded?: boolean } = $props();
  $effect(() => { loadFit(); });

  const tg = $derived(fitTargets());
  const span = $derived(expanded ? 28 : 14);
  const series = $derived(movingAvg(fit.days, fit.today, span));
  const tr = $derived(weightTrend(fit.days, fit.today));
  const ad = $derived(adherence(fit.days, fit.today, tg));
  const goal = $derived(fit.profile.targetWeight);
  const kg = (n: number | null) => (n === null ? '—' : n.toLocaleString('pt-BR', { minimumFractionDigits: 1, maximumFractionDigits: 1 }));

  // gráfico: pontos = pesagens do dia; linha = média móvel; tracejado = peso meta
  const W = 100, H = 40;
  const ys = $derived([...series.flatMap(s => [s.weight, s.avg]), goal].filter((v): v is number => typeof v === 'number'));
  const lo = $derived(ys.length ? Math.min(...ys) - 0.4 : 0);
  const hi = $derived(ys.length ? Math.max(...ys) + 0.4 : 1);
  const x = (i: number) => (i / Math.max(1, series.length - 1)) * W;
  const y = (v: number) => H - ((v - lo) / (hi - lo || 1)) * H;
  const line = $derived(series.map((s, i) => (s.avg === null ? null : `${x(i).toFixed(2)},${y(s.avg).toFixed(2)}`)).filter(Boolean).join(' '));
  let hover = $state<number | null>(null);
  function move(e: PointerEvent) {
    const r = (e.currentTarget as SVGElement).getBoundingClientRect();
    hover = Math.max(0, Math.min(series.length - 1, Math.round(((e.clientX - r.left) / r.width) * (series.length - 1))));
  }
  const week = $derived(lastDays(fit.days, fit.today, 7).reverse());
</script>

{#snippet stats()}
  <div class="stats">
    <div class="st main">
      <span class="label">Peso · média 7 dias</span>
      <span class="v mono">{kg(tr.cur)}<small> kg</small></span>
      {#if tr.delta !== null}<span class="d" class:up={tr.delta > 0} class:down={tr.delta < 0}>{tr.delta > 0 ? '▲' : tr.delta < 0 ? '▼' : '='} {kg(Math.abs(tr.delta))} kg na semana</span>
      {:else}<span class="d faint">registre o peso para ver a tendência</span>{/if}
    </div>
    <div class="st">
      <span class="label">Calorias na meta</span>
      <span class="v mono">{ad.kcalHit}<small>/7 dias</small></span>
    </div>
    <div class="st">
      <span class="label">Proteína na meta</span>
      <span class="v mono">{ad.protHit}<small>/7 dias</small></span>
    </div>
  </div>
{/snippet}

{#if !fit.ready}
  <div class="empty">Carregando…</div>
{:else}
  <div class="wk" class:x={expanded}>
    {@render stats()}
    <div class="chart" role="img" aria-label="Peso dos últimos {span} dias">
      {#if !ys.length}
        <div class="empty">Registre o peso da manhã na aba Hoje para ver o gráfico.</div>
      {:else}
        <svg viewBox="0 0 {W} {H}" preserveAspectRatio="none" role="presentation" onpointermove={move} onpointerleave={() => (hover = null)}>
          {#if goal}<line x1="0" x2={W} y1={y(goal)} y2={y(goal)} class="goal" vector-effect="non-scaling-stroke" />{/if}
          {#if line}<polyline points={line} class="avg" vector-effect="non-scaling-stroke" />{/if}
          {#if hover !== null}<line x1={x(hover)} x2={x(hover)} y1="0" y2={H} class="cross" vector-effect="non-scaling-stroke" />{/if}
        </svg>
        <!-- pontos em HTML para não distorcerem com o preserveAspectRatio -->
        {#each series as s, i}
          {#if s.weight !== null}<i class="dot" style="left:{x(i)}%;top:{(y(s.weight) / H) * 100}%"></i>{/if}
        {/each}
        {#if goal}<span class="gl" style="top:{(y(goal) / H) * 100}%">meta {kg(goal)}</span>{/if}
        {#if hover !== null}
          {@const s = series[hover]}
          <span class="tip mono" style="left:{x(hover)}%">{fmtShort.format(parseDay(s.date))} · {kg(s.weight)} kg · média {kg(s.avg)}</span>
        {/if}
      {/if}
    </div>
    <div class="legend faint"><span><i class="ld"></i> pesagem</span><span><i class="ll"></i> média 7 dias</span>{#if goal}<span><i class="lg"></i> meta</span>{/if}</div>

    {#if expanded}
      <table>
        <thead><tr><th>Dia</th><th>Peso</th><th>kcal</th><th>Proteína</th><th>Lançamentos</th></tr></thead>
        <tbody>
          {#each week as w}
            {@const s = dayTotals(w.log)}
            <tr>
              <td>{fmtShort.format(parseDay(w.date))}</td>
              <td class="mono">{kg(w.log?.weight ?? null)}</td>
              <td class="mono" class:ok={s.meals && Math.abs(s.kcal - tg.kcal) <= tg.kcal * 0.1}>{s.meals ? s.kcal.toLocaleString('pt-BR') : '—'}</td>
              <td class="mono" class:ok={s.meals && s.p >= tg.protein * 0.95}>{s.meals ? `${Math.round(s.p)} g` : '—'}</td>
              <td class="mono">{s.meals || '—'}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
{/if}

<style>
  .wk { height: 100%; display: flex; flex-direction: column; gap: 10px; min-height: 0; }
  .stats { display: grid; grid-template-columns: 1.4fr 1fr 1fr; gap: 12px; align-items: end; }
  .st { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .v { font-size: 26px; font-weight: 300; line-height: 1.1; }
  .main .v { font-size: 44px; }
  .x .v { font-size: 34px; }
  .x .main .v { font-size: 60px; }
  .v small { font-size: 12px; color: var(--muted); }
  .d { font-size: 11px; color: var(--muted); }
  .d.up { color: var(--amber); }
  .d.down { color: var(--teal); }
  .chart { position: relative; flex: 1; min-height: 70px; }
  .x .chart { flex: 0 0 38%; }
  svg { width: 100%; height: 100%; display: block; overflow: visible; }
  .avg { fill: none; stroke: var(--teal); stroke-width: 2; stroke-linejoin: round; }
  .goal { stroke: var(--leaf); stroke-width: 1.5; stroke-dasharray: 5 4; opacity: 0.8; }
  .cross { stroke: var(--line-strong); stroke-width: 1; }
  .dot { position: absolute; width: 7px; height: 7px; margin: -3.5px 0 0 -3.5px; border-radius: 50%; background: var(--text); opacity: 0.75; box-shadow: 0 0 0 2px rgba(7, 22, 26, 0.6); pointer-events: none; }
  .gl { position: absolute; right: 0; transform: translateY(-120%); font-size: 10.5px; color: var(--leaf); pointer-events: none; }
  .tip { position: absolute; top: -4px; transform: translate(-50%, -100%); white-space: nowrap; font-size: 11px; padding: 3px 8px; border-radius: 8px; background: var(--glass-strong); border: 1px solid var(--line); pointer-events: none; }
  .legend { display: flex; gap: 14px; font-size: 11px; }
  .legend i { display: inline-block; vertical-align: middle; margin-right: 4px; }
  .ld { width: 7px; height: 7px; border-radius: 50%; background: var(--text); opacity: 0.75; }
  .ll { width: 14px; height: 2px; background: var(--teal); }
  .lg { width: 14px; border-top: 2px dashed var(--leaf); }
  table { width: 100%; border-collapse: collapse; font-size: 13px; }
  th { text-align: left; font: 600 10.5px/1 var(--sans); letter-spacing: 0.12em; text-transform: uppercase; color: var(--faint); padding: 6px 8px; }
  td { padding: 7px 8px; border-top: 1px solid var(--line); }
  td.ok { color: var(--leaf); }
</style>
