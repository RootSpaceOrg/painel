<script lang="ts">
  import { live } from '../lib/state.svelte';
  import { fmtBytes } from '../lib/util';
  import Sparkline from '../components/Sparkline.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  const s = $derived(live.sys);
  const h = live.hist;
  const rate = (v: number) => `${fmtBytes(v)}/s`;
  const ramPct = $derived(s ? (s.ram_used / s.ram_total) * 100 : 0);
  // temperatura nunca só por cor: o número vem junto; tom quente só acima de 80°C
  const hot = $derived((s?.gpu?.temp ?? 0) >= 80);
</script>

{#if !s}
  <div class="empty">Lendo sensores…</div>
{:else}
  <div class="hw" class:x={expanded}>
    <div class="tile">
      <div class="top"><span class="label">CPU</span><span class="v mono">{Math.round(s.cpu)}%</span></div>
      <div class="sp"><Sparkline values={h.cpu} hover={expanded} /></div>
    </div>
    <div class="tile">
      <div class="top">
        <span class="label">GPU</span>
        <span class="v mono">{s.gpu ? `${Math.round(s.gpu.util)}%` : '—'}</span>
      </div>
      {#if s.gpu}<div class="sub mono" class:hot>{s.gpu.temp}°C{#if s.gpu.power_w !== null}{` · ${Math.round(s.gpu.power_w)} W`}{/if}</div>{/if}
      <div class="sp"><Sparkline values={h.gpu} color="var(--leaf)" hover={expanded} /></div>
    </div>
    <div class="tile">
      <div class="top"><span class="label">RAM</span><span class="v mono">{Math.round(ramPct)}%</span></div>
      <div class="sub mono">{fmtBytes(s.ram_used)} / {fmtBytes(s.ram_total, 0)}</div>
      <div class="sp"><Sparkline values={h.ram} color="var(--amber)" hover={expanded} /></div>
    </div>
    <div class="tile">
      <div class="top"><span class="label">Rede</span><span class="v mono small">↓ {rate(s.net_rx)}</span></div>
      <div class="sub mono">↑ {rate(s.net_tx)}</div>
      <div class="sp"><Sparkline values={h.rx} max={0} color="var(--blossom)" format={rate} hover={expanded} /></div>
    </div>

    {#if expanded}
      <div class="tile wide">
        <span class="label">Núcleos</span>
        <div class="cores">
          {#each s.cores as c, i}
            <div class="core" title="Núcleo {i}: {Math.round(c)}%"><i style="height:{c}%"></i></div>
          {/each}
        </div>
      </div>
      <div class="tile wide col">
        {#if s.gpu}
          <span class="label">{s.gpu.name}</span>
          <div class="meter"><span>VRAM</span><div class="m"><i style="width:{(s.gpu.vram_used / s.gpu.vram_total) * 100}%"></i></div><span class="mono">{fmtBytes(s.gpu.vram_used)} / {fmtBytes(s.gpu.vram_total, 0)}</span></div>
        {/if}
        <span class="label">Discos</span>
        {#each s.disks as d}
          <div class="meter"><span class="ellipsis">{d.mount} {d.name}</span><div class="m"><i style="width:{(d.used / d.total) * 100}%"></i></div><span class="mono">{fmtBytes(d.total - d.used, 0)} livres</span></div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  .hw { height: 100%; display: grid; grid-template-columns: repeat(auto-fit, minmax(130px, 1fr)); grid-auto-rows: 1fr; gap: 10px; }
  .hw.x { grid-template-columns: repeat(4, 1fr); grid-template-rows: 1.2fr 1fr; }
  .tile { display: flex; flex-direction: column; gap: 2px; min-height: 0; min-width: 0; }
  .x .tile { background: rgba(0, 0, 0, 0.14); border: 1px solid var(--line); border-radius: 12px; padding: 14px; }
  .top { display: flex; justify-content: space-between; align-items: baseline; }
  .v { font-size: 20px; font-weight: 300; }
  .x .v { font-size: 30px; }
  .small { font-size: 13px; }
  .sub { font-size: 11px; color: var(--muted); }
  .hot { color: var(--amber); }
  .sp { flex: 1; min-height: 20px; margin-top: 4px; }
  .wide { grid-column: span 2; }
  .cores { flex: 1; display: flex; gap: 3px; align-items: flex-end; margin-top: 8px; }
  .core { flex: 1; height: 100%; background: rgba(190, 240, 225, 0.07); border-radius: 3px; display: flex; align-items: flex-end; overflow: hidden; }
  .core i { width: 100%; background: var(--teal); border-radius: 3px 3px 0 0; transition: height 0.6s; }
  .meter { display: grid; grid-template-columns: 110px 1fr auto; gap: 10px; align-items: center; font-size: 12px; color: var(--muted); }
  .m { height: 6px; background: rgba(190, 240, 225, 0.1); border-radius: 3px; overflow: hidden; }
  .m i { display: block; height: 100%; background: var(--leaf); border-radius: 3px; }
</style>
