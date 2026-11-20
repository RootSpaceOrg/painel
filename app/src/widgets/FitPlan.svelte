<script lang="ts">
  // Academia · Metas: objetivo com um toque e as metas do dia em números grandes. Ajustes finos no expandido.
  import { fit, loadFit, saveProfile, fitTargets } from '../lib/fit.svelte';
  import { GOAL_LABEL, type Goal } from '../lib/fit.ts';

  let { expanded = false }: { expanded?: boolean } = $props();
  $effect(() => { loadFit(); });

  const p = $derived(fit.profile);
  const tg = $derived(fitTargets());
  const fromLogs = $derived(Object.values(fit.days).some(d => typeof d?.weight === 'number')); // já tem pesagens
  const n = (v: number) => Math.round(v).toLocaleString('pt-BR');
  const kg = (v: number) => v.toLocaleString('pt-BR', { maximumFractionDigits: 1 });
  const save = () => saveProfile();
  const setGoal = (g: Goal) => { p.goal = g; save(); };
  function setNum(field: 'weight' | 'targetWeight' | 'maintenance', v: string) {
    const x = parseFloat(v.replace(',', '.'));
    const ok = Number.isFinite(x) && x > 0;
    if (field === 'weight') { if (ok && x > 20 && x < 400) p.weight = Math.round(x * 10) / 10; }
    else if (field === 'targetWeight') p.targetWeight = ok && x > 20 ? Math.round(x * 10) / 10 : null;
    else p.maintenance = ok && x > 800 ? Math.round(x) : null;
    save();
  }
</script>

{#if !fit.ready}
  <div class="empty">Carregando…</div>
{:else}
  <div class="plan" class:x={expanded}>
    <div class="goals" role="group" aria-label="Objetivo">
      {#each ['cut', 'maintain', 'bulk'] as const as g}
        <button class:on={p.goal === g} aria-pressed={p.goal === g} onclick={() => setGoal(g)}>{GOAL_LABEL[g]}</button>
      {/each}
    </div>
    <div class="nums">
      <div><span class="label">Calorias / dia</span><span class="v mono">{n(tg.kcal)}</span></div>
      <div><span class="label">Proteína / dia</span><span class="v mono">{tg.protein}<small> g</small></span></div>
    </div>
    <div class="sub faint">Gordura {tg.fat} g · Carbo {tg.carbs} g</div>
    <div class="sub faint">
      {fromLogs ? `Pelo seu peso médio: ${kg(tg.weight)} kg` : `Peso do perfil: ${kg(tg.weight)} kg`}
      · {p.maintenance ? `manutenção ${n(p.maintenance)}` : 'manutenção 33 kcal/kg'}{p.goal !== 'maintain' ? ` ${p.goal === 'cut' ? '−' : '+'}${p.adjust}%` : ''}
    </div>

    {#if expanded}
      <div class="form glass">
        {#if p.goal !== 'maintain'}
          <label class="f"><span>Quanto {p.goal === 'cut' ? 'cortar' : 'somar'} · {p.adjust}%</span>
            <input type="range" min="5" max="10" step="1" bind:value={p.adjust} style="--p:{((p.adjust - 5) / 5) * 100}" onchange={save} />
          </label>
        {/if}
        <label class="f"><span>Proteína · {p.proteinGkg.toLocaleString('pt-BR')} g por kg</span>
          <input type="range" min="1.6" max="2.2" step="0.1" bind:value={p.proteinGkg} style="--p:{((p.proteinGkg - 1.6) / 0.6) * 100}" onchange={save} />
        </label>
        <label class="f"><span>Gordura · {p.fatGkg.toLocaleString('pt-BR')} g por kg</span>
          <input type="range" min="0.8" max="1.2" step="0.1" bind:value={p.fatGkg} style="--p:{((p.fatGkg - 0.8) / 0.4) * 100}" onchange={save} />
        </label>
        <div class="trio">
          <label class="f"><span>Manutenção (kcal)</span>
            {#key p.maintenance}<input class="mono" inputmode="numeric" placeholder="33 × peso" value={p.maintenance ?? ''} onchange={e => setNum('maintenance', e.currentTarget.value)} />{/key}
          </label>
          <label class="f"><span>Peso meta (kg)</span>
            {#key p.targetWeight}<input class="mono" inputmode="decimal" placeholder="—" value={p.targetWeight ?? ''} onchange={e => setNum('targetWeight', e.currentTarget.value)} />{/key}
          </label>
          <label class="f"><span>Peso inicial (kg)</span>
            {#key p.weight}<input class="mono" inputmode="decimal" value={p.weight} onchange={e => setNum('weight', e.currentTarget.value)} />{/key}
          </label>
        </div>
        <p class="faint small">As metas usam a média das suas pesagens dos últimos 7 dias (o peso inicial só vale até você registrar).
          Se souber sua manutenção real, coloque acima; senão fica 33 kcal por kg.</p>
      </div>
    {/if}
  </div>
{/if}

<style>
  .plan { height: 100%; display: flex; flex-direction: column; gap: 10px; min-height: 0; overflow-y: auto; }
  .goals { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }
  .goals button { padding: 8px 4px; border-radius: 10px; font-size: 13px; }
  .goals .on { color: var(--ink); background: var(--teal); border-color: transparent; font-weight: 600; }
  .nums { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
  .nums > div { display: flex; flex-direction: column; gap: 2px; }
  .v { font-size: 30px; font-weight: 300; line-height: 1.05; }
  .x .v { font-size: 56px; }
  .v small { font-size: 16px; color: var(--muted); }
  .sub { font-size: 12px; }
  .form { padding: 16px; display: flex; flex-direction: column; gap: 14px; margin-top: 6px; }
  .f { display: flex; flex-direction: column; gap: 6px; font-size: 13px; color: var(--muted); }
  .trio { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; }
  .small { font-size: 12px; margin: 0; }
</style>
