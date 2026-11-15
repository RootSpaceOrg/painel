<script lang="ts">
  import { ui } from '../lib/state.svelte';
  import { WIDGETS } from '../widgets/registry';
  import Icon from './Icon.svelte';

  const w = $derived(ui.expanded!);
  const def = $derived(WIDGETS[w.type]);
</script>

<div class="overlay" role="presentation" onpointerdown={e => e.target === e.currentTarget && (ui.expanded = null)}>
  <div class="panel glass" role="dialog" aria-modal="true" aria-label={def.label}>
    <header>
      <span class="label">{def.label}</span>
      <button class="ghost icon" aria-label="Fechar (Esc)" onclick={() => (ui.expanded = null)}><Icon name="x" /></button>
    </header>
    <div class="body"><def.comp expanded /></div>
  </div>
</div>

<style>
  .overlay {
    position: fixed; inset: 0; z-index: 50; display: grid; padding: 22px;
    background: rgba(3, 12, 14, 0.45); animation: fade 0.18s ease-out;
  }
  .panel { display: flex; flex-direction: column; min-height: 0; padding: 18px 26px 26px; background: var(--glass-strong); animation: rise 0.22s cubic-bezier(0.2, 0.8, 0.2, 1); }
  header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; }
  .body { flex: 1; min-height: 0; position: relative; }
  @keyframes fade { from { opacity: 0; } }
  @keyframes rise { from { opacity: 0; transform: translateY(10px) scale(0.985); } }
  @media (prefers-reduced-motion: reduce) { .overlay, .panel { animation: none; } }
</style>
