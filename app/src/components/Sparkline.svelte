<script lang="ts">
  // Uma série, linha 2px + área suave. Hover mostra o valor (texto em tinta de texto, não na cor da série).
  let { values, max = 100, color = 'var(--teal)', format = (v: number) => `${Math.round(v)}%`, hover = false }:
    { values: number[]; max?: number; color?: string; format?: (v: number) => string; hover?: boolean } = $props();

  const N = 90;
  const top = $derived(Math.max(max, ...values, 1e-9));
  const pts = $derived(values.map((v, i) => [((N - values.length + i) / (N - 1)) * 100, 30 - (v / top) * 28] as const));
  const line = $derived(pts.map(([x, y]) => `${x.toFixed(2)},${y.toFixed(2)}`).join(' '));
  const area = $derived(pts.length ? `${pts[0][0]},30 ${line} ${pts[pts.length - 1][0]},30` : '');
  let hi = $state<number | null>(null);
  const gid = `g${Math.random().toString(36).slice(2, 8)}`;

  function move(e: PointerEvent) {
    const r = (e.currentTarget as SVGElement).getBoundingClientRect();
    const x = ((e.clientX - r.left) / r.width) * 100;
    let best = -1, bd = Infinity;
    pts.forEach(([px], i) => { const d = Math.abs(px - x); if (d < bd) { bd = d; best = i; } });
    hi = best >= 0 ? best : null;
  }
</script>

<div class="spark">
  <svg viewBox="0 0 100 30" preserveAspectRatio="none" role="img" aria-label={values.length ? format(values[values.length - 1]) : ''}
    onpointermove={hover ? move : undefined} onpointerleave={() => (hi = null)}>
    <defs>
      <linearGradient id={gid} x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color={color} stop-opacity="0.28" />
        <stop offset="1" stop-color={color} stop-opacity="0" />
      </linearGradient>
    </defs>
    {#if pts.length > 1}
      <polygon points={area} fill="url(#{gid})" />
      <polyline points={line} fill="none" stroke={color} stroke-width="2" vector-effect="non-scaling-stroke" stroke-linejoin="round" />
    {/if}
    {#if hi !== null}
      <line x1={pts[hi][0]} x2={pts[hi][0]} y1="0" y2="30" stroke="var(--line-strong)" vector-effect="non-scaling-stroke" />
    {/if}
  </svg>
  {#if hi !== null}
    <span class="tip mono" style="left:{pts[hi][0]}%">{format(values[hi])} · há {(values.length - 1 - hi) * 2}s</span>
  {/if}
</div>

<style>
  .spark { position: relative; width: 100%; height: 100%; min-height: 24px; }
  svg { width: 100%; height: 100%; display: block; overflow: visible; }
  .tip {
    position: absolute; top: -6px; transform: translate(-50%, -100%); pointer-events: none;
    background: var(--glass-strong); border: 1px solid var(--line); border-radius: 8px;
    padding: 3px 8px; font-size: 11px; white-space: nowrap; color: var(--text);
  }
</style>
