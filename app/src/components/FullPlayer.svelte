<script lang="ts">
  // Player em tela cheia. Topo: herói (capa grande à direita ou letra 50/50). Rolando: fila, dispositivos, playlists.
  import { live } from '../lib/state.svelte';
  import { mediaControl, mediaSelect, spotifyStatus, spotifyApi } from '../lib/api';
  import { fmtDuration } from '../lib/util';
  import Icon from './Icon.svelte';
  import WaveBar from './WaveBar.svelte';
  import Lyrics from './Lyrics.svelte';

  const m = $derived(live.media!);
  const accent = $derived(live.artColor || 'var(--teal)');
  const source = $derived(m.sessions.find(s => s.id === m.active)?.app ?? '');
  let showLyrics = $state(false);
  let scroller: HTMLDivElement;

  // ---------- Spotify ----------
  type Img = { url: string }[];
  let sp = $state({
    on: false, err: '', liked: null as boolean | null, uri: '', shuffle: false, repeat: 'off',
    queue: [] as any[], playlists: [] as any[], devices: [] as any[],
  });
  // cada chamada é independente: uma falhar não esvazia as outras
  const safe = async <T,>(p: Promise<T>, fb: T): Promise<T> => {
    try { return await p; } catch (e) { sp.err = String(e); return fb; }
  };

  async function refresh() {
    sp.err = '';
    const pl = await safe(spotifyApi('GET', '/me/player'), null);
    sp.uri = pl?.item?.uri ?? '';
    sp.shuffle = !!pl?.shuffle_state;
    sp.repeat = pl?.repeat_state ?? 'off';
    // /me/tracks foi desativado pela Spotify (403); a biblioteca agora é /me/library com URIs
    sp.liked = sp.uri ? ((await safe(spotifyApi<boolean[]>('GET', `/me/library/contains?uris=${sp.uri}`), [null as any]))[0] ?? null) : null;
    sp.queue = ((await safe(spotifyApi('GET', '/me/player/queue'), null))?.queue ?? []).slice(0, 20);
    sp.devices = (await safe(spotifyApi('GET', '/me/player/devices'), null))?.devices ?? [];
  }
  $effect(() => {
    spotifyStatus().then(async st => {
      sp.on = !!st?.connected;
      if (!sp.on) return;
      refresh();
      sp.playlists = (await safe(spotifyApi('GET', '/me/playlists?limit=50'), null))?.items?.filter(Boolean) ?? [];
    }).catch(() => {});
  });
  const title = $derived(m.title); // só muda quando a faixa muda
  $effect(() => {
    if (!sp.on || !title) return;
    const t = setTimeout(refresh, 800);
    return () => clearTimeout(t);
  });

  const act = (p: Promise<unknown>) => p.then(() => setTimeout(refresh, 600)).catch(e => (sp.err = String(e)));
  const toggleLike = () => act(spotifyApi(sp.liked ? 'DELETE' : 'PUT', `/me/library?uris=${sp.uri}`));
  const toggleShuffle = () => act(spotifyApi('PUT', `/me/player/shuffle?state=${!sp.shuffle}`));
  const nextRepeat = () => act(spotifyApi('PUT', `/me/player/repeat?state=${sp.repeat === 'off' ? 'context' : sp.repeat === 'context' ? 'track' : 'off'}`));
  const playContext = (uri: string) => act(spotifyApi('PUT', '/me/player/play', { context_uri: uri }));
  const transfer = (id: string) => act(spotifyApi('PUT', '/me/player', { device_ids: [id], play: true }));
  const img = (i: Img | undefined, small = false) => (i?.length ? i[small ? i.length - 1 : 0].url : '');
  const artists = (t: any) => t.artists?.map((a: any) => a.name).join(', ') ?? t.show?.name ?? '';
  const nice = (e: string) => (e.includes('403') ? 'A Spotify recusou um comando (alguns exigem Premium).' : e);
</script>

<div class="fp" bind:this={scroller} style="--accent:{accent}">
  <section class="hero" class:lyr={showLyrics}>
    {#if live.art}<div class="ambient" style="background-image:url({live.art})"></div>{/if}

    <div class="left">
      <div class="top row">
        {#if source}<span class="label">Tocando no {source}</span>{/if}
        {#if m.sessions.length > 1}
          <span class="grow"></span>
          {#each m.sessions as s}
            <button class="chip" class:on={s.id === m.active} onclick={() => mediaSelect(s.id)}>{s.app}</button>
          {/each}
        {/if}
      </div>

      <div class="meta">
        {#if showLyrics && live.art}<img class="mini-art" src={live.art} alt="" />{/if}
        <div class="grow">
          <h1>{m.title}</h1>
          <p class="artist">{m.artist}</p>
          {#if m.album}<p class="album faint">{m.album}</p>{/if}
        </div>
      </div>

      <div class="deck">
        <div class="row actions">
          {#if sp.on && sp.uri}
            <button class="ghost icon" class:liked={sp.liked} aria-label={sp.liked ? 'Remover das curtidas' : 'Curtir'} onclick={toggleLike}>
              <Icon name="heart" filled={!!sp.liked} />
            </button>
          {/if}
          <span class="grow"></span>
          <button class="chip" class:on={showLyrics} aria-pressed={showLyrics} onclick={() => (showLyrics = !showLyrics)}>
            <Icon name="lyrics" /> Letra
          </button>
        </div>
        {#if m.duration}
          <WaveBar thick position={m.position} duration={m.duration} playing={m.playing} color={accent} onseek={s => mediaControl('seek', s)} />
          <div class="row between mono faint t"><span>{fmtDuration(m.position)}</span><span>{fmtDuration(m.duration)}</span></div>
        {/if}
        <div class="ctrl">
          {#if sp.on}
            <button class="ghost icon side" class:act={sp.shuffle} aria-label="Aleatório" aria-pressed={sp.shuffle} onclick={toggleShuffle}><Icon name="shuffle" /></button>
          {/if}
          <button class="ghost icon" aria-label="Anterior" onclick={() => mediaControl('prev')}><Icon name="prev" /></button>
          <button class="icon play" class:round={!m.playing} aria-label={m.playing ? 'Pausar' : 'Tocar'} onclick={() => mediaControl('toggle')}>
            <Icon name={m.playing ? 'pause' : 'play'} />
          </button>
          <button class="ghost icon" aria-label="Próxima" onclick={() => mediaControl('next')}><Icon name="next" /></button>
          {#if sp.on}
            <button class="ghost icon side" class:act={sp.repeat !== 'off'} aria-label="Repetir: {sp.repeat}" onclick={nextRepeat}>
              <Icon name="repeat" />{#if sp.repeat === 'track'}<span class="one">1</span>{/if}
            </button>
          {/if}
        </div>
      </div>
    </div>

    <div class="right">
      {#if showLyrics}
        <Lyrics artist={m.artist} title={m.title} album={m.album} duration={m.duration} position={m.position} playing={m.playing} />
      {:else if live.art}
        <img class="cover" src={live.art} alt="Capa de {m.album || m.title}" />
      {/if}
    </div>

    <button class="more" onclick={() => scroller.scrollTo({ top: scroller.clientHeight, behavior: 'smooth' })}>
      {sp.on ? 'Fila e playlists' : 'Mais'} <Icon name="down" />
    </button>
  </section>

  {#if !sp.on}
    <section class="sec"><div class="empty">Conecte o Spotify nas configurações para ver fila, playlists e dispositivos.</div></section>
  {:else}
    {#if sp.err}<div class="err">{nice(sp.err)}</div>{/if}
    <section class="sec">
      <h2>A seguir</h2>
      <div class="rowscroll">
        {#each sp.queue as t, i (i)}
          <div class="tile">
            {#if img(t.album?.images ?? t.images)}<img src={img(t.album?.images ?? t.images)} alt="" loading="lazy" />{:else}<span class="ph"><Icon name="music" /></span>{/if}
            <span class="tn ellipsis">{t.name}</span>
            <span class="ts ellipsis">{artists(t)}</span>
          </div>
        {:else}<div class="faint">Fila vazia.</div>{/each}
      </div>
    </section>

    {#if sp.devices.length}
      <section class="sec">
        <h2>Ouvir em</h2>
        <div class="row wrap">
          {#each sp.devices as d}
            <button class="chip dev" class:on={d.is_active} onclick={() => transfer(d.id)}><Icon name="speaker" />{d.name}</button>
          {/each}
        </div>
      </section>
    {/if}

    <section class="sec">
      <h2>Suas playlists</h2>
      <div class="grid">
        {#each sp.playlists as p (p.id)}
          <button class="tile pl" onclick={() => playContext(p.uri)} title="Tocar {p.name}">
            <span class="cv">
              {#if img(p.images)}<img src={img(p.images)} alt="" loading="lazy" />{:else}<span class="ph"><Icon name="music" /></span>{/if}
              <span class="fab"><Icon name="play" /></span>
            </span>
            <span class="tn ellipsis">{p.name}</span>
            <span class="ts ellipsis">{p.owner?.display_name ?? ''}{(p.tracks ?? p.items)?.total ? ` · ${(p.tracks ?? p.items).total} músicas` : ''}</span>
          </button>
        {/each}
      </div>
    </section>
  {/if}
</div>

<style>
  .fp { height: 100%; overflow-y: auto; overflow-x: hidden; scroll-behavior: smooth; margin-right: -10px; padding-right: 10px; }

  /* ---------- herói ---------- */
  .hero {
    position: relative; isolation: isolate; height: 100%; min-height: 420px;
    display: grid; grid-template-columns: minmax(0, 0.9fr) minmax(0, 1.1fr); gap: 40px; padding-bottom: 36px;
  }
  .hero.lyr { grid-template-columns: 1fr 1fr; }
  .ambient {
    position: absolute; inset: -40px; z-index: -1; background-size: cover; background-position: center;
    filter: blur(90px) saturate(1.5); opacity: 0.3; pointer-events: none;
    mask: radial-gradient(closest-side, #000 55%, transparent); /* sem borda retangular */
  }
  .left { display: flex; flex-direction: column; min-width: 0; min-height: 0; }
  .top { min-height: 28px; flex-wrap: wrap; gap: 6px; }
  .meta { flex: 1; display: flex; align-items: center; gap: 18px; min-height: 0; }
  h1 { font-size: clamp(28px, 4.4vw, 54px); font-weight: 300; line-height: 1.05; margin: 0 0 10px; letter-spacing: -0.01em;
    display: -webkit-box; -webkit-line-clamp: 3; line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden; }
  .lyr h1 { font-size: clamp(22px, 2.6vw, 34px); }
  .artist { font-size: 18px; margin: 0; color: var(--accent); }
  .album { margin: 4px 0 0; font-size: 14px; }
  .mini-art { width: 120px; height: 120px; border-radius: 14px; object-fit: cover; box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4); flex: none; }
  .deck { display: flex; flex-direction: column; gap: 8px; }
  .actions { min-height: 36px; }
  .between { justify-content: space-between; }
  .t { font-size: 12px; }
  .ctrl { display: flex; align-items: center; justify-content: center; gap: 10px; margin-top: 4px; }
  .ctrl .icon { width: 44px; height: 44px; }
  .ctrl :global(svg) { width: 22px; height: 22px; }
  .side { color: var(--muted); position: relative; }
  .side.act { color: var(--accent); }
  .side.act::after { content: ''; position: absolute; bottom: 3px; left: 50%; width: 4px; height: 4px; border-radius: 50%; background: var(--accent); translate: -50% 0; }
  .one { position: absolute; top: 6px; right: 6px; font-size: 9px; font-weight: 700; }
  .play {
    width: 64px !important; height: 64px !important; border: 0; border-radius: 18px; background: var(--accent); color: var(--ink);
    box-shadow: 0 6px 22px color-mix(in srgb, var(--accent) 45%, transparent);
    transition: border-radius 0.35s cubic-bezier(0.3, 0.7, 0.2, 1), transform 0.15s, filter 0.15s;
  }
  .play:hover { background: var(--accent); filter: brightness(1.08); }
  .play.round { border-radius: 50%; }
  .ctrl button:active { transform: scale(0.9); }
  .actions .icon :global(svg) { width: 22px; height: 22px; }
  .liked { color: var(--blossom); }
  .chip { display: inline-flex; align-items: center; gap: 6px; padding: 5px 12px; border-radius: 999px; font-size: 12px; }
  .chip :global(svg) { width: 14px; height: 14px; }
  .chip.on { color: var(--ink); background: var(--accent); border-color: transparent; }

  .right { position: relative; min-height: 0; display: flex; align-items: center; justify-content: center; }
  .cover {
    max-width: 100%; max-height: 100%; aspect-ratio: 1; object-fit: cover; border-radius: 22px;
    box-shadow: 0 30px 70px rgba(0, 0, 0, 0.5), 0 0 90px color-mix(in srgb, var(--accent) 22%, transparent);
    animation: in 0.5s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  @keyframes in { from { opacity: 0; transform: scale(0.96); } }
  .more {
    position: absolute; left: 50%; bottom: 0; translate: -50% 0; display: inline-flex; align-items: center; gap: 6px;
    background: transparent; border: 0; color: var(--faint); font-size: 12px; letter-spacing: 0.08em; text-transform: uppercase;
  }
  .more:hover { color: var(--text); background: transparent; }
  .more :global(svg) { width: 14px; height: 14px; animation: bob 2s ease-in-out infinite; }
  @keyframes bob { 50% { transform: translateY(3px); } }

  /* ---------- seções ---------- */
  .sec { padding: 26px 0 8px; }
  h2 { font-size: 20px; font-weight: 400; margin: 0 0 14px; }
  .rowscroll { display: flex; gap: 14px; overflow-x: auto; padding-bottom: 8px; }
  .tile { display: flex; flex-direction: column; gap: 4px; width: 140px; flex: none; min-width: 0; text-align: left; }
  .tile img, .ph, .cv { width: 100%; aspect-ratio: 1; border-radius: 10px; object-fit: cover; display: grid; place-items: center; background: rgba(190, 240, 225, 0.06); }
  .tile img { box-shadow: 0 8px 20px rgba(0, 0, 0, 0.3); }
  .tn { font-size: 13px; margin-top: 4px; }
  .ts { font-size: 12px; color: var(--faint); }
  .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); gap: 20px 16px; }
  .pl { width: auto; background: none; border: 0; padding: 0; border-radius: 10px; }
  .pl:hover { background: none; }
  .cv { position: relative; overflow: hidden; }
  .fab {
    position: absolute; right: 8px; bottom: 8px; width: 40px; height: 40px; border-radius: 50%;
    display: grid; place-items: center; background: var(--accent); color: var(--ink);
    opacity: 0; transform: translateY(6px); transition: opacity 0.2s, transform 0.2s; box-shadow: 0 6px 16px rgba(0, 0, 0, 0.4);
  }
  .fab :global(svg) { width: 18px; height: 18px; }
  .pl:hover .fab, .pl:focus-visible .fab { opacity: 1; transform: none; }
  .pl:hover img { filter: brightness(0.85); }
  .wrap { flex-wrap: wrap; }
  .dev { padding: 8px 14px; font-size: 13px; }
  .err { color: var(--danger); font-size: 12px; padding-top: 10px; }
</style>
