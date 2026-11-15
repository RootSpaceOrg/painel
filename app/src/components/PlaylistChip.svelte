<script lang="ts">
  // Mostra se a música atual está na playlist que está tocando; se não estiver (ex.: Smart Shuffle), permite adicionar.
  import { spx, addToPlaylist } from '../lib/spotify.svelte';
  import Icon from './Icon.svelte';
  let { big = false }: { big?: boolean } = $props();
  const p = $derived(spx.playlist);
</script>

{#if p && spx.inList !== null}
  {#if spx.inList}
    <span class="pc in" class:big title="Esta música faz parte da playlist">
      <Icon name="check" /><span class="ellipsis">{p.name}</span>
    </span>
  {:else if p.own}
    <button class="pc add" class:big onclick={e => { e.stopPropagation(); addToPlaylist(); }} disabled={spx.adding}
      title="Esta música não está na playlist (recomendação do Smart Shuffle?)">
      <Icon name="plus" /><span class="ellipsis">{spx.adding ? 'Adicionando…' : `Adicionar à ${p.name}`}</span>
    </button>
  {:else}
    <span class="pc out" class:big title="Esta música não faz parte da playlist">
      <Icon name="music" /><span class="ellipsis">fora de {p.name}</span>
    </span>
  {/if}
  {#if spx.err && !spx.inList}<span class="err" class:big>{spx.err}</span>{/if}
{/if}

<style>
  .pc {
    display: inline-flex; align-items: center; gap: 5px; max-width: 100%; min-width: 0;
    font-size: 11px; padding: 2px 9px 2px 7px; border-radius: 999px; border: 1px solid transparent; line-height: 1.5;
  }
  .pc :global(svg) { width: 11px; height: 11px; flex: none; }
  .in { color: var(--faint); padding-left: 0; }
  .out { color: var(--muted); background: rgba(0, 0, 0, 0.2); }
  .add { color: var(--ink); background: var(--accent, var(--teal)); font-weight: 600; cursor: pointer; }
  .add:hover { background: var(--accent, var(--teal)); filter: brightness(1.08); }
  .big { font-size: 13px; padding: 4px 12px 4px 10px; }
  .big :global(svg) { width: 14px; height: 14px; }
  .big.in { padding-left: 0; }
  .err { color: var(--danger); font-size: 11px; margin-left: 6px; align-self: center; }
  .err.big { font-size: 12px; }
</style>
