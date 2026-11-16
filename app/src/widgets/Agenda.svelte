<script lang="ts">
  import { agenda, eventSave, eventDelete, onAgendaChanged, openUrl, type CalEvent } from '../lib/api';
  import { startOfDay, addDays, dayKey, parseDay, dayHeading, fmtTime, relMinutes } from '../lib/util';
  import { ui } from '../lib/state.svelte';
  import Icon from '../components/Icon.svelte';

  let { expanded = false }: { expanded?: boolean } = $props();
  let events = $state<CalEvent[]>([]);
  let now = $state(Date.now());
  const span = $derived(expanded ? 21 : 7);

  const load = async () => {
    const from = startOfDay(new Date()).getTime();
    events = await agenda(from, addDays(new Date(from), span).getTime());
  };
  $effect(() => {
    span; load();
    const un = onAgendaChanged(load);
    const t = setInterval(() => { now = Date.now(); }, 30000);
    const r = setInterval(load, 5 * 60000);
    return () => { un.then(f => f()); clearInterval(t); clearInterval(r); };
  });

  // agrupa por dia; um evento de vários dias aparece em cada dia que cobre
  const groups = $derived.by(() => {
    const out: { day: Date; items: CalEvent[] }[] = [];
    const base = startOfDay(new Date(now));
    for (let i = 0; i < span; i++) {
      const d0 = addDays(base, i).getTime(), d1 = addDays(base, i + 1).getTime();
      const items = events.filter(e => e.start < d1 && e.end > d0 && (i > 0 || expanded || e.end > now));
      if (items.length || i === 0) out.push({ day: new Date(d0), items });
    }
    return out;
  });

  const status = (e: CalEvent) => {
    if (e.all_day) return '';
    if (e.start <= now && e.end > now) return 'agora';
    const d = e.start - now;
    return d > 0 && d < 3 * 3600e3 ? `em ${relMinutes(d)}` : '';
  };
  // link de reunião no local/descrição (Meet, Zoom, Teams)
  const MEET = /https:\/\/(?:meet\.google\.com|[\w.-]*zoom\.us|teams\.microsoft\.com|teams\.live\.com)\/[^\s<>"')\\]+/i;
  // Meet abre na conta da agenda do evento: reaproveita o authuser que o backend pôs no link do Google Agenda
  const meeting = (e: CalEvent) => {
    const m = `${e.location} ${e.notes}`.match(MEET)?.[0] ?? '';
    if (!m || !/^https:\/\/meet\.google\.com\//i.test(m) || /[?&]authuser=/.test(m)) return m;
    let user: string | null = null;
    try { user = e.url ? new URL(e.url).searchParams.get('authuser') : null; } catch { /* link inválido: segue sem conta */ }
    return user ? `${m}${m.includes('?') ? '&' : '?'}authuser=${encodeURIComponent(user)}` : m;
  };
  const go = (url: string) => url && openUrl(url).catch(() => {});
  // pequeno: abre direto (reunião primeiro, senão a agenda); expandido: detalhes (local edita)
  const pick = (e: CalEvent) => (expanded ? open(e) : go(meeting(e) || e.url));

  const timeLabel = (e: CalEvent) => (e.all_day ? 'dia' : fmtTime.format(e.start));

  // ---------- formulário (eventos locais) ----------
  type Form = { id?: number; title: string; date: string; start: string; end: string; all_day: boolean; location: string; notes: string };
  const blank = (): Form => {
    const h = new Date(); h.setMinutes(0, 0, 0); h.setHours(h.getHours() + 1);
    const p = (n: number) => String(n).padStart(2, '0');
    return { title: '', date: dayKey(h), start: `${p(h.getHours())}:00`, end: `${p((h.getHours() + 1) % 24)}:00`, all_day: false, location: '', notes: '' };
  };
  let form = $state<Form>(blank());
  let viewing = $state<CalEvent | null>(null);
  let err = $state('');

  function open(e: CalEvent) {
    if (e.local_id === null) { viewing = e; return; }
    viewing = null;
    const s = new Date(e.start), en = new Date(e.end);
    const hm = (d: Date) => `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
    form = { id: e.local_id, title: e.title, date: dayKey(s), start: hm(s), end: hm(en), all_day: e.all_day, location: e.location, notes: e.notes };
  }

  async function save() {
    err = '';
    if (!form.title.trim()) { err = 'Dê um título.'; return; }
    let start: number, end: number;
    if (form.all_day) {
      start = parseDay(form.date).getTime();
      end = addDays(parseDay(form.date), 1).getTime();
    } else {
      start = new Date(`${form.date}T${form.start}`).getTime();
      end = new Date(`${form.date}T${form.end}`).getTime();
      if (end <= start) end += 864e5; // termina depois da meia-noite
    }
    if (Number.isNaN(start) || Number.isNaN(end)) { err = 'Data ou hora inválida.'; return; }
    await eventSave({ id: form.id, title: form.title.trim(), start, end, all_day: form.all_day, location: form.location, notes: form.notes });
    form = blank();
    load();
  }
  async function remove() {
    if (!form.id) return;
    await eventDelete(form.id);
    form = blank();
    load();
  }
</script>

{#snippet list()}
  <div class="list scroll">
    {#each groups as g}
      <div class="day">
        <div class="dh">{dayHeading(g.day)}</div>
        {#each g.items as e (e.id + g.day.getTime())}
          {@const st = status(e)}
          {@const meet = meeting(e)}
          {@const clickable = expanded || !!e.url || !!meet}
          <!-- clicar abre a reunião (Meet/Zoom/Teams) ou o evento no Google Agenda; no expandido, os detalhes -->
          <button class="ev" class:now={st === 'agora'} class:link={clickable} disabled={!clickable} onclick={() => pick(e)}
            title={expanded ? undefined : meet ? `Entrar na reunião: ${e.title}` : e.url ? `Abrir no Google Agenda: ${e.title}` : e.title}>
            <i class="bar" style="background:{e.color || 'var(--leaf)'}"></i>
            <span class="tm mono">{timeLabel(e)}</span>
            <span class="tt ellipsis">{e.title}</span>
            {#if st}<span class="st">{st}</span>{/if}
          </button>
        {:else}
          <div class="free">Livre pelo resto do dia</div>
        {/each}
      </div>
    {/each}
  </div>
{/snippet}

{#if !expanded}
  {@render list()}
{:else}
  <div class="full">
    {@render list()}
    <aside class="col">
      {#if viewing}
        <div class="glass pane col">
          <div class="row"><i class="dotc" style="background:{viewing.color}"></i><span class="label grow">Evento da agenda (só leitura)</span>
            <button class="ghost icon" aria-label="Fechar" onclick={() => (viewing = null)}><Icon name="x" /></button></div>
          <div class="vt">{viewing.title}</div>
          <div class="muted">{dayHeading(new Date(viewing.start))} · {viewing.all_day ? 'dia todo' : `${fmtTime.format(viewing.start)} – ${fmtTime.format(viewing.end)}`}</div>
          {#if viewing.location}<div class="muted">📍 {viewing.location}</div>{/if}
          {#if viewing.notes}<div class="notes scroll">{viewing.notes}</div>{/if}
          <div class="row wrap">
            {#if meeting(viewing)}<button class="primary" onclick={() => viewing && go(meeting(viewing))}>Entrar na reunião</button>{/if}
            {#if viewing.url}
              <button class="ghost" onclick={() => viewing && go(viewing.url)}>
                <Icon name="calendar" /> {viewing.url.includes('calendar.google.com') ? 'Abrir no Google Agenda' : 'Abrir no navegador'}
              </button>
            {/if}
          </div>
        </div>
      {/if}
      <form class="glass pane col" onsubmit={e => { e.preventDefault(); save(); }}>
        <span class="label">{form.id ? 'Editar evento' : 'Novo evento'}</span>
        <input bind:value={form.title} placeholder="Título" aria-label="Título" />
        <div class="row">
          <input type="date" bind:value={form.date} aria-label="Data" class="grow" />
          <label class="row small"><input type="checkbox" bind:checked={form.all_day} /> dia todo</label>
        </div>
        {#if !form.all_day}
          <div class="row">
            <input type="time" bind:value={form.start} aria-label="Início" class="grow" />
            <span class="faint">até</span>
            <input type="time" bind:value={form.end} aria-label="Fim" class="grow" />
          </div>
        {/if}
        <input bind:value={form.location} placeholder="Local (opcional)" aria-label="Local" />
        <textarea bind:value={form.notes} rows="3" placeholder="Notas" aria-label="Notas"></textarea>
        {#if err}<div class="err">{err}</div>{/if}
        <div class="row">
          <button class="primary" type="submit">{form.id ? 'Salvar' : 'Criar'}</button>
          {#if form.id}
            <button type="button" class="ghost" onclick={() => (form = blank())}>Novo</button>
            <button type="button" class="ghost danger" onclick={remove}>Excluir</button>
          {/if}
        </div>
      </form>
      <button class="ghost small" onclick={() => { ui.expanded = null; ui.settings = true; }}>
        <Icon name="calendar" /> Gerenciar agendas Google (links ICS)
      </button>
    </aside>
  </div>
{/if}

<style>
  .list { height: 100%; display: flex; flex-direction: column; gap: 10px; container-type: inline-size; }
  .day { display: flex; flex-direction: column; gap: 1px; }
  .dh { font: 600 10px/1 var(--sans); letter-spacing: 0.12em; text-transform: uppercase; color: var(--faint); margin: 0 0 4px; }
  .ev {
    display: grid; grid-template-columns: 3px 38px minmax(0, 1fr) auto; align-items: center; column-gap: 9px;
    padding: 4px 6px 4px 0; text-align: left; background: transparent; border: 0; border-radius: 7px;
    min-width: 0; cursor: default; font-size: 13px; line-height: 1.3;
  }
  .ev:disabled { opacity: 1; }
  .ev.link { cursor: pointer; }
  .ev.link:hover { background: rgba(190, 240, 225, 0.06); }
  .ev.now { background: rgba(127, 224, 208, 0.1); }
  .bar { width: 3px; align-self: stretch; border-radius: 2px; grid-row: 1 / -1; }
  .tm { font-size: 11px; color: var(--muted); white-space: nowrap; }
  .st { font-size: 10.5px; color: var(--teal); white-space: nowrap; }
  .free { font-size: 11.5px; color: var(--faint); padding-left: 12px; }
  .full .ev { font-size: 14px; padding: 5px 8px 5px 0; }
  /* estreito (widget com 2 colunas): hora + status em cima, título embaixo */
  @container (max-width: 250px) {
    .ev { grid-template-columns: 3px auto 1fr; grid-template-areas: 'bar tm st' 'bar tt tt'; row-gap: 1px; }
    .bar { grid-area: bar; }
    .tm { grid-area: tm; }
    .st { grid-area: st; justify-self: end; }
    .tt { grid-area: tt; font-size: 12.5px; }
  }
  .full { height: 100%; display: grid; grid-template-columns: 1.3fr 1fr; gap: 28px; min-height: 0; }
  aside { min-height: 0; overflow-y: auto; }
  .pane { padding: 16px; }
  .vt { font-size: 18px; font-weight: 600; }
  .notes { max-height: 160px; white-space: pre-wrap; font-size: 13px; color: var(--muted); user-select: text; }
  .dotc { width: 10px; height: 10px; border-radius: 50%; }
  .small { font-size: 12px; }
  label.small { white-space: nowrap; gap: 6px; }
  .err { color: var(--danger); font-size: 12px; }
  aside > button { display: inline-flex; gap: 8px; align-items: center; align-self: start; }
</style>
