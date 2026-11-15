import type { Component } from 'svelte';
import type { WidgetType } from '../lib/state.svelte';
import Clock from './Clock.svelte';
import Media from './Media.svelte';
import Todo from './Todo.svelte';
import Habits from './Habits.svelte';
import Agenda from './Agenda.svelte';
import Hardware from './Hardware.svelte';
import Mixer from './Mixer.svelte';

// Novo widget = um componente com prop `expanded` + uma linha aqui.
export interface WidgetDef { label: string; comp: Component<{ expanded?: boolean }>; size: [number, number]; min: [number, number]; bare?: boolean }
export const WIDGETS: Record<WidgetType, WidgetDef> = {
  clock: { label: 'Relógio', comp: Clock, size: [4, 3], min: [2, 2], bare: true },
  media: { label: 'Tocando agora', comp: Media, size: [5, 3], min: [3, 2], bare: true },
  todo: { label: 'Tarefas', comp: Todo, size: [4, 5], min: [2, 3] },
  habits: { label: 'Metas do dia', comp: Habits, size: [4, 4], min: [2, 2] },
  agenda: { label: 'Agenda', comp: Agenda, size: [4, 5], min: [2, 3] },
  hardware: { label: 'Sistema', comp: Hardware, size: [6, 3], min: [3, 2] },
  mixer: { label: 'Volume', comp: Mixer, size: [3, 3], min: [2, 2] },
};
