<script lang="ts">
  import { prefs } from '../lib/state.svelte';
  import { fmtWeekday, fmtDayMonth } from '../lib/util';

  let { expanded = false }: { expanded?: boolean } = $props();
  let now = $state(new Date());
  $effect(() => {
    const t = setInterval(() => (now = new Date()), 1000);
    return () => clearInterval(t);
  });

  const hh = $derived(String(now.getHours()).padStart(2, '0'));
  const mm = $derived(String(now.getMinutes()).padStart(2, '0'));
  const ss = $derived(String(now.getSeconds()).padStart(2, '0'));
  const greet = $derived(now.getHours() < 5 ? 'Boa madrugada' : now.getHours() < 12 ? 'Bom dia' : now.getHours() < 18 ? 'Boa tarde' : 'Boa noite');
  // progresso do dia, discreto
  const dayPct = $derived(((now.getHours() * 60 + now.getMinutes()) / 1440) * 100);
</script>

<div class="clock" class:big={expanded}>
  <div class="label">{greet}</div>
  <div class="time mono">
    {hh}<span class="colon">:</span>{mm}{#if prefs.seconds || expanded}<span class="sec">{ss}</span>{/if}
  </div>
  <div class="date">
    <span class="wd">{fmtWeekday.format(now)}</span>
    <span class="muted">{fmtDayMonth.format(now)}</span>
  </div>
  <div class="day" title="Quanto do dia já passou"><i style="width:{dayPct}%"></i></div>
</div>

<style>
  .clock { height: 100%; display: flex; flex-direction: column; justify-content: center; gap: 6px; padding: 4px 6px; container-type: size; }
  .time {
    font-size: clamp(40px, 34cqh, 120px); font-weight: 300; line-height: 1; letter-spacing: -0.02em;
    text-shadow: 0 0 40px rgba(127, 224, 208, 0.25);
  }
  .colon { color: var(--teal); opacity: 0.8; }
  .sec { font-size: 0.36em; color: var(--muted); margin-left: 6px; }
  .date { display: flex; gap: 8px; align-items: baseline; font-size: 15px; }
  .wd { text-transform: capitalize; color: var(--leaf); }
  .day { height: 2px; background: rgba(190, 240, 225, 0.1); border-radius: 1px; overflow: hidden; margin-top: 4px; }
  .day i { display: block; height: 100%; background: linear-gradient(90deg, var(--teal), var(--leaf)); }
  .big { align-items: center; text-align: center; gap: 18px; }
  .big .time { font-size: 22vh; }
  .big .date { font-size: 26px; }
  .big .day { width: 50%; }
  .big .label { font-size: 13px; }
</style>
