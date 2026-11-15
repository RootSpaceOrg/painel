<script lang="ts">
  import { ui, saveLayout, COLS, ROWS, type Tab, type Placed } from '../lib/state.svelte';
  import { WIDGETS } from '../widgets/registry';
  import Icon from './Icon.svelte';

  let { tab }: { tab: Tab } = $props();
  let gridEl: HTMLDivElement;
  let drag = $state<{ w: Placed; mode: 'move' | 'resize'; sx: number; sy: number; o: Placed } | null>(null);

  const clamp = (v: number, a: number, b: number) => Math.max(a, Math.min(b, v));
  const hits = (a: Placed, b: Placed) => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
  const collides = (w: Placed) => tab.widgets.some(o => o.id !== w.id && hits(o, w));

  function down(e: PointerEvent, w: Placed, mode: 'move' | 'resize') {
    if ((e.target as Element).closest('.rm')) return;
    e.preventDefault();
    e.stopPropagation();
    (e.currentTarget as Element).setPointerCapture(e.pointerId);
    drag = { w, mode, sx: e.clientX, sy: e.clientY, o: { ...w } };
  }
  function move(e: PointerEvent) {
    if (!drag) return;
    const r = gridEl.getBoundingClientRect();
    const dx = Math.round((e.clientX - drag.sx) / (r.width / COLS));
    const dy = Math.round((e.clientY - drag.sy) / (r.height / ROWS));
    const { w, o } = drag;
    const [mw, mh] = WIDGETS[w.type].min;
    if (drag.mode === 'move') {
      w.x = clamp(o.x + dx, 0, COLS - w.w);
      w.y = clamp(o.y + dy, 0, ROWS - w.h);
    } else {
      w.w = clamp(o.w + dx, mw, COLS - w.x);
      w.h = clamp(o.h + dy, mh, ROWS - w.y);
    }
  }
  function up() {
    if (!drag) return;
    if (collides(drag.w)) Object.assign(drag.w, drag.o); // não deixa sobrepor: volta
    drag = null;
    saveLayout();
  }
  function remove(w: Placed) {
    tab.widgets.splice(tab.widgets.indexOf(w), 1);
    saveLayout();
  }
  const expand = (w: Placed) => { if (!ui.editing) ui.expanded = w; };
</script>

<div class="grid" role="presentation" class:editing={ui.editing} bind:this={gridEl} onpointermove={move} onpointerup={up} onpointercancel={up}>
  {#if ui.editing}
    {#each Array(COLS * ROWS) as _, i}<div class="cell" style="grid-column:{(i % COLS) + 1};grid-row:{Math.floor(i / COLS) + 1}"></div>{/each}
  {/if}

  {#each tab.widgets as w (w.id)}
    {@const def = WIDGETS[w.type]}
    <section class="card glass" class:bad={drag?.w === w && collides(w)} class:dragging={drag?.w === w}
      style="grid-column:{w.x + 1} / span {w.w};grid-row:{w.y + 1} / span {w.h}">
      {#if def.bare}
        <div class="body click" role="button" tabindex="0" aria-label="Abrir {def.label}"
          onclick={() => expand(w)} onkeydown={e => e.key === 'Enter' && expand(w)}>
          <def.comp />
        </div>
      {:else}
        <header>
          <button class="title label" onclick={() => expand(w)}>{def.label}</button>
          <button class="ghost icon exp" aria-label="Expandir {def.label}" onclick={() => expand(w)}><Icon name="expand" /></button>
        </header>
        <div class="body"><def.comp /></div>
      {/if}

      {#if ui.editing}
        <div class="edit" role="presentation" onpointerdown={e => down(e, w, 'move')}>
          <span class="name">{def.label}</span>
          <span class="faint mono">{w.w}×{w.h}</span>
          <button class="icon rm" aria-label="Remover {def.label}" onclick={() => remove(w)}><Icon name="x" /></button>
          <span class="rs" role="presentation" onpointerdown={e => down(e, w, 'resize')}></span>
        </div>
      {/if}
    </section>
  {/each}
</div>

<style>
  .grid {
    flex: 1; min-height: 0; display: grid; gap: 12px; padding: 14px 14px 0;
    grid-template-columns: repeat(12, minmax(0, 1fr)); grid-template-rows: repeat(8, minmax(0, 1fr));
  }
  .cell { border: 1px dashed rgba(190, 240, 225, 0.1); border-radius: 10px; }
  .card { overflow: hidden; position: relative; display: flex; flex-direction: column; min-height: 0; min-width: 0; padding: 12px 14px 14px; z-index: 1; }
  header { display: flex; align-items: center; justify-content: space-between; margin: -4px -6px 6px 0; }
  .title { background: none; border: 0; padding: 4px 0; }
  .title:hover { color: var(--muted); background: none; }
  .exp { width: 26px; height: 26px; opacity: 0; color: var(--faint); }
  .card:hover .exp { opacity: 1; }
  .exp :global(svg) { width: 13px; height: 13px; }
  .body { flex: 1; min-height: 0; position: relative; }
  .click { cursor: pointer; outline-offset: 4px; border-radius: 8px; }
  .edit {
    position: absolute; inset: 0; border-radius: var(--radius); cursor: grab; z-index: 2;
    background: rgba(7, 22, 26, 0.72); border: 1.5px solid rgba(127, 224, 208, 0.5);
    display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px;
  }
  .dragging { z-index: 3; }
  .dragging .edit { cursor: grabbing; background: rgba(127, 224, 208, 0.16); }
  .bad .edit { border-color: var(--danger); background: rgba(242, 154, 154, 0.14); }
  .name { font-weight: 600; }
  .rm { position: absolute; top: 8px; right: 8px; width: 28px; height: 28px; }
  .rm:hover { color: var(--danger); }
  .rs {
    position: absolute; right: 4px; bottom: 4px; width: 18px; height: 18px; cursor: nwse-resize;
    border-right: 2px solid var(--teal); border-bottom: 2px solid var(--teal); border-radius: 0 0 6px 0;
  }
</style>
