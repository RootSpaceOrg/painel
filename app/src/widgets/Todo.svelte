<script lang="ts">
  import { todoList, todoSave, todoDelete, type Todo } from '../lib/api';
  import { today, dayBit, daysLabel } from '../lib/util';
  import Icon from '../components/Icon.svelte';
  import DaysPicker from '../components/DaysPicker.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  let todos = $state<Todo[]>([]);
  let day = $state(today());
  let draft = $state('');
  let draftDays = $state(0);
  let editing = $state<Todo | null>(null);

  const load = async () => (todos = await todoList());
  $effect(() => {
    load();
    // vira o dia → recorrentes "desmarcam" sozinhas (done_on deixa de ser hoje)
    const t = setInterval(() => { if (today() !== day) { day = today(); load(); } }, 30000);
    return () => clearInterval(t);
  });

  const isDone = (t: Todo) => t.done_on === day;
  const isToday = (t: Todo) =>
    t.days === 0 ? !t.done_on || t.done_on === day : !!(t.days & dayBit(new Date()));
  const shown = $derived(
    todos.filter(isToday).sort((a, b) => +isDone(a) - +isDone(b) || a.id - b.id),
  );
  const doneCount = $derived(shown.filter(isDone).length);

  async function toggle(t: Todo) {
    await todoSave({ ...t, done_on: isDone(t) ? null : day });
    load();
  }
  async function add() {
    const title = draft.trim();
    if (!title) return;
    await todoSave({ title, days: draftDays, done_on: null });
    draft = '';
    load();
  }
  async function saveEdit() {
    if (!editing) return;
    if (editing.title.trim()) await todoSave({ ...editing, title: editing.title.trim() });
    editing = null;
    load();
  }
  async function remove(id: number) {
    await todoDelete(id);
    if (editing?.id === id) editing = null;
    load();
  }
  const clearDone = async () => {
    await Promise.all(todos.filter(t => t.days === 0 && t.done_on).map(t => todoDelete(t.id)));
    load();
  };
</script>

{#snippet item(t: Todo)}
  <li class:done={isDone(t)}>
    <button class="check" class:on={isDone(t)} aria-label={isDone(t) ? 'Desmarcar' : 'Concluir'} onclick={() => toggle(t)}>
      {#if isDone(t)}<Icon name="check" />{/if}
    </button>
    <span class="grow ellipsis t">{t.title}</span>
    {#if t.days}<span class="tag" title={daysLabel(t.days)}><Icon name="refresh" /></span>{/if}
  </li>
{/snippet}

<div class="todo" class:x={expanded}>
  <section class="today">
    <div class="row head">
      <span class="label">Hoje</span>
      <span class="mono faint small">{doneCount}/{shown.length}</span>
    </div>
    <ul class="scroll">
      {#each shown as t (t.id)}{@render item(t)}{:else}<li class="empty">Nada para hoje 🌿</li>{/each}
    </ul>
    <form class="add" onsubmit={e => { e.preventDefault(); add(); }}>
      <input bind:value={draft} placeholder="Nova tarefa…" aria-label="Nova tarefa" />
      {#if expanded}<button class="primary" type="submit">Adicionar</button>{/if}
    </form>
    {#if expanded}<DaysPicker bind:value={draftDays} allowNone />{/if}
  </section>

  {#if expanded}
    <section class="all">
      <div class="row head">
        <span class="label">Todas</span>
        <button class="ghost small" onclick={clearDone}>Limpar avulsas concluídas</button>
      </div>
      <ul class="scroll">
        {#each todos as t (t.id)}
          <li class="manage">
            {#if editing?.id === t.id}
              <div class="col grow">
                <input bind:value={editing.title} aria-label="Título" onkeydown={e => e.key === 'Enter' && saveEdit()} />
                <DaysPicker bind:value={editing.days} allowNone />
                <div class="row"><button class="primary" onclick={saveEdit}>Salvar</button><button class="ghost" onclick={() => (editing = null)}>Cancelar</button></div>
              </div>
            {:else}
              <span class="grow ellipsis" class:faint={t.days === 0 && t.done_on}>{t.title}</span>
              <span class="faint small">{daysLabel(t.days)}</span>
              <button class="ghost icon" aria-label="Editar" onclick={() => (editing = { ...t })}><Icon name="edit" /></button>
              <button class="ghost icon danger" aria-label="Excluir" onclick={() => remove(t.id)}><Icon name="trash" /></button>
            {/if}
          </li>
        {:else}<li class="empty">Sem tarefas</li>{/each}
      </ul>
    </section>
  {/if}
</div>

<style>
  .todo { height: 100%; display: flex; flex-direction: column; min-height: 0; }
  .todo.x { display: grid; grid-template-columns: 1fr 1.2fr; gap: 32px; }
  section { display: flex; flex-direction: column; gap: 8px; min-height: 0; height: 100%; }
  .head { justify-content: space-between; }
  .small { font-size: 12px; }
  ul { list-style: none; margin: 0; padding: 0; flex: 1; display: flex; flex-direction: column; gap: 2px; }
  li { display: flex; align-items: center; gap: 10px; padding: 6px 4px; border-radius: 8px; }
  li:hover { background: rgba(190, 240, 225, 0.04); }
  li.done .t { color: var(--faint); text-decoration: line-through; text-decoration-color: rgba(207, 224, 142, 0.5); }
  .tag { color: var(--faint); display: grid; }
  .tag :global(svg) { width: 12px; height: 12px; }
  .add { display: flex; gap: 8px; }
  .add input { flex: 1; }
  .manage { border-bottom: 1px solid var(--line); border-radius: 0; }
  .x .t { font-size: 16px; }
</style>
