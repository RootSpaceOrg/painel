<script lang="ts">
  import { live, prefs, savePrefs } from '../lib/state.svelte';
  import Lyrics from '../components/Lyrics.svelte';
  import { mediaControl } from '../lib/api';
  import { fmtDuration } from '../lib/util';
  import Icon from '../components/Icon.svelte';
  import WaveBar from '../components/WaveBar.svelte';
  import FullPlayer from '../components/FullPlayer.svelte';
  import PlaylistChip from '../components/PlaylistChip.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  const m = $derived(live.media);
  const has = $derived(!!m?.active && !!(m.title || m.artist));
  const accent = $derived(live.artColor || 'var(--teal)');
  const seek = (s: number) => mediaControl('seek', s);

</script>

{#snippet controls(size: 'sm' | 'lg')}
  <div class="ctrl {size}" style="--accent:{accent}">
    <button class="ghost icon" aria-label="Anterior" onclick={() => mediaControl('prev')}><Icon name="prev" /></button>
    <button class="icon play" class:round={!m?.playing} aria-label={m?.playing ? 'Pausar' : 'Tocar'} onclick={() => mediaControl('toggle')}>
      <Icon name={m?.playing ? 'pause' : 'play'} />
    </button>
    <button class="ghost icon" aria-label="Próxima" onclick={() => mediaControl('next')}><Icon name="next" /></button>
  </div>
{/snippet}

{#if !has}
  <div class="none">
    <Icon name="music" />
    <span>Nada tocando</span>
  </div>
{:else if m && !expanded}
  <div class="compact" style="--accent:{accent}">
    {#if live.art}<div class="glow sm" style="background-image:url({live.art})"></div>{/if}
    <div class="art">{#if live.art}<img src={live.art} alt="" />{:else}<Icon name="music" />{/if}</div>
    <div class="info">
      {#if prefs.miniLyrics}
        <!-- letra: título e artista numa linha, linha atual da letra em destaque -->
        <div class="tline ellipsis"><b>{m.title}</b> <span class="muted">· {m.artist}</span></div>
        <div class="lyr"><Lyrics mini artist={m.artist} title={m.title} album={m.album} duration={m.duration} position={m.position} playing={m.playing} /></div>
      {:else}
        <div class="title ellipsis">{m.title}</div>
        <div class="artist ellipsis muted">{m.artist}</div>
        <div class="plc"><PlaylistChip /></div>
      {/if}
      <div class="wave-row">
        <WaveBar position={m.position} duration={m.duration} playing={m.playing} color={accent} onseek={m.duration ? seek : undefined} />
      </div>
      <div class="row between">
        <span class="mono faint t">{fmtDuration(m.position)}</span>
        {@render controls('sm')}
        <span class="end">
          <button class="ghost icon lyrbtn" class:on={prefs.miniLyrics} aria-pressed={prefs.miniLyrics} title="Letra"
            aria-label={prefs.miniLyrics ? 'Esconder letra' : 'Mostrar letra'}
            onclick={() => { prefs.miniLyrics = !prefs.miniLyrics; savePrefs(); }}><Icon name="lyrics" /></button>
          <span class="mono faint t">{m.duration ? fmtDuration(m.duration) : ''}</span>
        </span>
      </div>
    </div>
  </div>
{:else}
  <FullPlayer />
{/if}

<style>
  .none { height: 100%; display: grid; place-content: center; justify-items: center; gap: 8px; color: var(--faint); }
  .none :global(svg) { width: 28px; height: 28px; }
  .compact { position: relative; isolation: isolate; display: flex; gap: 16px; height: 100%; align-items: center; min-width: 0; }
  .glow.sm { inset: -60px; filter: blur(46px) saturate(1.5); opacity: 0.32; transition: background-image 0.6s; }
  .wave-row { margin: 8px 0 2px; }
  .plc { display: flex; min-width: 0; min-height: 0; }
  .tline { font-size: 13px; }
  .lyr { min-height: 44px; display: flex; align-items: center; margin-top: 2px; }
  .end { display: flex; align-items: center; gap: 2px; }
  .lyrbtn { width: 26px; height: 26px; color: var(--faint); }
  .lyrbtn :global(svg) { width: 14px; height: 14px; }
  .lyrbtn.on { color: var(--accent); }
  .art {
    height: 100%; max-height: 150px; aspect-ratio: 1; border-radius: 12px; overflow: hidden; flex: none;
    background: rgba(190, 240, 225, 0.06); display: grid; place-items: center; color: var(--faint);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
  }
  .art :global(svg) { width: 38%; height: 38%; }
  .art img { width: 100%; height: 100%; object-fit: cover; }
  .info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
  .title { font-size: 16px; font-weight: 600; }
  .between { justify-content: space-between; }
  .t { font-size: 11px; }
  .ctrl { display: flex; align-items: center; gap: 4px; }
  /* One UI: botão central em "squircle" na cor da capa, que vira círculo ao pausar */
  .play {
    background: var(--accent); color: var(--ink); border: 0; border-radius: 12px;
    box-shadow: 0 4px 16px color-mix(in srgb, var(--accent) 45%, transparent);
    transition: border-radius 0.35s cubic-bezier(0.3, 0.7, 0.2, 1), transform 0.15s, filter 0.15s;
  }
  .play:hover { background: var(--accent); filter: brightness(1.08); }
  .play:active, .ctrl .ghost:active { transform: scale(0.9); }
  .ctrl .play.round { border-radius: 50%; }
  .glow { position: absolute; background-size: cover; background-position: center; z-index: -1; pointer-events: none; }
</style>
