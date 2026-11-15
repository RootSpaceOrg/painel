<script lang="ts">
  import { ui, saveLayout, newWidget, newTab, COLS, ROWS, type WidgetType, type Placed } from '../lib/state.svelte';
  import { WIDGETS } from '../widgets/registry';
  import Icon from './Icon.svelte';

  let picking = $state(false);
  const tab = $derived(ui.tabs[ui.active]);

  // primeiro espaço livre: tenta o tamanho padrão, depois o mínimo
  function findSpot(ws: Placed[], w: number, h: number) {
    for (let y = 0; y + h <= ROWS; y++)
      for (let x = 0; x + w <= COLS; x++)
        if (!ws.some(o => x < o.x + o.w && o.x < x + w && y < o.y + o.h && o.y < y + h)) return { x, y };
    return null;
  }
  function add(type: WidgetType) {
    const d = WIDGETS[type];
    for (const [w, h] of [d.size, d.min]) {
      const s = findSpot(tab.widgets, w, h);
      if (s) { tab.widgets.push(newWidget(type, s.x, s.y, w, h)); saveLayout(); picking = false; return; }
    }
    alert('Sem espaço nesta aba. Diminua ou remova um widget, ou crie outra aba.');
  }
  function addTab() {
    ui.tabs.push(newTab(`Aba ${ui.tabs.length + 1}`));
    ui.active = ui.tabs.length - 1;
    saveLayout();
  }
  function removeTab() {
    if (ui.tabs.length < 2 || !confirm(`Excluir a aba "${tab.name}"?`)) return;
    ui.tabs.splice(ui.active, 1);
    ui.active = Math.max(0, ui.active - 1);
    saveLayout();
  }
  function moveTab(dir: -1 | 1) {
    const j = ui.active + dir;
    if (j < 0 || j >= ui.tabs.length) return;
    const [t] = ui.tabs.splice(ui.active, 1);
    ui.tabs.splice(j, 0, t);
    ui.active = j;
    saveLayout();
  }
</script>

<nav class="dock glass" aria-label="Abas">
  {#each ui.tabs as t, i (t.id)}
    {#if ui.editing && i === ui.active}
      <input class="rename" bind:value={t.name} onchange={saveLayout} aria-label="Nome da aba" size={Math.max(4, t.name.length)} />
    {:else}
      <button class="tab" class:on={i === ui.active} onclick={() => (ui.active = i)}>{t.name}</button>
    {/if}
  {/each}

  <span class="sep"></span>

  {#if ui.editing}
    <button class="ghost icon" aria-label="Mover aba para a esquerda" onclick={() => moveTab(-1)}>‹</button>
    <button class="ghost icon" aria-label="Mover aba para a direita" onclick={() => moveTab(1)}>›</button>
    <button class="ghost icon danger" aria-label="Excluir aba" disabled={ui.tabs.length < 2} onclick={removeTab}><Icon name="trash" /></button>
    <button class="ghost" onclick={addTab}>+ aba</button>
    <div class="pick">
      <button class="primary" onclick={() => (picking = !picking)} aria-expanded={picking}>+ widget</button>
      {#if picking}
        <div class="menu glass">
          {#each Object.entries(WIDGETS) as [k, d]}
            <button class="ghost" onclick={() => add(k as WidgetType)}>{d.label}</button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
  <button class="ghost icon" class:on={ui.editing} aria-label={ui.editing ? 'Concluir edição' : 'Editar layout'} aria-pressed={ui.editing}
    onclick={() => { ui.editing = !ui.editing; picking = false; }}>
    <Icon name={ui.editing ? 'check' : 'edit'} />
  </button>
  <button class="ghost icon" aria-label="Configurações" onclick={() => (ui.settings = true)}><Icon name="gear" /></button>
</nav>

<style>
  .dock {
    position: relative; z-index: 10;
    align-self: center; margin: 12px 0 12px; display: flex; align-items: center; gap: 4px;
    padding: 5px 6px; border-radius: 999px; max-width: calc(100% - 28px);
  }
  .tab { background: transparent; border-color: transparent; border-radius: 999px; padding: 5px 16px; color: var(--muted); position: relative; }
  .tab.on { color: var(--text); background: rgba(190, 240, 225, 0.1); }
  .tab.on::after {
    content: ''; position: absolute; left: 50%; bottom: 1px; width: 16px; height: 2px; translate: -50% 0;
    background: var(--teal); border-radius: 1px; box-shadow: 0 0 8px var(--teal);
  }
  .rename { border-radius: 999px; padding: 4px 14px; text-align: center; border-color: rgba(127, 224, 208, 0.5); }
  .sep { width: 1px; height: 20px; background: var(--line-strong); margin: 0 6px; }
  .icon { width: 30px; height: 30px; }
  .on.icon { color: var(--teal); }
  .pick { position: relative; }
  .pick > button { border-radius: 999px; }
  .menu {
    position: absolute; bottom: calc(100% + 10px); right: 0; display: flex; flex-direction: column;
    padding: 6px; min-width: 170px; background: var(--glass-strong); z-index: 20;
  }
  .menu button { text-align: left; }
</style>
