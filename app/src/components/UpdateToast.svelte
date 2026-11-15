<script lang="ts">
  // Aviso de versão nova: fica no topo até a pessoa clicar (não some sozinho).
  import { upd, install } from '../lib/update.svelte';
  import Icon from './Icon.svelte';
  const v = $derived(upd.info?.latest ?? '');
</script>

{#if upd.info?.available && upd.dismissed !== v}
  <div class="toast glass" role="status">
    <span class="dot"></span>
    <span><b>Painel {v}</b> disponível <span class="faint">· você está na {upd.info.current}</span></span>
    {#if upd.error}<span class="err">{upd.error}</span>{/if}
    <button class="primary" onclick={install} disabled={upd.installing}>{upd.installing ? 'Baixando…' : 'Atualizar e reiniciar'}</button>
    <button class="ghost icon" aria-label="Fechar aviso" onclick={() => (upd.dismissed = v)}><Icon name="x" /></button>
  </div>
{/if}

<style>
  .toast {
    position: fixed; top: 14px; left: 50%; translate: -50% 0; z-index: 80;
    display: flex; align-items: center; gap: 14px; padding: 8px 8px 8px 16px; border-radius: 999px;
    background: var(--glass-strong); border-color: rgba(127, 224, 208, 0.35);
    animation: drop 0.4s cubic-bezier(0.2, 0.8, 0.2, 1);
  }
  .toast button.primary { border-radius: 999px; }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: var(--teal); box-shadow: 0 0 10px var(--teal); }
  .err { color: var(--danger); font-size: 12px; max-width: 260px; }
  @keyframes drop { from { opacity: 0; transform: translateY(-12px); } }
  @media (prefers-reduced-motion: reduce) { .toast { animation: none; } }
</style>
