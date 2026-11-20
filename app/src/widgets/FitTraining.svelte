<script lang="ts">
  // Academia · Treino: séries por grupo na semana (meta 5–12) e frequência (meta 2–3), segunda a domingo.
  import { fit, loadFit } from '../lib/fit.svelte';
  import { weekVolume, SETS_MIN, SETS_MAX, FREQ_MIN, FREQ_MAX } from '../lib/fit.ts';

  let { expanded = false }: { expanded?: boolean } = $props();
  $effect(() => { loadFit(); });

  const vol = $derived(weekVolume(fit.days, fit.today));
  const SCALE = 16; // séries no fim da barra
  // grupos abaixo da meta primeiro; dentro disso, os com menos séries
  const rows = $derived(
    Object.entries(vol)
      .map(([g, v]) => ({ g, ...v, low: v.sets < SETS_MIN || v.freq < FREQ_MIN, high: v.sets > SETS_MAX }))
      .sort((a, b) => +b.low - +a.low || a.sets - b.sets),
  );
  const lows = $derived(rows.filter(r => r.low).length);
  // dias restantes na semana (para não alarmar na segunda-feira)
  const daysLeft = $derived(6 - ((new Date(fit.today + 'T12:00').getDay() + 6) % 7));
</script>

{#if !fit.ready}
  <div class="empty">Carregando…</div>
{:else}
  <div class="tr" class:x={expanded}>
    <div class="head">
      <span class="faint small">
        {lows ? `${lows} ${lows === 1 ? 'grupo abaixo' : 'grupos abaixo'} da meta` : 'Todos os grupos na meta 💪'}
        · {daysLeft ? `${daysLeft} ${daysLeft === 1 ? 'dia' : 'dias'} até domingo` : 'último dia da semana'}
      </span>
      <span class="faint small">meta {SETS_MIN}–{SETS_MAX} séries · {FREQ_MIN}–{FREQ_MAX}×</span>
    </div>
    <ul class="scroll">
      {#each rows as r (r.g)}
        <li class:low={r.low}>
          <span class="g ellipsis">{r.g}</span>
          <div class="bar" title="{r.sets} séries na semana">
            <span class="band" style="left:{(SETS_MIN / SCALE) * 100}%;width:{((SETS_MAX - SETS_MIN) / SCALE) * 100}%"></span>
            <i style="width:{Math.min(100, (r.sets / SCALE) * 100)}%" class:hi={r.high}></i>
          </div>
          <span class="mono n">{r.sets}</span>
          <span class="freq" title="{r.freq}× na semana">
            {#each Array(FREQ_MAX) as _, i}<i class:on={i < r.freq}></i>{/each}
          </span>
          {#if r.low}<span class="flag">abaixo</span>{:else if r.high}<span class="flag hi">acima</span>{:else}<span class="flag okf">ok</span>{/if}
        </li>
      {/each}
    </ul>
    {#if expanded}
      <p class="faint small">Registre as séries de cada grupo na aba <b>Hoje</b> (expandida). A semana vai de segunda a domingo.
        Abaixo da meta = menos de {SETS_MIN} séries ou treinado menos de {FREQ_MIN}× na semana.</p>
    {/if}
  </div>
{/if}

<style>
  .tr { height: 100%; display: flex; flex-direction: column; gap: 8px; min-height: 0; }
  .head { display: flex; justify-content: space-between; gap: 8px; flex-wrap: wrap; }
  .small { font-size: 11.5px; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 5px; flex: 1; min-height: 0; }
  li { display: grid; grid-template-columns: 82px minmax(0, 1fr) 22px auto 46px; align-items: center; gap: 8px; font-size: 12.5px; }
  .x li { grid-template-columns: 120px minmax(0, 1fr) 28px auto 56px; font-size: 14px; padding: 4px 0; }
  .g { color: var(--muted); }
  .low .g { color: var(--text); }
  .bar { position: relative; height: 8px; border-radius: 4px; background: rgba(190, 240, 225, 0.08); overflow: hidden; }
  .band { position: absolute; top: 0; bottom: 0; background: rgba(207, 224, 142, 0.14); }
  .bar i { position: absolute; left: 0; top: 0; bottom: 0; border-radius: 4px; background: var(--teal); }
  .low .bar i { background: var(--amber); }
  .bar i.hi { background: var(--blossom); }
  .n { text-align: right; color: var(--muted); }
  .freq { display: inline-flex; gap: 3px; }
  .freq i { width: 7px; height: 7px; border-radius: 50%; background: rgba(190, 240, 225, 0.14); }
  .freq i.on { background: var(--leaf); }
  .flag { font-size: 10.5px; text-align: center; padding: 1px 6px; border-radius: 999px; color: var(--amber); background: rgba(232, 194, 124, 0.14); }
  .flag.hi { color: var(--blossom); background: rgba(243, 169, 196, 0.12); }
  .flag.okf { color: var(--leaf); background: rgba(207, 224, 142, 0.1); }
</style>
