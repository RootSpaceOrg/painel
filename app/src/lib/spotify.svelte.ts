// Estado do Spotify compartilhado entre o widget e o player: a música atual faz parte da playlist que está tocando?
// Atualiza só quando a faixa muda. As URIs de cada playlist ficam em cache e só são baixadas de novo
// quando o snapshot_id da playlist muda.
import { spotifyApi, spotifyStatus } from './api';

export interface PlaylistCtx { id: string; name: string; own: boolean }
export const spx = $state({
  on: false,
  uri: '',
  playlist: null as PlaylistCtx | null,
  inList: null as boolean | null, // null = não está tocando de uma playlist (ou ainda não sabe)
  adding: false,
  err: '',
});

const cache = new Map<string, { snap: string; uris: Set<string> }>();
let meId = '';

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

let seq = 0;
export async function spRefresh() {
  if (!spx.on) return;
  const my = ++seq; // ignora respostas de uma faixa anterior
  try {
    const pl = await spotifyApi('GET', '/me/player');
    if (my !== seq) return;
    spx.uri = pl?.item?.uri ?? '';
    const ctx = pl?.context;
    if (ctx?.type !== 'playlist' || !spx.uri) { spx.playlist = null; spx.inList = null; return; }
    const id = String(ctx.uri).split(':').pop()!;
    const meta = await spotifyApi('GET', `/playlists/${id}?fields=name,owner(id),collaborative,snapshot_id`);
    if (!meId) meId = (await spotifyApi('GET', '/me'))?.id ?? '';
    const own = meta?.owner?.id === meId || !!meta?.collaborative;
    // a Spotify só deixa ler os itens das playlists do próprio usuário (as outras dão 403)
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
    if (my === seq) spx.err = String(e);
  }
}

export async function addToPlaylist() {
  const p = spx.playlist;
  if (!p || !spx.uri || spx.adding) return;
  spx.adding = true;
  try {
    const r = await spotifyApi('POST', `/playlists/${p.id}/items`, { uris: [spx.uri] });
    const c = cache.get(p.id);
    c?.uris.add(spx.uri);
    if (c && r?.snapshot_id) c.snap = r.snapshot_id;
    spx.inList = true;
    spx.err = '';
  } catch (e) {
    // login antigo não tem permissão de editar playlists (403): precisa conectar de novo
    spx.err = String(e).includes('403') ? 'Reconecte o Spotify em Configurações para permitir adicionar à playlist.' : String(e);
  } finally {
    spx.adding = false;
  }
}
