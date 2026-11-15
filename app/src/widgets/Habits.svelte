<script lang="ts">
  import { habitList, habitSave, habitDelete, habitMark, type Habit } from '../lib/api';
  import { today, dayKey, dayBit, addDays, streak, daysLabel, ALL_DAYS, DAY_NAMES } from '../lib/util';
  import Icon from '../components/Icon.svelte';
  import DaysPicker from '../components/DaysPicker.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  let habits = $state<Habit[]>([]);
  let day = $state(today());
  const blank = () => ({ name: '', icon: '💧', days: ALL_DAYS }) as { id?: number; name: string; icon: string; days: number };
  let form = $state(blank());

  const load = async () => (habits = await habitList());
  $effect(() => {
    load();
    const t = setInterval(() => { if (today() !== day) { day = today(); load(); } }, 30000);
    return () => clearInterval(t);
  });

  const rows = $derived(
    habits.map(h => {
      const log = new Set(h.log);
      const scheduled = !!((h.days || ALL_DAYS) & dayBit(new Date(day + 'T12:00')));
      return { h, log, done: log.has(day), scheduled, streak: streak(log, h.days, new Date(day + 'T12:00')) };
    }),
  );
  const todayRows = $derived(rows.filter(r => r.scheduled));
  const week = $derived(Array.from({ length: 7 }, (_, i) => addDays(new Date(day + 'T12:00'), i - 6)));

  async function mark(h: Habit, d: string, done: boolean) {
    await habitMark(h.id, d, done);
    load();
  }
  async function save() {
    if (!form.name.trim()) return;
    await habitSave({ ...form, name: form.name.trim(), icon: form.icon.trim() || '✦' });
    form = blank();
    load();
  }
  async function remove(id: number) {
    if (!confirm('Excluir esta meta e todo o histórico?')) return;
    await habitDelete(id);
    form = blank();
    load();
  }
</script>

{#if !expanded}
  {@const done = todayRows.filter(r => r.done).length}
  {@const C = 2 * Math.PI * 22}
  <div class="hc">
    {#if todayRows.length}
      <div class="sum">
        <svg class="ring" viewBox="0 0 52 52" aria-hidden="true">
          <circle cx="26" cy="26" r="22" class="trk" />
          <circle cx="26" cy="26" r="22" class="val" stroke-dasharray={C} stroke-dashoffset={C * (1 - done / todayRows.length)} />
        </svg>
        <div class="col tight">
          <span class="big mono">{done}<span class="faint">/{todayRows.length}</span></span>
          <span class="faint small">{done === todayRows.length ? 'tudo feito hoje ✨' : 'metas feitas hoje'}</span>
        </div>
      </div>
    {/if}
    <div class="list scroll">
      {#each todayRows as r (r.h.id)}
        <button class="hrow" class:done={r.done} onclick={() => mark(r.h, day, !r.done)} aria-pressed={r.done} title={r.h.name}>
          <span class="hic">{r.h.icon}</span>
          <span class="txt">
            <span class="name">{r.h.name}</span>
            <span class="meta">
              <span class="dots" aria-hidden="true">
                {#each week as d}{@const k = dayKey(d)}<i class:on={r.log.has(k)} class:today={k === day}></i>{/each}
              </span>
              {#if r.streak}<span class="streak mono"><Icon name="flame" filled />{r.streak} {r.streak === 1 ? 'dia' : 'dias'}</span>{/if}
            </span>
          </span>
          <span class="tick" aria-hidden="true"><Icon name="check" /></span>
        </button>
      {:else}
        <div class="empty">Nenhuma meta para hoje.<br />Clique no título para criar.</div>
      {/each}
    </div>
  </div>
{:else}
  <div class="full">
    <section class="col">
      <div class="row head">
        <span class="label">Metas</span>
        <div class="week-h">{#each week as d}<span title={dayKey(d)}>{DAY_NAMES[(d.getDay() + 6) % 7][0]}</span>{/each}</div>
      </div>
      <ul class="scroll">
        {#each rows as r (r.h.id)}
          <li>
            <span class="ic">{r.h.icon}</span>
            <div class="grow">
              <div class="ellipsis">{r.h.name}</div>
              <div class="faint small">{daysLabel(r.h.days)} · {r.streak} {r.streak === 1 ? 'dia seguido' : 'dias seguidos'}</div>
            </div>
            <div class="week">
              {#each week as d}
                {@const k = dayKey(d)}
                {@const sch = !!((r.h.days || ALL_DAYS) & dayBit(d))}
                <button class="dot" class:on={r.log.has(k)} class:off={!sch} aria-label="{r.h.name} em {k}"
                  aria-pressed={r.log.has(k)} onclick={() => mark(r.h, k, !r.log.has(k))}></button>
              {/each}
            </div>
            <button class="ghost icon" aria-label="Editar" onclick={() => (form = { id: r.h.id, name: r.h.name, icon: r.h.icon, days: r.h.days })}><Icon name="edit" /></button>
          </li>
        {:else}<li class="empty">Crie a primeira meta ao lado →</li>{/each}
      </ul>
    </section>
    <section class="col edit glass">
      <span class="label">{form.id ? 'Editar meta' : 'Nova meta'}</span>
        <div class="row">
          <input class="emoji" bind:value={form.icon} maxlength="4" aria-label="Ícone (emoji)" />
          <input class="grow" bind:value={form.name} placeholder="Beber água, academia, ler…" aria-label="Nome" onkeydown={e => e.key === 'Enter' && save()} />
        </div>
        <div class="row wrap">
          {#each ['💧', '🏋️', '📖', '🧘', '🚶', '🥗', '😴', '💊', '🎸', '✍️'] as e}
            <button class="ghost em" onclick={() => (form.icon = e)}>{e}</button>
          {/each}
        </div>
        <span class="label">Dias</span>
        <DaysPicker bind:value={form.days} />
        <div class="row">
          <button class="primary" onclick={save}>{form.id ? 'Salvar' : 'Criar meta'}</button>
          {#if form.id}
            <button class="ghost" onclick={() => (form = blank())}>Nova</button>
            <button class="ghost danger" onclick={() => form.id && remove(form.id)}>Excluir</button>
          {/if}
        </div>
    </section>
  </div>
{/if}

<style>
  .hc { height: 100%; display: flex; flex-direction: column; gap: 12px; min-height: 0; }
  .sum { display: flex; align-items: center; gap: 12px; }
  .ring { width: 46px; height: 46px; rotate: -90deg; flex: none; }
  .ring circle { fill: none; stroke-width: 5; }
  .trk { stroke: rgba(190, 240, 225, 0.12); }
  .val { stroke: var(--leaf); stroke-linecap: round; transition: stroke-dashoffset 0.6s cubic-bezier(0.3, 0.7, 0.2, 1); filter: drop-shadow(0 0 4px rgba(207, 224, 142, 0.5)); }
  .tight { gap: 0; }
  .big { font-size: 22px; font-weight: 300; line-height: 1.1; }
  .list { flex: 1; display: flex; flex-direction: column; gap: 8px; }
  .hrow {
    display: flex; align-items: center; gap: 12px; padding: 10px 12px; text-align: left; min-width: 0;
    background: rgba(190, 240, 225, 0.05); border-radius: 14px;
    transition: background 0.25s, border-color 0.25s, transform 0.12s;
  }
  .hrow:active { transform: scale(0.98); }
  .hrow.done {
    background: linear-gradient(100deg, rgba(207, 224, 142, 0.22), rgba(127, 224, 208, 0.08));
    border-color: rgba(207, 224, 142, 0.4);
  }
  .hic {
    width: 38px; height: 38px; flex: none; display: grid; place-items: center; font-size: 20px;
    background: rgba(0, 0, 0, 0.2); border-radius: 11px;
  }
  .txt { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 4px; }
  .name {
    font-size: 14px; font-weight: 500; line-height: 1.25;
    display: -webkit-box; -webkit-line-clamp: 2; line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;
  }
  .done .name { color: var(--leaf); }
  .meta { display: flex; align-items: center; gap: 10px; flex-wrap: wrap; }
  .dots { display: inline-flex; gap: 3px; }
  .dots i { width: 6px; height: 6px; border-radius: 50%; background: rgba(190, 240, 225, 0.16); }
  .dots i.on { background: var(--leaf); }
  .dots i.today { outline: 1px solid var(--muted); outline-offset: 1px; }
  .tick {
    width: 28px; height: 28px; flex: none; border-radius: 50%; display: grid; place-items: center;
    border: 1.5px solid var(--line-strong); color: transparent; transition: all 0.25s;
  }
  .tick :global(svg) { width: 15px; height: 15px; }
  .done .tick { background: var(--leaf); border-color: var(--leaf); color: var(--ink); box-shadow: 0 0 12px rgba(207, 224, 142, 0.45); animation: pop 0.3s; }
  @keyframes pop { 50% { transform: scale(1.18); } }
  @media (prefers-reduced-motion: reduce) { .done .tick { animation: none; } }
  .ic { font-size: 18px; line-height: 1; }
  .streak { display: inline-flex; align-items: center; gap: 3px; font-size: 11px; color: var(--amber); }
  .streak :global(svg) { width: 12px; height: 12px; }
  .full { height: 100%; display: grid; grid-template-columns: 1.6fr 1fr; gap: 28px; min-height: 0; }
  section { min-height: 0; }
  .head { justify-content: space-between; padding-right: 44px; }
  .week-h, .week { display: grid; grid-template-columns: repeat(7, 22px); gap: 6px; text-align: center; font-size: 11px; color: var(--faint); }
  ul { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 4px; }
  li { display: flex; align-items: center; gap: 12px; padding: 8px 4px; border-bottom: 1px solid var(--line); }
  li .ic { font-size: 22px; width: 30px; text-align: center; }
  .small { font-size: 12px; }
  .dot { width: 22px; height: 22px; padding: 0; border-radius: 6px; }
  .dot.on { background: var(--leaf); border-color: var(--leaf); }
  .dot.off:not(.on) { opacity: 0.35; }
  .edit { padding: 18px; align-self: start; }
  .emoji { width: 52px; text-align: center; font-size: 18px; }
  .wrap { flex-wrap: wrap; gap: 2px; }
  .em { padding: 4px 6px; font-size: 18px; }
</style>
