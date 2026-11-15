<script lang="ts">
  import { ui, prefs, savePrefs, loadWallpaper } from '../lib/state.svelte';
  import * as api from '../lib/api';
  import Icon from './Icon.svelte';
  import { upd, check, install } from '../lib/update.svelte';
  import { spInit } from '../lib/spotify.svelte';

  const COLORS = ['#7fe0d0', '#cfe08e', '#f3a9c4', '#e8c27c', '#8cc8f0', '#c3a6f0', '#7dbb6a'];

  let monitors = $state<api.MonitorInfo[]>([]);
  let win = $state<api.WindowCfg>({ monitor: null, fullscreen: true });
  let feeds = $state<api.Feed[]>([]);
  let nf = $state({ name: '', url: '', color: COLORS[0] });
  let syncing = $state(false);
  let sp = $state<api.SpotifyStatus>({ client_id: '', connected: false });
  let spMsg = $state('');
  let msg = $state('');
  let autostart = $state(false);
  type Tab = 'tela' | 'visual' | 'agendas' | 'spotify' | 'atualizacoes';
  const TABS: { id: Tab; label: string; icon: string }[] = [
    { id: 'tela', label: 'Tela', icon: 'expand' },
    { id: 'visual', label: 'Visual', icon: 'image' },
    { id: 'agendas', label: 'Agendas', icon: 'calendar' },
    { id: 'spotify', label: 'Spotify', icon: 'music' },
    { id: 'atualizacoes', label: 'Atualizações', icon: 'refresh' },
  ];
  let tab = $state<Tab>('tela');

  const loadFeeds = async () => (feeds = await api.feedsList());
  $effect(() => {
    api.monitorsList().then(m => (monitors = m));
    api.windowGet().then(w => (win = w));
    api.spotifyStatus().then(s => (sp = s));
    loadFeeds();
    api.autostartGet().then(v => (autostart = v));
    if (!upd.info) check();
  });

  const say = (e: unknown) => (msg = String(e));
  const monLabel = (m: api.MonitorInfo) =>
    `${m.width}×${m.height}${m.primary ? ' · principal' : ''} — ${m.name.replace(/^[\\.]+/, '')}`;
  const applyWindow = () => api.windowApply(win.monitor, win.fullscreen).catch(say);

  async function pickWallpaper(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (!f) return;
    await api.wallpaperSet(new Uint8Array(await f.arrayBuffer())).catch(say);
    loadWallpaper();
  }
  const clearWallpaper = () => api.wallpaperClear().then(loadWallpaper).catch(say);

  async function addFeed() {
    const url = nf.url.trim();
    if (!url) return;
    if (!/^(https?|webcal):\/\//i.test(url)) { msg = 'O link precisa começar com https:// (ou webcal://).'; return; }
    await api.feedSave({ name: nf.name.trim() || 'Agenda', url: url.replace(/^webcal:/i, 'https:'), color: nf.color }).catch(say);
    nf = { name: '', url: '', color: COLORS[(feeds.length + 1) % COLORS.length] };
    await sync();
  }
  async function sync() {
    syncing = true;
    await api.feedsRefresh().catch(say);
    syncing = false;
    loadFeeds();
  }
  const delFeed = (id: number) => api.feedDelete(id).then(loadFeeds).catch(say);
  const recolor = (f: api.Feed, color: string) => api.feedSave({ id: f.id, name: f.name, url: f.url, color }).then(loadFeeds).catch(say);

  async function spConnect() {
    spMsg = 'Autorize no navegador…';
    try {
      await api.spotifySetClientId(sp.client_id.trim());
      await api.spotifyConnect();
      spMsg = 'Conectado.';
    } catch (e) { spMsg = String(e); }
    sp = await api.spotifyStatus();
    spInit();
  }
  async function spDisconnect() {
    await api.spotifyDisconnect();
    sp = await api.spotifyStatus();
    spInit();
    spMsg = '';
  }
</script>

<div class="overlay" role="presentation" onpointerdown={e => e.target === e.currentTarget && (ui.settings = false)}>
  <div class="panel glass" role="dialog" aria-modal="true" aria-label="Configurações">
    <header>
      <span class="label">Configurações</span>
      {#if msg}<span class="err grow">{msg}</span>{/if}
      <button class="ghost icon" aria-label="Fechar (Esc)" onclick={() => (ui.settings = false)}><Icon name="x" /></button>
    </header>

    <div class="body">
      <nav class="tabs" aria-label="Seções">
        {#each TABS as t}
          <button class="tab" class:on={tab === t.id} aria-current={tab === t.id ? 'page' : undefined} onclick={() => (tab = t.id)}>
            <Icon name={t.icon} /><span>{t.label}</span>
            {#if t.id === 'atualizacoes' && upd.info?.available}<i class="badge" title="Atualização disponível"></i>{/if}
          </button>
        {/each}
      </nav>

      <!-- só rola se o conteúdo da aba passar da altura -->
      <section class="content">
        {#if tab === 'tela'}
          <h3>Tela</h3>
          <label class="f"><span>Monitor</span>
            <select bind:value={win.monitor}>
              {#each monitors as m}<option value={m.name}>{monLabel(m)}</option>{/each}
            </select>
          </label>
          <label class="row"><input type="checkbox" bind:checked={win.fullscreen} /> Tela cheia</label>
          <div><button class="primary" onclick={applyWindow}>Aplicar</button></div>
          <label class="row"><input type="checkbox" bind:checked={autostart} onchange={() => api.autostartSet(autostart).catch(say)} /> Iniciar com o Windows</label>
          <p class="hint">Fechar a janela só esconde; para sair use o ícone na bandeja.</p>

        {:else if tab === 'visual'}
          <h3>Visual</h3>
          <div class="two">
            <div class="col">
              <label class="f"><span>Escurecer fundo · {Math.round(prefs.dim * 100)}%</span>
                <input type="range" min="0" max="0.85" step="0.05" bind:value={prefs.dim} style="--p:{(prefs.dim / 0.85) * 100}" onchange={savePrefs} />
              </label>
              <label class="f"><span>Desfoque dos cards · {prefs.blur}px</span>
                <input type="range" min="0" max="40" step="2" bind:value={prefs.blur} style="--p:{(prefs.blur / 40) * 100}" onchange={savePrefs} />
              </label>
              <label class="row"><input type="checkbox" bind:checked={prefs.seconds} onchange={savePrefs} /> Mostrar segundos no relógio</label>
              <p class="hint">Desfoque em 0 deixa o Painel mais leve.</p>
            </div>
            <div class="col">
              <span class="f"><span>Wallpaper</span></span>
              {#if prefs.wallUrl}<img class="thumb" src={prefs.wallUrl} alt="Wallpaper atual" />{/if}
              <div class="row">
                <label class="file"><Icon name="image" /> Escolher imagem<input type="file" accept="image/*" onchange={pickWallpaper} hidden /></label>
                {#if prefs.wallUrl}<button class="ghost" onclick={clearWallpaper}>Remover</button>{/if}
              </div>
            </div>
          </div>

        {:else if tab === 'agendas'}
          <h3>Agendas (Google, Outlook…)</h3>
          <p class="hint">
            Google Agenda → ⚙ Configurações → clique na agenda → <b>Endereço secreto no formato iCal</b>. Copie e cole aqui.
            Um link por agenda/conta. Só leitura, atualiza a cada 15 min.
          </p>
          <form class="col" onsubmit={e => { e.preventDefault(); addFeed(); }}>
            <div class="row">
              <input class="nm" bind:value={nf.name} placeholder="Nome (ex.: Trabalho)" aria-label="Nome da agenda" />
              <div class="sw">{#each COLORS as c}<button type="button" class="c" class:on={c === nf.color} style="background:{c}" aria-label="Cor {c}" onclick={() => (nf.color = c)}></button>{/each}</div>
            </div>
            <div class="row">
              <input class="grow" bind:value={nf.url} placeholder="https://calendar.google.com/calendar/ical/…/basic.ics" aria-label="Link ICS" />
              <button class="primary" type="submit">Adicionar</button>
            </div>
          </form>
          <div class="row between">
            <span class="label">Suas agendas</span>
            <button type="button" class="ghost" onclick={sync} disabled={syncing}><Icon name="refresh" /> {syncing ? 'Sincronizando…' : 'Sincronizar agora'}</button>
          </div>
          {#each feeds as f (f.id)}
            <div class="feed">
              <div class="sw">
                {#each COLORS as c}<button class="c" class:on={c === f.color} style="background:{c}" aria-label="Cor {c}" onclick={() => recolor(f, c)}></button>{/each}
              </div>
              <div class="grow">
                <div class="ellipsis">{f.name}</div>
                <div class="small" class:err={f.error} class:faint={!f.error}>
                  {f.error ?? (f.last_sync ? `sincronizada ${new Date(f.last_sync).toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' })}` : 'aguardando…')}
                </div>
              </div>
              <button class="ghost icon danger" aria-label="Remover {f.name}" onclick={() => delFeed(f.id)}><Icon name="trash" /></button>
            </div>
          {:else}
            <p class="hint">Nenhuma agenda ainda.</p>
          {/each}

        {:else if tab === 'spotify'}
          <h3>Spotify <span class="faint small">(opcional)</span></h3>
          {#if sp.connected}
            <div class="row"><span class="ok">● Conectado</span><button class="ghost" onclick={spDisconnect}>Desconectar</button></div>
            <p class="hint">Se algum recurso novo pedir permissão (ex.: adicionar à playlist), desconecte e conecte de novo.</p>
          {:else}
            <ol class="hint">
              <li>Abra developer.spotify.com/dashboard → <b>Create app</b>.</li>
              <li>Redirect URI: <code>http://127.0.0.1:43821/callback</code> · marque <b>Web API</b>.</li>
              <li>Copie o <b>Client ID</b> e cole abaixo.</li>
            </ol>
            <div class="row">
              <input class="grow" bind:value={sp.client_id} placeholder="Client ID" aria-label="Spotify Client ID" />
              <button class="primary" onclick={spConnect} disabled={!sp.client_id.trim()}>Conectar</button>
            </div>
          {/if}
          {#if spMsg}<p class="hint">{spMsg}</p>{/if}
          <p class="hint">Sem Spotify, o widget já controla qualquer player do Windows. Com ele: fila, playlists, curtir, adicionar à playlist e trocar dispositivo (controles exigem Premium).</p>

        {:else}
          <h3>Atualizações</h3>
          <p class="hint big">
            Versão {upd.info?.current ?? '…'}.
            {#if upd.checking}Procurando…
            {:else if upd.info?.error}<span class="err">{upd.info.error}</span>
            {:else if upd.info?.available}<b class="ok">Versão {upd.info.latest} disponível.</b>
            {:else if upd.info}Você está na versão mais recente.{/if}
          </p>
          <div class="row">
            <button class="ghost" onclick={check} disabled={upd.checking}><Icon name="refresh" /> Procurar atualizações</button>
            {#if upd.info?.available}
              <button class="primary" onclick={install} disabled={upd.installing}>{upd.installing ? 'Baixando…' : `Atualizar para ${upd.info.latest}`}</button>
            {/if}
          </div>
          {#if upd.error}<p class="err">{upd.error}</p>{/if}
          {#if upd.info?.available && upd.info.notes}<pre class="notes">{upd.info.notes}</pre>{/if}
        {/if}
      </section>
    </div>
  </div>
</div>

<style>
  .overlay { position: fixed; inset: 0; z-index: 60; display: grid; padding: 22px; background: rgba(3, 12, 14, 0.5); }
  .panel { display: flex; flex-direction: column; min-height: 0; padding: 18px 26px 22px; background: var(--glass-strong); }
  header { display: flex; justify-content: space-between; align-items: center; gap: 12px; margin-bottom: 6px; }
  .body { flex: 1; min-height: 0; display: grid; grid-template-columns: 190px minmax(0, 1fr); gap: 28px; }
  .tabs { display: flex; flex-direction: column; gap: 4px; padding-top: 8px; border-right: 1px solid var(--line); padding-right: 16px; }
  .tab { display: flex; align-items: center; gap: 10px; text-align: left; background: transparent; border-color: transparent; color: var(--muted); position: relative; }
  .tab :global(svg) { width: 16px; height: 16px; }
  .tab.on { background: rgba(127, 224, 208, 0.12); color: var(--teal); border-color: rgba(127, 224, 208, 0.25); }
  .badge { position: absolute; right: 10px; width: 8px; height: 8px; border-radius: 50%; background: var(--teal); box-shadow: 0 0 8px var(--teal); }
  .content { min-height: 0; overflow-y: auto; padding-right: 6px; max-width: 760px; }
  .two { display: grid; grid-template-columns: 1fr 1fr; gap: 32px; align-items: start; }
  .between { justify-content: space-between; margin-top: 6px; }
  .big { font-size: 14px; }
  .notes { white-space: pre-wrap; font: 12px/1.5 var(--sans); color: var(--muted); background: rgba(0, 0, 0, 0.2); border-radius: 10px; padding: 12px; user-select: text; }
  section { display: flex; flex-direction: column; gap: 10px; }
  h3 { font-size: 17px; font-weight: 600; margin: 6px 0 4px; color: var(--leaf); }
  .f { display: flex; flex-direction: column; gap: 6px; font-size: 13px; color: var(--muted); }
  label.row { font-size: 13px; cursor: pointer; }
  .hint { font-size: 12px; color: var(--faint); margin: 0; line-height: 1.5; user-select: text; }
  ol.hint { padding-left: 18px; }
  code { color: var(--teal); user-select: all; }
  .file { display: inline-flex; gap: 8px; align-items: center; cursor: pointer; padding: 6px 12px; border: 1px solid var(--line); border-radius: 10px; }
  .file:hover { border-color: var(--line-strong); }
  .file :global(svg) { width: 16px; height: 16px; }
  .thumb { width: 100%; max-width: 320px; aspect-ratio: 16/9; object-fit: cover; border-radius: 10px; border: 1px solid var(--line); }
  .feed { display: flex; align-items: center; gap: 12px; padding: 8px 0; border-bottom: 1px solid var(--line); }
  .sw { display: flex; gap: 4px; }
  .c { width: 16px; height: 16px; padding: 0; border-radius: 50%; border: 2px solid transparent; }
  .c.on { border-color: var(--text); }
  .nm { flex: 1; }
  .small { font-size: 12px; }
  .err { color: var(--danger); font-size: 12px; }
  .ok { color: var(--leaf); }
  button :global(svg) { vertical-align: -3px; }
</style>
