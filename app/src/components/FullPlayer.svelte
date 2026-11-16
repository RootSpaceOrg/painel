<script lang="ts">
  // Player em tela cheia. Topo: fundo com foto do artista (ou capa) em blur escuro, título, letra opcional à direita
  // e uma barra de ponta a ponta embaixo. Rolando: fila, dispositivos, playlists.
  import { live, prefs, savePrefs } from '../lib/state.svelte';
  import { mediaControl, mediaSelect, spotifyStatus, spotifyApi, audioState, audioSetMaster, audioSetSession } from '../lib/api';
  import { iconFor } from '../lib/icons';
  import { fmtDuration, fmtTime, fmtWeekday, sameApp } from '../lib/util';
  import Icon from './Icon.svelte';
  import WaveBar from './WaveBar.svelte';
  import Lyrics from './Lyrics.svelte';
  import PlaylistChip from './PlaylistChip.svelte';

  const m = $derived(live.media!);
  const accent = $derived(live.artColor || 'var(--teal)');
  const source = $derived(m.sessions.find(s => s.id === m.active)?.app ?? '');
  let showLyrics = $state(false);
  let bgCtl = $state(false); // popover do desfoque do fundo

  // horário no canto superior direito (atualiza a cada 10s; só os minutos aparecem)
  let now = $state(new Date());
  $effect(() => {
    const t = setInterval(() => (now = new Date()), 10_000);
    return () => clearInterval(t);
  });
  let scroller: HTMLDivElement;

  // ---------- Spotify ----------
  type Img = { url: string }[];
  let sp = $state({
    on: false, err: '', liked: null as boolean | null, uri: '', shuffle: false, repeat: 'off', bg: '', artistId: '',
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
    // fundo: foto do artista > capa do álbum em alta > capa do player (SMTC)
    const artistId = pl?.item?.artists?.[0]?.id ?? '';
    const albumImg = pl?.item?.album?.images?.[0]?.url ?? '';
    if (artistId !== sp.artistId || !sp.bg) {
      sp.artistId = artistId;
      const art = artistId ? await safe(spotifyApi('GET', `/artists/${artistId}`), null) : null;
      sp.bg = art?.images?.[0]?.url || albumImg;
    }
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
  const bg = $derived(sp.bg || live.art);

  // volume geral do sistema (mesmo do widget Volume)
  // Volume do app que está tocando (ex.: só o Spotify), pela sessão de áudio dele no mixer do Windows.
  // Sem sessão de áudio desse app (ou app não reconhecido), controla o volume geral.
  let vol = $state<number | null>(null);
  let target = $state<{ pids: number[]; name: string; exe: string } | null>(null);
  let dragging = false; // não sobrescreve o slider enquanto arrasta
  const activeId = $derived(live.media?.active ?? '');
  async function loadVol() {
    if (dragging) return;
    try {
      const a = await audioState();
      const hits = activeId ? a.sessions.filter(x => x.pid !== 0 && sameApp(activeId, x.name)) : [];
      if (hits.length) {
        target = { pids: hits.map(x => x.pid), name: hits[0].name, exe: hits[0].exe };
        vol = hits[0].volume;
      } else {
        target = null;
        vol = a.master;
      }
    } catch { /* sem áudio: mantém o último valor */ }
  }
  $effect(() => {
    activeId; // troca de fonte → reavalia na hora
    loadVol();
    const t = setInterval(loadVol, 3000);
    return () => clearInterval(t);
  });
  const setVol = (v: number) => {
    vol = v;
    if (target) for (const pid of target.pids) audioSetSession(pid, v).catch(() => {});
    else audioSetMaster(v).catch(() => {});
  };
  // ícone do app no lugar do alto-falante (só muda quando o app muda)
  const targetExe = $derived(target?.exe ?? '');
  let volIcon = $state('');
  $effect(() => {
    const exe = targetExe;
    volIcon = '';
    if (exe) iconFor(exe).then(u => { if (exe === targetExe) volIcon = u; });
  });

  const nice = (e: string) => (e.includes('403') ? 'A Spotify recusou um comando (alguns exigem Premium).' : e);
</script>

<div class="fp" bind:this={scroller} style="--accent:{accent}">
  <section class="hero" class:lyr={showLyrics}>
    {#if bg}
      {#key bg}<div class="bg" style="background-image:url({bg});--art-blur:{prefs.artBlur}px"></div>{/key}
    {/if}
    <div class="shade"></div>
    <div class="clock" aria-label="Horário">
      <span class="mono hm">{fmtTime.format(now)}</span>
      <span class="wd">{fmtWeekday.format(now)}</span>
    </div>

    <div class="stage">
      <div class="meta">
        <div class="top row">
          {#if source}<span class="label">Tocando no {source}</span>{/if}
          {#each m.sessions.length > 1 ? m.sessions : [] as s}
            <button class="chip" class:on={s.id === m.active} onclick={() => mediaSelect(s.id)}>{s.app}</button>
          {/each}
        </div>
        <h1>{m.title}</h1>
        <p class="artist">{m.artist}</p>
        {#if m.album}<p class="album">{m.album}</p>{/if}
        <div class="plc"><PlaylistChip big /></div>
      </div>
      {#if showLyrics}
        <div class="lyrics">
          <Lyrics artist={m.artist} title={m.title} album={m.album} duration={m.duration} position={m.position} playing={m.playing} />
        </div>
      {/if}
    </div>

    <!-- barra de ponta a ponta -->
    <div class="bar">
      <div class="now">
        {#if live.art}<img class="thumb" src={live.art} alt="" />{/if}
        <div class="nt">
          <span class="ellipsis">{m.title}</span>
          <span class="ellipsis faint">{m.artist}</span>
        </div>
        {#if sp.on && sp.uri}
          <button class="ghost icon" class:liked={sp.liked} aria-label={sp.liked ? 'Remover das curtidas' : 'Curtir'} onclick={toggleLike}>
            <Icon name="heart" filled={!!sp.liked} />
          </button>
        {/if}
      </div>

      <div class="center">
        <div class="track">
          <span class="mono t">{fmtDuration(m.position)}</span>
          {#if m.duration}
            <WaveBar thick position={m.position} duration={m.duration} playing={m.playing} color={accent} onseek={s => mediaControl('seek', s)} />
          {:else}<span class="grow"></span>{/if}
          <span class="mono t">{m.duration ? fmtDuration(m.duration) : ''}</span>
        </div>
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

      <div class="side-r">
        <div class="bgctl">
          <button class="chip" class:on={bgCtl} aria-expanded={bgCtl} onclick={() => (bgCtl = !bgCtl)}>
            <Icon name="image" /> Fundo
          </button>
          {#if bgCtl}
            <div class="pop glass">
              <label class="f"><span>Desfoque do fundo · {prefs.artBlur}px</span>
                <input type="range" min="0" max="40" step="1" bind:value={prefs.artBlur} style="--p:{(prefs.artBlur / 40) * 100}"
                  onchange={savePrefs} aria-label="Desfoque do fundo" />
              </label>
            </div>
          {/if}
        </div>
        <button class="chip" class:on={showLyrics} aria-pressed={showLyrics} onclick={() => (showLyrics = !showLyrics)}>
          <Icon name="lyrics" /> Letra
        </button>
        {#if vol !== null}
          <div class="vol" title={target ? `Volume do ${target.name} (só este app)` : 'Volume geral'}>
            {#if volIcon}<img class="vapp" src={volIcon} alt="" />{:else}<Icon name="volume" />{/if}
            <input type="range" min="0" max="100" value={Math.round(vol * 100)} style="--p:{vol * 100}"
              aria-label={target ? `Volume do ${target.name}` : 'Volume geral'}
              onpointerdown={() => (dragging = true)} onpointerup={() => (dragging = false)}
              oninput={e => setVol(+e.currentTarget.value / 100)} />
          </div>
        {/if}
      </div>
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
  .fp { height: 100%; overflow-y: auto; overflow-x: hidden; scroll-behavior: smooth; }

  /* ---------- herói ---------- */
  .hero { position: relative; isolation: isolate; height: 100%; min-height: 460px; display: flex; flex-direction: column; overflow: hidden; }
  .bg {
    position: absolute; inset: -30px; z-index: -2; background-size: cover; background-position: center 25%;
    filter: blur(var(--art-blur, 14px)) brightness(0.5) saturate(1.15); transform: scale(1.04);
    animation: fadein 0.8s ease-out;
  }
  @keyframes fadein { from { opacity: 0; } }
  /* escurece à esquerda (texto) e embaixo (barra), como na referência */
  .shade {
    position: absolute; inset: 0; z-index: -1; pointer-events: none;
    background:
      linear-gradient(90deg, rgba(5, 14, 16, 0.85) 0%, rgba(5, 14, 16, 0.35) 45%, rgba(5, 14, 16, 0.15) 100%),
      linear-gradient(0deg, rgba(5, 14, 16, 0.92) 0%, rgba(5, 14, 16, 0.5) 22%, transparent 45%);
  }
  .stage { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr); gap: 40px; padding: 40px 48px 8px; }
  .lyr .stage { grid-template-columns: 1fr 1fr; }
  .meta { display: flex; flex-direction: column; justify-content: center; min-width: 0; max-width: 640px; }
  .top { flex-wrap: wrap; gap: 6px; margin-bottom: 14px; }
  h1 {
    font-size: clamp(34px, 5vw, 68px); font-weight: 300; line-height: 1.02; margin: 0 0 12px; letter-spacing: -0.015em;
    text-shadow: 0 4px 30px rgba(0, 0, 0, 0.5);
    display: -webkit-box; -webkit-line-clamp: 3; line-clamp: 3; -webkit-box-orient: vertical; overflow: hidden;
  }
  .lyr h1 { font-size: clamp(28px, 3.4vw, 46px); }
  .artist { font-size: 20px; margin: 0; color: var(--accent); }
  .album { margin: 6px 0 0; font-size: 14px; color: var(--muted); }
  .lyrics { min-height: 0; }
  .plc { margin-top: 14px; display: flex; }

  .bar {
    display: grid; grid-template-columns: minmax(200px, 1fr) minmax(0, 2.2fr) minmax(200px, 1fr); align-items: center; gap: 28px;
    padding: 16px 48px 34px;
  }
  .now { display: flex; align-items: center; gap: 12px; min-width: 0; }
  .thumb { width: 60px; height: 60px; border-radius: 10px; object-fit: cover; flex: none; box-shadow: 0 8px 22px rgba(0, 0, 0, 0.45); }
  .nt { display: flex; flex-direction: column; min-width: 0; font-size: 14px; }
  .now .icon :global(svg) { width: 20px; height: 20px; }
  .center { display: flex; flex-direction: column; gap: 6px; min-width: 0; }
  .track { display: flex; align-items: center; gap: 14px; }
  .t { font-size: 12px; color: var(--muted); min-width: 36px; text-align: center; }
  .ctrl { display: flex; align-items: center; justify-content: center; gap: 12px; }
  .ctrl .icon { width: 42px; height: 42px; }
  .ctrl :global(svg) { width: 20px; height: 20px; }
  .side { color: var(--muted); position: relative; }
  .side.act { color: var(--accent); }
  .side.act::after { content: ''; position: absolute; bottom: 3px; left: 50%; width: 4px; height: 4px; border-radius: 50%; background: var(--accent); translate: -50% 0; }
  .one { position: absolute; top: 6px; right: 6px; font-size: 9px; font-weight: 700; }
  .play {
    width: 56px !important; height: 56px !important; border: 0; border-radius: 16px; background: var(--accent); color: var(--ink);
    box-shadow: 0 6px 22px color-mix(in srgb, var(--accent) 45%, transparent);
    transition: border-radius 0.35s cubic-bezier(0.3, 0.7, 0.2, 1), transform 0.15s, filter 0.15s;
  }
  .play:hover { background: var(--accent); filter: brightness(1.08); }
  .play.round { border-radius: 50%; }
  .ctrl button:active { transform: scale(0.9); }
  .liked { color: var(--blossom); }
  .side-r { display: flex; align-items: center; justify-content: flex-end; gap: 10px; }
  .bgctl { position: relative; }
  .pop {
    position: absolute; bottom: calc(100% + 12px); right: 0; width: 240px; padding: 14px 16px; z-index: 5;
    background: var(--glass-strong);
  }
  .f { display: flex; flex-direction: column; gap: 8px; font-size: 12px; color: var(--muted); }
  .vol { display: flex; align-items: center; gap: 8px; color: var(--muted); width: 140px; }
  .vol :global(svg) { width: 18px; height: 18px; flex: none; }
  .vol input { flex: 1; }
  .vapp { width: 20px; height: 20px; flex: none; border-radius: 4px; }
  .chip { display: inline-flex; align-items: center; gap: 6px; padding: 5px 12px; border-radius: 999px; font-size: 12px; background: rgba(0, 0, 0, 0.25); }
  .chip :global(svg) { width: 14px; height: 14px; }
  .chip.on { color: var(--ink); background: var(--accent); border-color: transparent; }
  /* canto superior esquerdo, alinhado ao conteúdo */
  .clock {
    position: absolute; top: 22px; left: 48px; z-index: 2; pointer-events: none;
    display: flex; flex-direction: column; align-items: flex-start; line-height: 1;
    text-shadow: 0 2px 16px rgba(0, 0, 0, 0.5);
  }
  .hm { font-size: 34px; font-weight: 300; letter-spacing: -0.01em; }
  .wd { font-size: 12px; color: var(--muted); text-transform: capitalize; margin-top: 6px; }
  .more {
    position: absolute; left: 50%; bottom: 6px; translate: -50% 0; display: inline-flex; align-items: center; gap: 6px;
    background: transparent; border: 0; color: var(--faint); font-size: 11px; letter-spacing: 0.08em; text-transform: uppercase;
  }
  .more:hover { color: var(--text); background: transparent; }
  .more :global(svg) { width: 14px; height: 14px; animation: bob 2s ease-in-out infinite; }
  @keyframes bob { 50% { transform: translateY(3px); } }
  @media (prefers-reduced-motion: reduce) { .bg, .more :global(svg) { animation: none; } }

  /* ---------- seções ---------- */
  .sec { padding: 26px 48px 8px; }
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
  .err { color: var(--danger); font-size: 12px; padding: 10px 48px 0; }
</style>
