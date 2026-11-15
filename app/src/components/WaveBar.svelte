<script lang="ts">
  // Barra de progresso estilo player One UI/Android: a parte tocada vira uma onda que corre
  // enquanto toca e achata ao pausar; o resto é uma linha fina. Clique/arraste para buscar.
  // Só transform/opacity animam (compositor), então o custo é ~zero.
  let { position, duration, playing, color = 'var(--teal)', thick = false, onseek }:
    { position: number; duration: number; playing: boolean; color?: string; thick?: boolean; onseek?: (s: number) => void } = $props();

  const H = 12;
  const WL = $derived(thick ? 20 : 14); // comprimento da onda (px)
  const A = $derived(thick ? 3.2 : 2.4); // amplitude
  // caminho longo o bastante para qualquer largura; quadráticas suaves repetidas
  const d = $derived(`M0 ${H / 2} q${WL / 4} ${-A * 2} ${WL / 2} 0` + ` t${WL / 2} 0`.repeat(400));

  let el: HTMLDivElement;
  let drag = $state<number | null>(null);
  const shown = $derived(drag ?? position);
  const pct = $derived(duration ? Math.min(100, (shown / duration) * 100) : 0);

  const at = (e: PointerEvent) => {
    const r = el.getBoundingClientRect();
    return Math.max(0, Math.min(1, (e.clientX - r.left) / r.width)) * duration;
  };
  function down(e: PointerEvent) {
    if (!onseek || !duration) return;
    e.stopPropagation();
    el.setPointerCapture(e.pointerId);
    drag = at(e);
  }
  const move = (e: PointerEvent) => { if (drag !== null) drag = at(e); };
  function up() {
    if (drag === null) return;
    onseek?.(drag);
    // segura o valor até o próximo evento de mídia trazer a posição nova
    setTimeout(() => (drag = null), 1200);
  }
  function key(e: KeyboardEvent) {
    if (!onseek || !duration) return;
    if (e.key === 'ArrowRight') onseek(Math.min(duration, position + 5));
    if (e.key === 'ArrowLeft') onseek(Math.max(0, position - 5));
  }
</script>

<div class="wb" class:thick class:playing={playing && drag === null} class:seek={!!onseek} bind:this={el}
  style="--c:{color};--pct:{pct}%;--wl:{WL}px"
  role="slider" tabindex={onseek ? 0 : -1} aria-label="Posição da música" aria-valuemin={0} aria-valuemax={Math.round(duration)}
  aria-valuenow={Math.round(shown)} onpointerdown={down} onpointermove={move} onpointerup={up} onpointercancel={up} onkeydown={key}>
  <div class="played">
    <svg class="wave" height={H} aria-hidden="true">
      <path {d} />
    </svg>
  </div>
  <div class="rest"></div>
  <div class="thumb"></div>
</div>

<style>
  .wb { position: relative; height: 16px; width: 100%; touch-action: none; outline-offset: 4px; border-radius: 4px; }
  .wb.thick { height: 22px; }
  .seek { cursor: pointer; }
  .played { position: absolute; left: 0; top: 0; bottom: 0; width: var(--pct); overflow: hidden;
    mask: linear-gradient(90deg, #000 calc(100% - 6px), transparent); }
  .wave {
    position: absolute; left: 0; top: 50%; width: 4000px; translate: 0 -50%;
    transform: scaleY(0.15); transition: transform 0.45s cubic-bezier(0.3, 0.7, 0.2, 1);
    filter: drop-shadow(0 0 4px color-mix(in srgb, var(--c) 60%, transparent));
  }
  .wave path { fill: none; stroke: var(--c); stroke-width: 2.6; stroke-linecap: round; }
  .thick .wave path { stroke-width: 3.4; }
  .playing .wave { transform: scaleY(1); animation: run 1.1s linear infinite; }
  @keyframes run { from { translate: 0 -50%; } to { translate: calc(var(--wl) * -1) -50%; } }
  .rest {
    position: absolute; left: calc(var(--pct) + 6px); right: 0; top: 50%; height: 3px; translate: 0 -50%;
    background: rgba(233, 244, 236, 0.18); border-radius: 2px;
  }
  .thick .rest { height: 4px; }
  .thumb {
    position: absolute; left: var(--pct); top: 50%; width: 5px; height: 16px; translate: -50% -50%;
    background: var(--c); border-radius: 3px; box-shadow: 0 0 10px var(--c);
    transition: height 0.2s;
  }
  .thick .thumb { width: 6px; height: 22px; }
  .seek:hover .thumb { height: 20px; }
  @media (prefers-reduced-motion: reduce) { .playing .wave { animation: none; } }
</style>
