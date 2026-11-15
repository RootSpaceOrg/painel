<script lang="ts">
  import { DAY_LETTERS, DAY_NAMES, ALL_DAYS, WEEKDAYS } from '../lib/util';
  let { value = $bindable(0), allowNone = false }: { value: number; allowNone?: boolean } = $props();
</script>

<div class="days" role="group" aria-label="Dias da semana">
  {#if allowNone}
    <button type="button" class:on={value === 0} onclick={() => (value = 0)} title="Tarefa avulsa (uma vez)">avulsa</button>
  {/if}
  <button type="button" class:on={value === ALL_DAYS} onclick={() => (value = ALL_DAYS)}>todo dia</button>
  <button type="button" class:on={value === WEEKDAYS} onclick={() => (value = WEEKDAYS)}>úteis</button>
  {#each DAY_LETTERS as l, i}
    <button type="button" class="d" class:on={value & (1 << i)} aria-pressed={!!(value & (1 << i))} title={DAY_NAMES[i]}
      onclick={() => (value ^= 1 << i)}>{l}</button>
  {/each}
</div>

<style>
  .days { display: flex; flex-wrap: wrap; gap: 4px; }
  button { padding: 3px 8px; font-size: 12px; border-radius: 8px; color: var(--muted); }
  .d { width: 26px; padding: 3px 0; }
  .on { background: rgba(207, 224, 142, 0.18); border-color: rgba(207, 224, 142, 0.45); color: var(--leaf); }
</style>
