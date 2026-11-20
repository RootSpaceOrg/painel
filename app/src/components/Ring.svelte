<script lang="ts">
  // Anel de progresso (consumido / meta). Passou da meta: anel cheio em âmbar. Conteúdo vai no meio (children).
  import type { Snippet } from 'svelte';
  let { value, goal, color = 'var(--teal)', size = 120, stroke = 10, children }:
    { value: number; goal: number; color?: string; size?: number; stroke?: number; children?: Snippet } = $props();

  const r = $derived((size - stroke) / 2);
  const C = $derived(2 * Math.PI * r);
  const frac = $derived(goal > 0 ? Math.min(1, value / goal) : 0);
  const over = $derived(goal > 0 && value > goal);
</script>

<div class="ring" style="width:{size}px;height:{size}px">
  <svg viewBox="0 0 {size} {size}" aria-hidden="true">
    <circle cx={size / 2} cy={size / 2} {r} class="trk" stroke-width={stroke} />
    <circle cx={size / 2} cy={size / 2} {r} class="val" stroke-width={stroke} stroke={over ? 'var(--amber)' : color}
      stroke-dasharray={C} stroke-dashoffset={C * (1 - frac)} />
  </svg>
  <div class="mid">{@render children?.()}</div>
</div>

<style>
  .ring { position: relative; flex: none; }
  svg { width: 100%; height: 100%; rotate: -90deg; }
  circle { fill: none; }
  .trk { stroke: rgba(190, 240, 225, 0.1); }
  .val { stroke-linecap: round; transition: stroke-dashoffset 0.6s cubic-bezier(0.3, 0.7, 0.2, 1); }
  .mid { position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center; text-align: center; }
  @media (prefers-reduced-motion: reduce) { .val { transition: none; } }
</style>
