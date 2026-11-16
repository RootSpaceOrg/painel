<script lang="ts" module>
  import type { Lyrics as L } from '../lib/api';
  // uma busca por música, compartilhada entre o widget e o player (erros não ficam em cache)
  const cache = new Map<string, Promise<L | null>>();
</script>

<script lang="ts">
  // Letra sincronizada (LRCLIB). `mini`: só a linha atual e a próxima (widget pequeno). Relógio local suaviza a posição entre os eventos de mídia (1/s).
  import { lyrics as fetchLyrics, mediaControl } from '../lib/api';
  import { parseLrc, lrcIndex } from '../lib/util';
  import { untrack } from 'svelte';

  let { artist, title, album, duration, position, playing, mini = false }:
    { artist: string; title: string; album: string; duration: number; position: number; playing: boolean; mini?: boolean } = $props();

  let status = $state<'loading' | 'synced' | 'plain' | 'none' | 'instrumental' | 'error'>('loading');
  let lines = $state<{ t: number; text: string }[]>([]);
  let plain = $state('');

  // As props vêm do objeto de mídia, que é trocado a cada segundo; o $derived só notifica quando
  // a string muda de fato, então a busca roda uma vez por música (não a cada evento).
  const key = $derived(`${artist}|${title}`);
  $effect(() => {
    key;
    status = 'loading';
    lines = [];
    let alive = true;
    const k = key;
    let req = cache.get(k);
    if (!req) {
      req = untrack(() => fetchLyrics(artist, title, album, duration));
      req.catch(() => cache.delete(k));
      cache.set(k, req);
    }
    req
      .then(r => {
        if (!alive) return;
        if (!r) status = 'none';
        else if (r.instrumental) status = 'instrumental';
        else if (r.syncedLyrics) { lines = parseLrc(r.syncedLyrics); status = 'synced'; }
        else if (r.plainLyrics) { plain = r.plainLyrics; status = 'plain'; }
        else status = 'none';
      })
      .catch(() => alive && (status = 'error'));
    return () => { alive = false; };
  });

  // posição suave: base do último evento + tempo decorrido
  let base = { pos: 0, at: 0 };
  let now = $state(0);
  $effect(() => { base = { pos: position, at: performance.now() }; now = position; });
  $effect(() => {
    if (!playing || status !== 'synced') return;
    const t = setInterval(() => (now = base.pos + (performance.now() - base.at) / 1000), 200);
    return () => clearInterval(t);
  });

  // pequena antecipação: a linha acende um pouco antes, como nos apps de música
  const idx = $derived(lrcIndex(lines, now + 0.25));
  let box: HTMLDivElement | undefined = $state();
  const msg = $derived(
    status === 'loading' ? 'Buscando letra…' : status === 'instrumental' ? 'Instrumental ♪' : status === 'error' ? 'Não deu para buscar a letra agora.' : status === 'plain' ? 'Letra sem sincronia (veja no modo tela cheia)' : 'Letra não encontrada.',
  );
  $effect(() => {
    if (!box || idx < 0) return;
    const el = box.children[idx] as HTMLElement | undefined;
    if (el) box.scrollTo({ top: el.offsetTop - box.clientHeight * 0.4, behavior: 'smooth' });
  });
</script>

{#if mini}
  <div class="mini">
    {#if status === 'synced'}
      {#key idx}<div class="cur">{(idx >= 0 && lines[idx].text) || '♪'}</div>{/key}
      <div class="nxt ellipsis">{lines[idx + 1]?.text ?? ''}</div>
    {:else}
      <div class="nxt">{msg}</div>
    {/if}
  </div>
{:else if status === 'synced'}
  <div class="lines" bind:this={box}>
    {#each lines as l, i}
      <button class="ln" class:on={i === idx} class:past={i < idx} onclick={() => mediaControl('seek', l.t)}>
        {l.text || '♪'}
      </button>
    {/each}
  </div>
{:else if status === 'plain'}
  <div class="lines plain">{plain}</div>
{:else}
  <div class="msg">{msg}</div>
{/if}

<style>
  .lines {
    height: 100%; overflow-y: auto; display: flex; flex-direction: column; gap: 6px; padding: 30% 8px 50%;
    mask: linear-gradient(180deg, transparent, #000 14%, #000 80%, transparent);
    scrollbar-width: none;
  }
  .lines::-webkit-scrollbar { display: none; }
  .ln {
    background: none; border: 0; padding: 4px 0; text-align: left; border-radius: 8px;
    font: 700 24px/1.3 var(--sans); color: rgba(233, 244, 236, 0.3);
    transition: color 0.35s, transform 0.35s, text-shadow 0.35s; transform-origin: left center;
  }
  .ln:hover { background: none; color: rgba(233, 244, 236, 0.6); }
  .ln.past { color: rgba(233, 244, 236, 0.45); }
  .ln.on { color: var(--text); transform: scale(1.04); text-shadow: 0 0 24px color-mix(in srgb, var(--accent, var(--teal)) 55%, transparent); }
  .plain { white-space: pre-wrap; font-size: 18px; line-height: 1.6; color: var(--muted); user-select: text; padding: 16px 8px; }
  .msg { height: 100%; display: grid; place-items: center; color: var(--faint); font-size: 16px; }
  .mini { display: flex; flex-direction: column; gap: 2px; min-width: 0; }
  .cur {
    font: 700 15px/1.3 var(--sans); color: var(--text); text-shadow: 0 0 18px color-mix(in srgb, var(--accent, var(--teal)) 45%, transparent);
    display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;
    animation: up 0.35s ease-out;
  }
  .nxt { font-size: 12px; color: var(--faint); }
  @keyframes up { from { opacity: 0; transform: translateY(6px); } }
  @media (prefers-reduced-motion: reduce) { .ln { transition: none; } .cur { animation: none; } }
</style>
