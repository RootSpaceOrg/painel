// Estado do Spotify compartilhado entre o widget e o player: faixa atual, curtida, volume do app e
// se a música está na playlist que está tocando.
// A Web API tem limite de chamadas (passar dele bloqueia o app por horas): por isso tudo vem de uma
// consulta por troca de faixa + uma leve a cada 60 s, e nada é chamado durante um bloqueio (429).
import { spotifyApi, spotifyStatus } from './api';

export interface PlaylistCtx { id: string; name: string; own: boolean }
export const spx = $state({
  on: false,
  uri: '',
  liked: null as boolean | null,
  vol: null as number | null, // volume do app Spotify (0–1); null = dispositivo não informa
  playlist: null as PlaylistCtx | null,
  inList: null as boolean | null, // null = não está tocando de uma playlist sua (ou ainda não sabe)
  adding: false,
  blockedUntil: 0, // ms; Spotify mandou esperar (excesso de chamadas)
  err: '',
});

const MIN = 60_000;
const cache = new Map<string, { snap: string; uris: Set<string> }>(); // URIs por playlist (por snapshot_id)
const metaCache = new Map<string, { at: number; meta: any }>(); // nome/dono da playlist, por 10 min
let meId = '';

/** Erro "429: retry-after=N" → registra o bloqueio e limpa o que dependia da API. */
function fail(e: unknown) {
  const s = String(e);
  const m = s.match(/^429.*?retry-after=(\d+)/);
  if (m) spx.blockedUntil = Date.now() + +m[1] * 1000;
  spx.playlist = null;
  spx.inList = null;
  spx.liked = null;
  spx.vol = null;
  spx.err = m ? '' : s;
}
export const blocked = () => Date.now() < spx.blockedUntil;

export async function spInit() {
  spx.on = !!(await spotifyStatus().catch(() => null))?.connected;
  if (spx.on) spRefresh();
}

// /playlists/{id}/tracks foi desativado (403); /items devolve { items: [{ item: { uri } }], next }
async function playlistUris(id: string) {
  const uris = new Set<string>();
  let path: string | null = `/playlists/${id}/items?limit=100&fields=next,items(item(uri))`;
  for (let i = 0; path && i < 50; i++) { // ponytail: até 5 mil faixas por playlist
    const page: any = await spotifyApi('GET', path);
    for (const it of page?.items ?? []) if (it?.item?.uri) uris.add(it.item.uri);
    path = page?.next ? String(page.next).replace('https://api.spotify.com/v1', '') : null;
  }
  return uris;
}

async function playlistMeta(id: string) {
  const c = metaCache.get(id);
  if (c && Date.now() - c.at < 10 * MIN) return c.meta;
  const meta = await spotifyApi('GET', `/playlists/${id}?fields=name,owner(id),collaborative,snapshot_id`);
  metaCache.set(id, { at: Date.now(), meta });
  return meta;
}

let seq = 0;
let lastCtx = '';
/** Consulta completa (troca de faixa): faixa, volume, curtida e playlist. */
export async function spRefresh() {
  if (!spx.on || blocked()) return;
  const my = ++seq; // ignora respostas de uma faixa anterior
  try {
    const pl = await spotifyApi('GET', '/me/player');
    if (my !== seq) return;
    applyPlayer(pl);
    spx.liked = spx.uri ? ((await spotifyApi<boolean[]>('GET', `/me/library/contains?uris=${spx.uri}`))?.[0] ?? null) : null;
    const ctx = pl?.context;
    lastCtx = ctx?.uri ?? '';
    // só playlist sua (as de outras pessoas a Spotify não deixa ler): fora disso, sem chip
    if (ctx?.type !== 'playlist' || !spx.uri) { spx.playlist = null; spx.inList = null; return; }
    const id = String(ctx.uri).split(':').pop()!;
    const meta = await playlistMeta(id);
    if (!meId) meId = (await spotifyApi('GET', '/me'))?.id ?? '';
    const own = meta?.owner?.id === meId || !!meta?.collaborative;
    if (!own) { spx.playlist = null; spx.inList = null; return; }
    let c = cache.get(id);
    if (!c || c.snap !== meta?.snapshot_id) {
      c = { snap: meta?.snapshot_id ?? '', uris: await playlistUris(id) };
      cache.set(id, c);
    }
    if (my !== seq) return;
    spx.playlist = { id, name: meta?.name ?? 'playlist', own };
    spx.inList = c.uris.has(spx.uri);
    spx.err = '';
  } catch (e) {
    if (my === seq) fail(e);
  }
}

function applyPlayer(pl: any) {
  spx.uri = pl?.item?.uri ?? '';
  const dev = pl?.device;
  spx.vol = dev?.supports_volume && typeof dev.volume_percent === 'number' ? dev.volume_percent / 100 : null;
  if (!pl) { spx.playlist = null; spx.inList = null; spx.liked = null; } // nada tocando no Spotify
}

/** Consulta leve (a cada 60 s): mantém o volume em sincronia; se o contexto mudou, faz a completa. */
async function poll() {
  if (!spx.on || blocked()) return;
  try {
    const pl = await spotifyApi('GET', '/me/player');
    const ctx = pl?.context?.uri ?? '';
    if ((pl?.item?.uri ?? '') !== spx.uri || ctx !== lastCtx) return spRefresh();
    applyPlayer(pl);
  } catch (e) { fail(e); }
}
let pollers = 0, pollTimer = 0;
/** Liga a consulta leve enquanto houver alguém usando (widget/player); devolve a função de desligar. */
export function useSpotifyPoll() {
  if (pollers++ === 0) pollTimer = window.setInterval(poll, MIN);
  return () => { if (--pollers === 0) clearInterval(pollTimer); };
}

export async function toggleLike() {
  if (!spx.uri || spx.liked === null || blocked()) return;
  const want = !spx.liked;
  spx.liked = want; // otimista
  try { await spotifyApi(want ? 'PUT' : 'DELETE', `/me/library?uris=${spx.uri}`); }
  catch (e) {
    spx.liked = !want;
    const s = String(e);
    if (s.startsWith('429')) fail(e);
    else spx.err = s.includes('403') ? 'Reconecte o Spotify em Configurações para permitir curtir.' : s;
  }
}

/** Volume do app Spotify (Web API). Chamado com no máximo 1 envio a cada 250 ms pelo controle de volume. */
export async function setSpotifyVolume(v: number) {
  spx.vol = v;
  try { await spotifyApi('PUT', `/me/player/volume?volume_percent=${Math.round(v * 100)}`); }
  catch (e) { if (String(e).startsWith('429')) fail(e); throw e; }
}

export async function addToPlaylist() {
  const p = spx.playlist;
  if (!p || !spx.uri || spx.adding || blocked()) return;
  spx.adding = true;
  try {
    const r = await spotifyApi('POST', `/playlists/${p.id}/items`, { uris: [spx.uri] });
    const c = cache.get(p.id);
    c?.uris.add(spx.uri);
    if (c && r?.snapshot_id) c.snap = r.snapshot_id;
    spx.inList = true;
    spx.err = '';
  } catch (e) {
    if (String(e).startsWith('429')) fail(e);
    // login antigo não tem permissão de editar playlists (403): precisa conectar de novo
    else spx.err = String(e).includes('403') ? 'Reconecte o Spotify em Configurações para permitir adicionar à playlist.' : String(e);
  } finally {
    spx.adding = false;
  }
}
