<script lang="ts">
  // Registrar peso: número grande + teclado numérico grande + − / + (100 g). Começa no último peso registrado.
  import { fit, fitUi, fitDay, editDay } from '../lib/fit.svelte';
  import { lastWeight } from '../lib/fit.ts';
  import { dayHeading, parseDay } from '../lib/util';
  import Icon from './Icon.svelte';

  const d = fitDay();
  const start = fit.days[d]?.weight ?? lastWeight(fit.days, d) ?? fit.profile.weight;
  let text = $state(start.toFixed(1).replace('.', ','));
  let fresh = true; // primeira tecla substitui o valor (como numa calculadora)
  const value = $derived(parseFloat(text.replace(',', '.')));
  const valid = $derived(Number.isFinite(value) && value > 20 && value < 400);

  function key(k: string) {
    if (k === '⌫') { text = text.slice(0, -1); fresh = false; return; }
    if (fresh) { text = ''; fresh = false; }
    if (k === ',' && text.includes(',')) return;
    if (/,\d$/.test(text)) return; // uma casa decimal basta
    if (text.replace(',', '').length >= 4 && k !== ',') return;
    text = (text === '' && k === ',' ? '0' : text) + k;
  }
  const step = (dv: number) => { const v = (valid ? value : start) + dv; text = (Math.round(v * 10) / 10).toFixed(1).replace('.', ','); fresh = false; };
  function save() {
    if (!valid) return;
    editDay(d, x => { x.weight = Math.round(value * 10) / 10; });
    fitUi.weight = false;
  }
  function onKey(e: KeyboardEvent) {
    if (/^[0-9]$/.test(e.key)) key(e.key);
    else if (e.key === ',' || e.key === '.') key(',');
    else if (e.key === 'Backspace') key('⌫');
    else if (e.key === 'Enter') save();
    else if (e.key === 'Escape') fitUi.weight = false;
    else return;
    e.preventDefault();
    e.stopPropagation();
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="overlay" role="presentation" onpointerdown={e => e.target === e.currentTarget && (fitUi.weight = false)}>
  <div class="panel glass" role="dialog" aria-modal="true" aria-label="Registrar peso">
    <div class="row between">
      <span class="label">Peso · {dayHeading(parseDay(d))}</span>
      <button class="ghost icon" aria-label="Fechar" onclick={() => (fitUi.weight = false)}><Icon name="x" /></button>
    </div>
    <div class="disp">
      <button class="step" aria-label="Menos 100 g" onclick={() => step(-0.1)}>−</button>
      <span class="val mono" class:bad={!valid}>{text || '0'}<small> kg</small></span>
      <button class="step" aria-label="Mais 100 g" onclick={() => step(0.1)}>+</button>
    </div>
    <div class="pad">
      {#each ['1', '2', '3', '4', '5', '6', '7', '8', '9', ',', '0', '⌫'] as k}
        <button class="k mono" onclick={() => key(k)} aria-label={k === '⌫' ? 'Apagar' : k}>{k}</button>
      {/each}
    </div>
    <button class="primary save" disabled={!valid} onclick={save}>Salvar peso</button>
  </div>
</div>

<style>
  .overlay { position: fixed; inset: 0; z-index: 70; display: grid; place-items: center; background: rgba(3, 12, 14, 0.55); animation: fade 0.15s; }
  .panel { width: 380px; padding: 18px 22px 22px; display: flex; flex-direction: column; gap: 16px; background: var(--glass-strong); }
  .between { justify-content: space-between; }
  .disp { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
  .val { font-size: 58px; font-weight: 300; letter-spacing: -0.02em; }
  .val small { font-size: 20px; color: var(--muted); }
  .val.bad { color: var(--faint); }
  .step { width: 52px; height: 52px; border-radius: 50%; font-size: 26px; padding: 0; }
  .pad { display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; }
  .k { height: 58px; font-size: 24px; border-radius: 14px; }
  .k:active { transform: scale(0.96); }
  .save { padding: 14px; font-size: 16px; border-radius: 14px; }
  @keyframes fade { from { opacity: 0; } }
</style>
