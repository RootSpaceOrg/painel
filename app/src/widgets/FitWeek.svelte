<script lang="ts">
  // Academia · Peso: último peso, variação (semana anterior e desde o início) e gráfico com média de 7 dias e meta.
  import { fit, fitUi, loadFit } from '../lib/fit.svelte';
  import { movingAvg, weightTrend, lastWeight, firstWeight } from '../lib/fit.ts';
  import { parseDay, fmtShort } from '../lib/util';

  let { expanded = false }: { expanded?: boolean } = $props();
  $effect(() => { loadFit(); });

  const current = $derived(lastWeight(fit.days, fit.today));
  const tr = $derived(weightTrend(fit.days, fit.today));
  const first = $derived(firstWeight(fit.days));
  const sinceStart = $derived(current !== null && first ? Math.round((current - first.weight) * 10) / 10 : null);
  const goal = $derived(fit.profile.targetWeight);
  const toGoal = $derived(current !== null && goal ? Math.round((current - goal) * 10) / 10 : null);
  const kg = (v: number | null) => (v === null ? '—' : v.toLocaleString('pt-BR', { minimumFractionDigits: 1, maximumFractionDigits: 1 }));
  const arrow = (v: number) => (v > 0 ? '▲' : v < 0 ? '▼' : '=');

  // gráfico: pontos = pesagens; linha = média de 7 dias; tracejado = peso meta
  const span = $derived(expanded ? 60 : 28);
  // começa na primeira pesagem da janela (sem área vazia à esquerda), mínimo de 7 dias
  const series = $derived.by(() => {
    const all = movingAvg(fit.days, fit.today, span);
    const first = all.findIndex(s => s.weight !== null);
    return first < 0 ? all : all.slice(Math.min(first, all.length - 7));
  });
  const W = 100, H = 40;
  const wys = $derived(series.flatMap(s => [s.weight, s.avg]).filter((v): v is number => typeof v === 'number'));
  // a meta só entra na escala se estiver perto; longe, vira só uma indicação (senão a linha do peso fica achatada)
  const goalNear = $derived(goal !== null && wys.length > 0 && goal >= Math.min(...wys) - 3 && goal <= Math.max(...wys) + 3);
  const ys = $derived(goalNear ? [...wys, goal!] : wys);
  const lo = $derived(ys.length ? Math.min(...ys) - 0.5 : 0);
  const hi = $derived(ys.length ? Math.max(...ys) + 0.5 : 1);
  const x = (i: number) => (i / Math.max(1, series.length - 1)) * W;
  const y = (v: number) => H - ((v - lo) / (hi - lo || 1)) * H;
  const line = $derived(series.map((s, i) => (s.avg === null ? null : `${x(i).toFixed(2)},${y(s.avg).toFixed(2)}`)).filter(Boolean).join(' '));
  let hover = $state<number | null>(null);
  function move(e: PointerEvent) {
    const r = (e.currentTarget as SVGElement).getBoundingClientRect();
    hover = Math.max(0, Math.min(series.length - 1, Math.round(((e.clientX - r.left) / r.width) * (series.length - 1))));
  }
</script>

{#if !fit.ready}
  <div class="empty">Carregando…</div>
{:else}
  <div class="wk" class:x={expanded}>
    <div class="top">
      <div class="now">
        <span class="v mono">{kg(current)}<small> kg</small></span>
        {#if current === null}<button class="primary" onclick={() => (fitUi.weight = true)}>Registrar o primeiro peso</button>{/if}
      </div>
      <div class="vars">
        <div class="var">
          <span class="label">vs. semana anterior</span>
          {#if tr.delta !== null}<span class="dv mono" class:up={tr.delta > 0} class:down={tr.delta < 0}>{arrow(tr.delta)} {kg(Math.abs(tr.delta))} kg</span>
          {:else}<span class="faint small">precisa de pesagens nas 2 semanas</span>{/if}
        </div>
        <div class="var">
          <span class="label">desde o início</span>
          {#if sinceStart !== null && first}<span class="dv mono" class:up={sinceStart > 0} class:down={sinceStart < 0}>{arrow(sinceStart)} {kg(Math.abs(sinceStart))} kg</span>
            <span class="faint small">desde {fmtShort.format(parseDay(first.date))}</span>
          {:else}<span class="faint small">—</span>{/if}
        </div>
        {#if toGoal !== null}
          <div class="var">
            <span class="label">até a meta</span>
            <span class="dv mono">{toGoal > 0 ? `faltam ${kg(toGoal)}` : 'meta atingida ✓'}{toGoal > 0 ? ' kg' : ''}</span>
          </div>
        {/if}
      </div>
    </div>

    <div class="chart" role="img" aria-label="Peso dos últimos {span} dias">
      {#if ys.length}
        <svg viewBox="0 0 {W} {H}" preserveAspectRatio="none" role="presentation" onpointermove={move} onpointerleave={() => (hover = null)}>
          {#if goal && goalNear}<line x1="0" x2={W} y1={y(goal)} y2={y(goal)} class="goal" vector-effect="non-scaling-stroke" />{/if}
          {#if line}<polyline points={line} class="avg" vector-effect="non-scaling-stroke" />{/if}
          {#if hover !== null}<line x1={x(hover)} x2={x(hover)} y1="0" y2={H} class="cross" vector-effect="non-scaling-stroke" />{/if}
        </svg>
        {#each series as s, i}
          {#if s.weight !== null}<i class="dot" style="left:{x(i)}%;top:{(y(s.weight) / H) * 100}%"></i>{/if}
        {/each}
        {#if goal && goalNear}<span class="gl" style="top:{(y(goal) / H) * 100}%">meta {kg(goal)}</span>
        {:else if goal}<span class="gl far">meta {kg(goal)} {goal < lo ? '↓' : '↑'}</span>{/if}
        {#if hover !== null}
          {@const s = series[hover]}
          <span class="tip mono" style="left:{x(hover)}%">{fmtShort.format(parseDay(s.date))} · {kg(s.weight)} kg · média {kg(s.avg)}</span>
        {/if}
      {:else}
        <div class="empty">O gráfico aparece a partir das primeiras pesagens.</div>
      {/if}
    </div>
    <div class="legend faint"><span><i class="ld"></i> pesagem</span><span><i class="ll"></i> média 7 dias</span>{#if goal && goalNear}<span><i class="lg"></i> meta</span>{/if}</div>
  </div>
{/if}

<style>
  .wk { height: 100%; display: flex; flex-direction: column; gap: 10px; min-height: 0; }
  .top { display: flex; align-items: flex-end; gap: 24px; flex-wrap: wrap; }
  .now { display: flex; flex-direction: column; gap: 6px; }
  .v { font-size: 52px; font-weight: 300; line-height: 1; letter-spacing: -0.02em; }
  .x .v { font-size: 76px; }
  .v small { font-size: 18px; color: var(--muted); }
  .vars { display: flex; gap: 22px; flex-wrap: wrap; padding-bottom: 4px; }
  .var { display: flex; flex-direction: column; gap: 2px; }
  .dv { font-size: 19px; }
  .x .dv { font-size: 24px; }
  .dv.up { color: var(--amber); }
  .dv.down { color: var(--teal); }
  .small { font-size: 11px; }
  .chart { position: relative; flex: 1; min-height: 60px; }
  svg { width: 100%; height: 100%; display: block; overflow: visible; }
  .avg { fill: none; stroke: var(--teal); stroke-width: 2; stroke-linejoin: round; }
  .goal { stroke: var(--leaf); stroke-width: 1.5; stroke-dasharray: 5 4; opacity: 0.8; }
  .cross { stroke: var(--line-strong); stroke-width: 1; }
  .dot { position: absolute; width: 7px; height: 7px; margin: -3.5px 0 0 -3.5px; border-radius: 50%; background: var(--text); opacity: 0.75; box-shadow: 0 0 0 2px rgba(7, 22, 26, 0.6); pointer-events: none; }
  .gl.far { top: auto; bottom: 0; transform: none; color: var(--muted); }
  .gl { position: absolute; right: 0; transform: translateY(-120%); font-size: 10.5px; color: var(--leaf); pointer-events: none; }
  .tip { position: absolute; top: -4px; transform: translate(-50%, -100%); white-space: nowrap; font-size: 11px; padding: 3px 8px; border-radius: 8px; background: var(--glass-strong); border: 1px solid var(--line); pointer-events: none; }
  .legend { display: flex; gap: 14px; font-size: 11px; }
  .legend i { display: inline-block; vertical-align: middle; margin-right: 4px; }
  .ld { width: 7px; height: 7px; border-radius: 50%; background: var(--text); opacity: 0.75; }
  .ll { width: 14px; height: 2px; background: var(--teal); }
  .lg { width: 14px; border-top: 2px dashed var(--leaf); }
</style>
