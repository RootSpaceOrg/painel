<script lang="ts">
  // Academia · Plano: perfil, metas calculadas, descoberta da manutenção e revisão da fase.
  import { fit, loadFit, saveProfile } from '../lib/fit.svelte';
  import { targets, maintenanceCheck, phaseReview, weightTrend, GOAL_LABEL, type Goal } from '../lib/fit.ts';
  import { parseDay } from '../lib/util';

  let { expanded = false }: { expanded?: boolean } = $props();
  $effect(() => { loadFit(); });

  const p = $derived(fit.profile);
  const tg = $derived(targets(p));
  const mc = $derived(maintenanceCheck(fit.days, fit.today, p));
  const rv = $derived(phaseReview(fit.days, fit.today, p));
  const tr = $derived(weightTrend(fit.days, fit.today));
  const phaseDays = $derived(Math.max(0, Math.round((parseDay(fit.today).getTime() - parseDay(p.phaseStart).getTime()) / 864e5)));
  const n = (v: number) => Math.round(v).toLocaleString('pt-BR');
  const kg = (v: number) => v.toLocaleString('pt-BR', { maximumFractionDigits: 1 });

  const save = () => saveProfile();
  /** Recomeça a contagem da fase (revisão a cada 14 dias). */
  const newPhase = () => { p.phaseStart = fit.today; };
  function setGoal(g: Goal) {
    if (g === p.goal) return;
    p.goal = g;
    p.review = 0; // revisões são da fase anterior
    newPhase();
    save();
  }
  function confirmMaintenance(kcal: number) { p.maintenance = kcal; p.estimate = null; p.review = 0; newPhase(); save(); }
  function useEstimate(kcal: number) { p.estimate = kcal; newPhase(); save(); } // segue descobrindo com a nova estimativa
  function applyReview(pct: number) { p.review += pct; newPhase(); save(); }
  function redoDiscovery() {
    if (!confirm('Voltar a descobrir a manutenção? A meta volta para 33 × peso por 1–2 semanas.')) return;
    p.maintenance = null; p.estimate = null; p.review = 0; newPhase(); save();
  }
  function setNum(field: 'weight' | 'targetWeight', v: string) {
    const x = parseFloat(v.replace(',', '.'));
    if (field === 'targetWeight') p.targetWeight = Number.isFinite(x) && x > 20 ? Math.round(x * 10) / 10 : null;
    else if (Number.isFinite(x) && x > 20 && x < 400) p.weight = Math.round(x * 10) / 10;
    save();
  }
</script>

{#snippet status()}
  {#if tg.estimated}
    <div class="card">
      <b>Descobrindo a manutenção</b>
      <p>Coma cerca de <b class="mono">{n(tg.maintenance)} kcal</b> por dia e pese-se toda manhã por 1–2 semanas{p.estimate ? '' : ` (33 × ${kg(p.weight)} kg)`}.</p>
      {#if !mc.ready}
        <p class="faint">Faltam pesagens: {mc.trend.nCur}/4 nesta semana e {mc.trend.nPrev}/4 na anterior.</p>
      {:else if mc.stable}
        <p>Peso estável ({mc.trend.delta! >= 0 ? '+' : ''}{kg(mc.trend.delta!)} kg). Sua manutenção é <b class="mono">{n(mc.measured)} kcal</b>.</p>
        <button class="primary" onclick={() => confirmMaintenance(mc.measured)}>Confirmar {n(mc.measured)} kcal</button>
      {:else}
        <p>Peso {mc.trend.delta! > 0 ? 'subiu' : 'caiu'} {kg(Math.abs(mc.trend.delta!))} kg na semana: ajuste para <b class="mono">{n(mc.suggested)} kcal</b> e observe mais uma semana.</p>
        <button class="primary" onclick={() => useEstimate(mc.suggested)}>Usar {n(mc.suggested)} kcal</button>
      {/if}
    </div>
  {:else if rv.due}
    <div class="card">
      <b>Revisão da fase · {rv.since} dias</b>
      {#if rv.pct}
        <p>O peso não andou para o lado do objetivo ({rv.trend.delta! >= 0 ? '+' : ''}{kg(rv.trend.delta!)} kg/semana). Sugestão: <b>{rv.pct > 0 ? '+' : ''}{rv.pct}%</b> nas calorias. A proteína não muda.</p>
        <div class="row">
          <button class="primary" onclick={() => applyReview(rv.pct)}>Aplicar {rv.pct > 0 ? '+' : ''}{rv.pct}%</button>
          <button class="ghost" onclick={() => { newPhase(); save(); }}>Agora não</button>
        </div>
      {:else}
        <p>No caminho certo ({rv.trend.delta! >= 0 ? '+' : ''}{kg(rv.trend.delta!)} kg/semana). Mantém as calorias.</p>
        <button class="ghost" onclick={() => { newPhase(); save(); }}>Ok, próxima revisão em 14 dias</button>
      {/if}
    </div>
  {:else}
    <p class="faint small">Manutenção medida: {n(tg.maintenance)} kcal · próxima revisão em {Math.max(0, 14 - phaseDays)} {14 - phaseDays === 1 ? 'dia' : 'dias'}.</p>
  {/if}
{/snippet}

{#if !fit.ready}
  <div class="empty">Carregando…</div>
{:else}
  <div class="plan" class:x={expanded}>
    <div class="col">
      <div class="row between">
        <span class="chip">{GOAL_LABEL[p.goal]}{!tg.estimated && p.goal !== 'maintain' ? ` · ${p.goal === 'cut' ? '−' : '+'}${p.adjust}%` : ''}{p.review ? ` · revisão ${p.review > 0 ? '+' : ''}${p.review}%` : ''}</span>
        <span class="faint small">fase há {phaseDays} {phaseDays === 1 ? 'dia' : 'dias'}</span>
      </div>
      <div class="tg">
        <div><span class="label">Calorias</span><span class="mono v">{n(tg.kcal)}</span></div>
        <div><span class="label">Proteína</span><span class="mono v">{tg.protein}<small>g</small></span></div>
        <div><span class="label">Gordura</span><span class="mono v">{tg.fat}<small>g</small></span></div>
        <div><span class="label">Carbo</span><span class="mono v">{tg.carbs}<small>g</small></span></div>
      </div>
      {@render status()}
    </div>

    {#if expanded}
      <div class="col form glass">
        <span class="label">Perfil</span>
        <label class="f"><span>Peso atual (kg)</span>
          <span class="row">
            {#key p.weight}<input class="mono num" inputmode="decimal" value={p.weight} onchange={e => setNum('weight', e.currentTarget.value)} />{/key}
            {#if tr.cur !== null && tr.cur !== p.weight}<button class="ghost" onclick={() => { p.weight = tr.cur!; save(); }}>Usar média da semana ({kg(tr.cur)} kg)</button>{/if}
          </span>
        </label>
        <div class="f"><span>Objetivo</span>
          <div class="row">
            {#each ['cut', 'maintain', 'bulk'] as const as g}
              <button class:on={p.goal === g} aria-pressed={p.goal === g} onclick={() => setGoal(g)}>{GOAL_LABEL[g]}</button>
            {/each}
          </div>
        </div>
        {#if p.goal !== 'maintain'}
          <label class="f"><span>Ajuste do objetivo · {p.goal === 'cut' ? '−' : '+'}{p.adjust}%</span>
            <input type="range" min="5" max="10" step="1" bind:value={p.adjust} style="--p:{((p.adjust - 5) / 5) * 100}" onchange={save} />
          </label>
        {/if}
        <label class="f"><span>Proteína · {p.proteinGkg.toLocaleString('pt-BR')} g/kg = {tg.protein} g (igual em todas as fases)</span>
          <input type="range" min="1.6" max="2.2" step="0.1" bind:value={p.proteinGkg} style="--p:{((p.proteinGkg - 1.6) / 0.6) * 100}" onchange={save} />
        </label>
        <label class="f"><span>Gordura · {p.fatGkg.toLocaleString('pt-BR')} g/kg = {tg.fat} g (castanhas, azeite, abacate)</span>
          <input type="range" min="0.8" max="1.2" step="0.1" bind:value={p.fatGkg} style="--p:{((p.fatGkg - 0.8) / 0.4) * 100}" onchange={save} />
        </label>
        <label class="f"><span>Peso meta (opcional, linha no gráfico)</span>
          {#key p.targetWeight}<input class="mono num" inputmode="decimal" placeholder="—" value={p.targetWeight ?? ''} onchange={e => setNum('targetWeight', e.currentTarget.value)} />{/key}
        </label>
        {#if !tg.estimated}<button class="ghost danger" onclick={redoDiscovery}>Refazer a descoberta da manutenção</button>{/if}
        <p class="faint small">Carboidrato = o que sobra: (calorias − proteína × 4 − gordura × 9) ÷ 4. 3 a 4 refeições por dia bastam; o que conta é a constância.</p>
      </div>
    {/if}
  </div>
{/if}

<style>
  .plan { height: 100%; display: flex; flex-direction: column; gap: 10px; min-height: 0; overflow-y: auto; }
  .plan.x { display: grid; grid-template-columns: 1fr 1fr; gap: 28px; }
  .between { justify-content: space-between; }
  .small { font-size: 11.5px; }
  .chip { font-size: 12px; padding: 3px 10px; border-radius: 999px; background: rgba(127, 224, 208, 0.12); color: var(--teal); }
  .tg { display: grid; grid-template-columns: 1fr 1fr; gap: 8px 14px; }
  .x .tg { grid-template-columns: repeat(4, 1fr); }
  .tg > div { display: flex; flex-direction: column; gap: 2px; }
  .v { font-size: 20px; font-weight: 300; }
  .x .v { font-size: 28px; }
  .v small { font-size: 12px; color: var(--muted); margin-left: 1px; }
  .card { display: flex; flex-direction: column; gap: 6px; align-items: flex-start; padding: 10px 12px; border-radius: 12px; background: rgba(190, 240, 225, 0.06); border: 1px solid var(--line); font-size: 12.5px; }
  .card p { margin: 0; line-height: 1.45; }
  .card button { font-size: 12px; padding: 4px 12px; }
  .form { padding: 16px; gap: 12px; align-self: start; }
  .f { display: flex; flex-direction: column; gap: 6px; font-size: 12.5px; color: var(--muted); }
  .num { width: 90px; }
  .on { border-color: rgba(127, 224, 208, 0.5); color: var(--teal); background: rgba(127, 224, 208, 0.1); }
</style>
