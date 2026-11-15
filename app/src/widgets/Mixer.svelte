<script lang="ts">
  import { audioState, audioSetMaster, audioSetMasterMute, audioSetSession, audioSetSessionMute, audioSetDefault, type AudioState } from '../lib/api';
  import Icon from '../components/Icon.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  let st = $state<AudioState | null>(null);
  let err = $state('');
  let busy = false; // enquanto arrasta, o polling não sobrescreve o slider

  const load = async () => {
    if (busy) return;
    try { st = await audioState(); err = ''; } catch (e) { err = String(e); }
  };
  $effect(() => {
    load();
    const t = setInterval(load, 2000);
    return () => clearInterval(t);
  });

  const run = (p: Promise<unknown>) => p.catch(e => (err = String(e)));
  const pct = (v: number) => Math.round(v * 100);
  const def = $derived(st?.devices.find(d => d.default));
</script>

{#snippet slider(value: number, muted: boolean, label: string, set: (v: number) => void, mute: () => void)}
  <div class="vol" class:muted>
    <button class="ghost icon" aria-label={muted ? `Ativar som: ${label}` : `Silenciar: ${label}`} onclick={mute}>
      <Icon name={muted ? 'mute' : 'volume'} />
    </button>
    <input type="range" min="0" max="100" value={pct(value)} style="--p:{pct(value)}" aria-label="Volume: {label}"
      onpointerdown={() => (busy = true)} onpointerup={() => { busy = false; setTimeout(load, 300); }}
      oninput={e => set(+e.currentTarget.value / 100)} />
    <span class="mono pc">{pct(value)}</span>
  </div>
{/snippet}

{#if !st}
  <div class="empty">{err || 'Carregando áudio…'}</div>
{:else}
  <div class="mixer" class:x={expanded}>
    <div class="col">
      {#if !expanded}
        <select aria-label="Saída de áudio" value={def?.id} onchange={e => run(audioSetDefault(e.currentTarget.value).then(load))}>
          {#each st.devices as d}<option value={d.id}>{d.name}</option>{/each}
        </select>
      {:else}
        <span class="label">Saída</span>
        <div class="devs">
          {#each st.devices as d}
            <button class:on={d.default} onclick={() => run(audioSetDefault(d.id).then(load))}><Icon name="speaker" /><span class="ellipsis">{d.name}</span></button>
          {/each}
        </div>
      {/if}
      <span class="label">Geral</span>
      {@render slider(st.master, st.muted, 'Geral', v => { st && (st.master = v); run(audioSetMaster(v)); }, () => run(audioSetMasterMute(!st?.muted).then(load)))}
    </div>

    {#if expanded || st.sessions.length}
      <div class="col apps scroll">
        <span class="label">Aplicativos</span>
        {#each expanded ? st.sessions : st.sessions.slice(0, 3) as s (s.pid)}
          <div class="app">
            <span class="ellipsis name" title={s.name}>{s.name}</span>
            {@render slider(s.volume, s.muted, s.name, v => { s.volume = v; run(audioSetSession(s.pid, v)); }, () => run(audioSetSessionMute(s.pid, !s.muted).then(load)))}
          </div>
        {:else}<div class="faint">Nenhum app tocando som.</div>{/each}
      </div>
    {/if}
    {#if err}<div class="err">{err}</div>{/if}
  </div>
{/if}

<style>
  .mixer { height: 100%; display: flex; flex-direction: column; gap: 12px; min-height: 0; }
  .mixer.x { display: grid; grid-template-columns: 1fr 1.4fr; gap: 32px; }
  .vol { display: flex; align-items: center; gap: 8px; }
  .vol input { flex: 1; }
  .vol.muted input { opacity: 0.4; }
  .pc { width: 26px; text-align: right; font-size: 12px; color: var(--muted); }
  .apps { flex: 1; gap: 4px; }
  .app .name { font-size: 12px; color: var(--muted); }
  .x .app .name { font-size: 14px; color: var(--text); }
  .x .app { padding: 6px 0; border-bottom: 1px solid var(--line); }
  .devs { display: flex; flex-direction: column; gap: 6px; }
  .devs button { display: flex; align-items: center; gap: 10px; text-align: left; padding: 10px 12px; }
  .devs .on { border-color: rgba(127, 224, 208, 0.5); color: var(--teal); background: rgba(127, 224, 208, 0.1); }
  .err { color: var(--danger); font-size: 12px; }
</style>
