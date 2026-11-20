// Volume do que está tocando, compartilhado entre o widget "Tocando agora" e o player em tela cheia.
// Ordem de preferência:
//  1. Spotify conectado e tocando → a barra de volume do próprio Spotify (Web API; exige Premium)
//  2. sessão de áudio do app que está tocando no mixer do Windows (ex.: YouTube no navegador)
//  3. volume geral
import { audioState, audioSetMaster, audioSetSession } from './api';
import { live } from './state.svelte';
import { spx, blocked, setSpotifyVolume } from './spotify.svelte';
import { sameApp } from './util';
import { iconFor } from './icons';

export const av = $state({
  vol: null as number | null,
  target: null as { pids: number[]; name: string; exe: string } | null,
  spVol: false, // controlando o volume interno do Spotify
  icon: '', // ícone do app (data URL) para mostrar no lugar do alto-falante
});

let spOff = false; // Spotify recusou (ex.: sem Premium): usa o mixer do Windows até reiniciar
let dragging = false; // não sobrescreve o slider enquanto arrasta
export const setDragging = (b: boolean) => { dragging = b; };

let iconExe = '';
function updateIcon(exe: string) {
  if (exe === iconExe) return;
  iconExe = exe;
  av.icon = '';
  if (exe) iconFor(exe).then(u => { if (exe === iconExe) av.icon = u; });
}

export async function loadVol() {
  if (dragging) return;
  const id = live.media?.active ?? '';
  try {
    const a = await audioState();
    const hits = id ? a.sessions.filter(x => x.pid !== 0 && sameApp(id, x.name)) : [];
    av.target = hits.length ? { pids: hits.map(x => x.pid), name: hits[0].name, exe: hits[0].exe } : null;
    updateIcon(av.target?.exe ?? '');
    // volume do app Spotify vem do estado do Spotify (consulta por faixa + a cada 60 s), sem chamar a API aqui
    if (spx.on && !spOff && !blocked() && spx.vol !== null && sameApp(id, 'Spotify')) {
      if (!dragging && spPending === null) av.vol = spx.vol;
      av.spVol = true;
      return;
    }
    av.spVol = false;
    av.vol = hits.length ? hits[0].volume : a.master;
  } catch { /* sem áudio: mantém o último valor */ }
}

// uma leitura periódica só, não importa quantos controles estão na tela
let users = 0, timer = 0;
export function useVolume() {
  if (users++ === 0) {
    loadVol();
    timer = window.setInterval(loadVol, 4000);
  }
  return () => { if (--users === 0) clearInterval(timer); };
}

// Web API tem limite de chamadas: no máximo 1 a cada 250ms enquanto arrasta, sempre mandando o último valor
let spTimer = 0, spPending: number | null = null;
function sendSpotifyVol() {
  if (spPending === null) { spTimer = 0; return; }
  const pct = spPending;
  spPending = null;
  setSpotifyVolume(pct / 100).catch(e => {
    // recusou (ex.: sem Premium): mixer do Windows até reiniciar; bloqueio (429) volta sozinho quando passar
    if (!String(e).startsWith('429')) spOff = true;
    av.spVol = false;
    if (av.vol !== null) setVol(av.vol);
  });
  spTimer = window.setTimeout(sendSpotifyVol, 250);
}

export function setVol(v: number) {
  av.vol = v;
  if (av.spVol) {
    spPending = Math.round(v * 100);
    if (!spTimer) sendSpotifyVol();
  } else if (av.target) for (const pid of av.target.pids) audioSetSession(pid, v).catch(() => {});
  else audioSetMaster(v).catch(() => {});
}

export const volLabel = () => (av.spVol ? 'Volume do Spotify' : av.target ? `Volume do ${av.target.name}` : 'Volume geral');
