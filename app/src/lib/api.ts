// Contrato UI <-> backend Rust (src-tauri). Tudo que a UI pode pedir passa por aqui.
// Rust usa snake_case nos argumentos; o Tauri converte camelCase das chaves de invoke automaticamente,
// então argumentos aqui são sempre de uma palavra para evitar ambiguidade.
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// ---------- Mídia (SMTC: qualquer player do Windows) ----------
export interface MediaSession { id: string; app: string } // id = SourceAppUserModelId
export interface MediaState {
  sessions: MediaSession[];
  active: string | null;   // sessão exibida (null = nenhuma)
  title: string; artist: string; album: string;
  playing: boolean;
  position: number;        // segundos, já extrapolado no momento do evento
  duration: number;        // segundos (0 = desconhecido)
  art_key: string;         // muda quando a capa muda ('' = sem capa)
}
export const onMedia = (cb: (s: MediaState) => void): Promise<UnlistenFn> =>
  listen<MediaState>('media', e => cb(e.payload)); // ~1x/s
export const mediaArt = () => invoke<string>('media_art'); // data URL ou ''
export const mediaControl = (action: 'toggle' | 'next' | 'prev' | 'seek', value?: number) =>
  invoke<void>('media_control', { action, value: value ?? null });
export const mediaSelect = (id: string | null) => invoke<void>('media_select', { id });

// ---------- Hardware ----------
export interface GpuSample { name: string; util: number; temp: number; vram_used: number; vram_total: number; power_w: number | null }
export interface DiskSample { name: string; mount: string; used: number; total: number }
export interface SysSample {
  cpu: number; cores: number[];          // %
  ram_used: number; ram_total: number;   // bytes
  gpu: GpuSample | null;
  net_rx: number; net_tx: number;        // bytes/s
  disks: DiskSample[];
}
export const onSys = (cb: (s: SysSample) => void) => listen<SysSample>('sys', e => cb(e.payload)); // a cada 2s

// ---------- Áudio ----------
export interface AudioDevice { id: string; name: string; default: boolean }
export interface AudioSession { pid: number; name: string; exe: string; volume: number; muted: boolean } // exe '' = sem ícone
export interface AudioState { devices: AudioDevice[]; master: number; muted: boolean; sessions: AudioSession[] }
export const audioState = () => invoke<AudioState>('audio_state');
export const audioSetMaster = (volume: number) => invoke<void>('audio_set_master', { volume });
export const audioSetMasterMute = (muted: boolean) => invoke<void>('audio_set_master_mute', { muted });
export const audioSetSession = (pid: number, volume: number) => invoke<void>('audio_set_session', { pid, volume });
export const audioSetSessionMute = (pid: number, muted: boolean) => invoke<void>('audio_set_session_mute', { pid, muted });
export interface AppIcon { w: number; h: number; rgba: string } // pixels RGBA em base64
export const appIcon = (path: string) => invoke<AppIcon | null>('app_icon', { path });
export const audioSetDefault = (id: string) => invoke<void>('audio_set_default', { id });

// ---------- Bluetooth (fones/caixas pareados) ----------
export interface BtDevice { name: string; connected: boolean }
export const btList = () => invoke<BtDevice[]>('bt_list');
export const btSet = (name: string, connect: boolean) => invoke<void>('bt_set', { name, connect });

// ---------- To-do ----------
// days: bitmask seg=1, ter=2, qua=4, qui=8, sex=16, sáb=32, dom=64. 0 = tarefa avulsa.
// done_on: 'YYYY-MM-DD' do dia em que foi concluída (recorrente: concluída hoje se done_on == hoje).
export interface Todo { id: number; title: string; days: number; done_on: string | null }
export const todoList = () => invoke<Todo[]>('todo_list');
export const todoSave = (todo: Omit<Todo, 'id'> & { id?: number }) => invoke<number>('todo_save', { todo }); // upsert, retorna id
export const todoDelete = (id: number) => invoke<void>('todo_delete', { id });

// ---------- Metas diárias ----------
export interface Habit { id: number; name: string; icon: string; days: number; log: string[] } // log: dias 'YYYY-MM-DD' concluídos (últimos 400)
export const habitList = () => invoke<Habit[]>('habit_list');
export const habitSave = (habit: { id?: number; name: string; icon: string; days: number }) => invoke<number>('habit_save', { habit });
export const habitDelete = (id: number) => invoke<void>('habit_delete', { id });
export const habitMark = (id: number, day: string, done: boolean) => invoke<void>('habit_mark', { id, day, done });

// ---------- Agenda ----------
export interface CalEvent {
  id: string;            // local: 'l:<id>'; feed: 'f<feed>:<uid>:<start>'
  local_id: number | null;
  title: string; location: string; notes: string;
  start: number; end: number; // epoch ms
  all_day: boolean;      // dia inteiro: start = 00:00 local do dia, end = 00:00 do dia seguinte ao último
  color: string;         // cor do feed ('' para eventos locais)
  feed: number | null;
  url: string;           // abre no navegador (Google Agenda ou URL do ICS); '' = sem link
}
export interface Feed { id: number; name: string; url: string; color: string; error: string | null; last_sync: number | null }
export const openUrl = (url: string) => invoke<void>('open_url', { url });
export const agenda = (from: number, to: number) => invoke<CalEvent[]>('agenda', { from, to }); // ordenado por start
export const eventSave = (event: { id?: number; title: string; start: number; end: number; all_day: boolean; notes: string; location: string }) =>
  invoke<number>('event_save', { event });
export const eventDelete = (id: number) => invoke<void>('event_delete', { id });
export const feedsList = () => invoke<Feed[]>('feeds_list');
export const feedSave = (feed: { id?: number; name: string; url: string; color: string }) => invoke<number>('feed_save', { feed });
export const feedDelete = (id: number) => invoke<void>('feed_delete', { id });
export const feedsRefresh = () => invoke<void>('feeds_refresh'); // baixa todos, resolve ao terminar
export const onAgendaChanged = (cb: () => void) => listen('agenda-changed', () => cb()); // após sync automático (15 min)

// ---------- Configurações genéricas (JSON livre da UI: 'layout', 'ui') ----------
export const kvGet = (key: string) => invoke<string | null>('kv_get', { key });
export const kvSet = (key: string, value: string) => invoke<void>('kv_set', { key, value });

// ---------- Janela / monitor ----------
export interface MonitorInfo { name: string; width: number; height: number; x: number; y: number; primary: boolean }
export interface WindowCfg { monitor: string | null; fullscreen: boolean }
export const monitorsList = () => invoke<MonitorInfo[]>('monitors_list');
export const windowGet = () => invoke<WindowCfg>('window_get');
export const windowApply = (monitor: string | null, fullscreen: boolean) => invoke<void>('window_apply', { monitor, fullscreen });

// ---------- Letras (LRCLIB) ----------
export interface Lyrics { syncedLyrics: string | null; plainLyrics: string | null; instrumental: boolean }
export const lyrics = (artist: string, title: string, album: string, duration: number) =>
  invoke<Lyrics | null>('lyrics', { artist, title, album, duration });

// ---------- Atualização (releases de RootSpaceOrg/painel) ----------
export interface UpdateInfo { current: string; latest: string | null; available: boolean; notes: string; url: string | null; error: string | null }
export const updateCheck = () => invoke<UpdateInfo>('update_check');
export const updateInstall = (url: string) => invoke<void>('update_install', { url }); // baixa, fecha, instala e reabre

// ---------- Iniciar com o Windows ----------
export const autostartGet = () => invoke<boolean>('autostart_get');
export const autostartSet = (on: boolean) => invoke<void>('autostart_set', { on });

// ---------- Wallpaper (bytes crus, salvo na pasta de dados) ----------
export const wallpaperSet = (bytes: Uint8Array) => invoke<void>('wallpaper_set', bytes);
export const wallpaperGet = () => invoke<ArrayBuffer>('wallpaper_get'); // vazio = sem wallpaper
export const wallpaperClear = () => invoke<void>('wallpaper_clear');

// ---------- Spotify (opcional, PKCE com Client ID do usuário) ----------
export interface SpotifyStatus { client_id: string; connected: boolean }
export const spotifyStatus = () => invoke<SpotifyStatus>('spotify_status');
export const spotifySetClientId = (id: string) => invoke<void>('spotify_set_client_id', { id });
export const spotifyConnect = () => invoke<void>('spotify_connect'); // abre navegador, resolve quando autorizado
export const spotifyDisconnect = () => invoke<void>('spotify_disconnect');
// Proxy autenticado da Web API: path relativo a https://api.spotify.com/v1 (ex.: '/me/player/queue').
// Retorna o JSON (ou null em 204). Erro = string com status e mensagem.
export const spotifyApi = <T = any>(method: 'GET' | 'PUT' | 'POST' | 'DELETE', path: string, body?: unknown) =>
  invoke<T>('spotify_api', { method, path, body: body ?? null });
