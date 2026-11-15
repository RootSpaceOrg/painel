<script lang="ts">
  import { ui, prefs, boot } from './lib/state.svelte';
  import Grid from './components/Grid.svelte';
  import Dock from './components/Dock.svelte';
  import Expanded from './components/Expanded.svelte';
  import Settings from './components/Settings.svelte';
  import UpdateToast from './components/UpdateToast.svelte';
  import { schedule } from './lib/update.svelte';

  boot();
  schedule();

  function key(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (ui.expanded) ui.expanded = null;
      else if (ui.settings) ui.settings = false;
      else if (ui.editing) ui.editing = false;
      return;
    }
    const typing = (e.target as Element).closest('input, textarea, select');
    if (typing || ui.expanded || ui.settings) return;
    if (e.key === 'ArrowRight') ui.active = (ui.active + 1) % ui.tabs.length;
    else if (e.key === 'ArrowLeft') ui.active = (ui.active - 1 + ui.tabs.length) % ui.tabs.length;
    else if (/^[1-9]$/.test(e.key) && +e.key <= ui.tabs.length) ui.active = +e.key - 1;
  }
</script>

<svelte:window onkeydown={key} oncontextmenu={e => e.preventDefault()} />

<div class="bg" class:wall={prefs.wallUrl} style={prefs.wallUrl ? `background-image:url(${prefs.wallUrl})` : ''}></div>
<div class="veil" style="--dim:{prefs.dim}"></div>

<main style="--blur:{prefs.blur}px">
  {#key ui.active}
    {#if ui.tabs[ui.active]}<Grid tab={ui.tabs[ui.active]} />{/if}
  {/key}
  <Dock />
  {#if ui.expanded}<Expanded />{/if}
  {#if ui.settings}<Settings />{/if}
  <UpdateToast />
</main>

<style>
  .bg {
    position: fixed; inset: 0; z-index: -2;
    /* sem wallpaper: mata ao entardecer em gradientes */
    background:
      radial-gradient(60% 50% at 22% 30%, rgba(127, 224, 208, 0.22), transparent 70%),
      radial-gradient(45% 40% at 78% 18%, rgba(243, 169, 196, 0.12), transparent 70%),
      radial-gradient(70% 55% at 50% 100%, rgba(125, 187, 106, 0.25), transparent 70%),
      linear-gradient(180deg, #0c2a30 0%, #0a2226 45%, #07161a 100%);
  }
  .bg.wall { background-size: cover; background-position: center; }
  .veil {
    position: fixed; inset: 0; z-index: -1; pointer-events: none;
    background: linear-gradient(180deg, rgba(4, 16, 18, calc(var(--dim) * 0.7)), rgba(4, 16, 18, var(--dim)));
  }
  main { height: 100%; display: flex; flex-direction: column; }
</style>
