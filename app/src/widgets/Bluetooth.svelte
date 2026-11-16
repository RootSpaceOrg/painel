<script lang="ts">
  // Fones/caixas Bluetooth pareados: um clique conecta ou desconecta (sem abrir as Configurações do Windows).
  import { btList, btSet, type BtDevice } from '../lib/api';
  import Icon from '../components/Icon.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  let devices = $state<BtDevice[] | null>(null);
  let busy = $state<Record<string, boolean>>({}); // nome → esperando o Windows conectar/desconectar
  let err = $state('');

  const load = async () => {
    try { devices = await btList(); err = ''; } catch (e) { err = String(e); }
  };
  $effect(() => {
    load();
    const t = setInterval(load, 3000);
    return () => clearInterval(t);
  });

  async function toggle(d: BtDevice) {
    const want = !d.connected;
    busy[d.name] = true;
    err = '';
    try {
      await btSet(d.name, want);
      // o Windows leva alguns segundos; acompanha até o estado mudar (máx. ~15s)
      for (let i = 0; i < 15; i++) {
        await new Promise(r => setTimeout(r, 1000));
        await load();
        if (devices?.find(x => x.name === d.name)?.connected === want) break;
      }
      if (devices?.find(x => x.name === d.name)?.connected !== want)
        err = want ? `${d.name} não conectou. Está ligado e fora do estojo?` : `${d.name} não desconectou.`;
    } catch (e) {
      err = String(e);
    } finally {
      busy[d.name] = false;
    }
  }
</script>

<div class="bt" class:x={expanded}>
  {#if devices === null}
    <div class="empty">{err || 'Procurando dispositivos…'}</div>
  {:else if !devices.length}
    <div class="empty">Nenhum fone ou caixa Bluetooth pareado.<br />Pareie uma vez nas Configurações do Windows.</div>
  {:else}
    <ul class="scroll">
      {#each devices as d (d.name)}
        <li class:on={d.connected}>
          <span class="ic"><Icon name="headphones" /></span>
          <div class="grow">
            <div class="nm ellipsis" title={d.name}>{d.name}</div>
            <div class="st">
              <i class="dot"></i>{busy[d.name] ? (d.connected ? 'Desconectando…' : 'Conectando…') : d.connected ? 'Conectado' : 'Desconectado'}
            </div>
          </div>
          <button class:primary={!d.connected} class:ghost={d.connected} disabled={busy[d.name]} onclick={() => toggle(d)}
            aria-label="{d.connected ? 'Desconectar' : 'Conectar'} {d.name}">
            {d.connected ? 'Desconectar' : 'Conectar'}
          </button>
        </li>
      {/each}
    </ul>
    {#if err}<div class="err">{err}</div>{/if}
  {/if}
  {#if expanded}<p class="hint">Mostra fones e caixas de som já pareados com o Windows.</p>{/if}
</div>

<style>
  .bt { height: 100%; display: flex; flex-direction: column; gap: 8px; min-height: 0; container-type: inline-size; }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; flex: 1; min-height: 0; }
  li { display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: 12px; background: rgba(190, 240, 225, 0.05); border: 1px solid transparent; }
  li.on { background: rgba(127, 224, 208, 0.1); border-color: rgba(127, 224, 208, 0.25); }
  .ic { width: 32px; height: 32px; flex: none; display: grid; place-items: center; border-radius: 10px; background: rgba(0, 0, 0, 0.2); color: var(--muted); }
  .on .ic { color: var(--teal); }
  .ic :global(svg) { width: 18px; height: 18px; }
  .nm { font-size: 13.5px; font-weight: 500; }
  .st { display: flex; align-items: center; gap: 6px; font-size: 11.5px; color: var(--faint); }
  .on .st { color: var(--teal); }
  .dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; }
  button { padding: 4px 12px; font-size: 12px; border-radius: 999px; flex: none; }
  .err { font-size: 11.5px; color: var(--danger); }
  .hint { font-size: 12px; color: var(--faint); margin: 0; }
  .x li { padding: 12px 14px; }
  .x .nm { font-size: 16px; }
  /* card estreito: some o ícone do fone para o nome caber */
  @container (max-width: 230px) {
    li { gap: 8px; padding: 7px 8px; }
    .ic { display: none; }
    button { padding: 4px 9px; font-size: 11px; }
  }
</style>
