<script lang="ts">
  // Controle de volume do que está tocando. `compact`: só o botão, o slider abre num balão (cards estreitos).
  import { live } from '../lib/state.svelte';
  import { spx } from '../lib/spotify.svelte';
  import { av, setVol, setDragging, useVolume, loadVol, volLabel } from '../lib/volume.svelte';
  import Icon from './Icon.svelte';

  let { compact = false }: { compact?: boolean } = $props();
  let open = $state(false);

  $effect(() => useVolume());
  const activeId = $derived(live.media?.active ?? '');
  $effect(() => {
    activeId; // troca de fonte → reavalia na hora
    spx.on;
    loadVol();
  });
  const title = $derived(av.spVol ? 'Volume do Spotify (a barra de volume do app)' : av.target ? `Volume do ${av.target.name} (só este app)` : 'Volume geral');
</script>

{#snippet slider()}
  {#if av.vol !== null}
    <input type="range" min="0" max="100" value={Math.round(av.vol * 100)} style="--p:{av.vol * 100}" aria-label={volLabel()}
      onpointerdown={() => setDragging(true)} onpointerup={() => setDragging(false)}
      oninput={e => setVol(+e.currentTarget.value / 100)} />
  {/if}
{/snippet}

{#snippet ic()}
  {#if av.icon}<img class="vapp" src={av.icon} alt="" />{:else}<Icon name="volume" />{/if}
{/snippet}

{#if compact}
  <span class="pw">
    <button class="ghost icon vb" class:on={open} aria-expanded={open} {title} aria-label={volLabel()}
      onclick={e => { e.stopPropagation(); open = !open; }}>{@render ic()}</button>
    {#if open}
      <div class="pop glass" role="group" aria-label={volLabel()}>
        {@render slider()}
        <span class="mono pc">{av.vol !== null ? Math.round(av.vol * 100) : ''}</span>
      </div>
    {/if}
  </span>
{:else}
  <div class="vol" {title}>
    {@render ic()}
    {@render slider()}
  </div>
{/if}

<style>
  .vol { display: flex; align-items: center; gap: 8px; color: var(--muted); min-width: 0; width: 100%; }
  .vol :global(svg) { width: 18px; height: 18px; flex: none; }
  .vol input { flex: 1; min-width: 40px; }
  .vapp { width: 20px; height: 20px; flex: none; border-radius: 4px; }
  .pw { position: relative; display: inline-flex; }
  .vb { width: 26px; height: 26px; color: var(--faint); }
  .vb.on { color: var(--accent, var(--teal)); }
  .vb :global(svg), .vb .vapp { width: 15px; height: 15px; }
  .pop {
    position: absolute; bottom: calc(100% + 8px); right: -6px; z-index: 5; display: flex; align-items: center; gap: 10px;
    width: 170px; padding: 10px 12px; background: var(--glass-strong);
  }
  .pop input { flex: 1; }
  .pc { font-size: 11px; color: var(--muted); width: 24px; text-align: right; }
</style>
