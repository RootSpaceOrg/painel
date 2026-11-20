import type { Component } from 'svelte';
import type { WidgetType } from '../lib/state.svelte';
import Clock from './Clock.svelte';
import Media from './Media.svelte';
import Todo from './Todo.svelte';
import Habits from './Habits.svelte';
import Agenda from './Agenda.svelte';
import Hardware from './Hardware.svelte';
import Mixer from './Mixer.svelte';
import Bluetooth from './Bluetooth.svelte';
import FitToday from './FitToday.svelte';
import FitWeek from './FitWeek.svelte';
import FitTraining from './FitTraining.svelte';
import FitPlan from './FitPlan.svelte';
import FitFoods from './FitFoods.svelte';

// Novo widget = um componente com prop `expanded` + uma linha aqui.
export interface WidgetDef { label: string; comp: Component<{ expanded?: boolean }>; size: [number, number]; min: [number, number]; bare?: boolean; immersive?: boolean }
export const WIDGETS: Record<WidgetType, WidgetDef> = {
  clock: { label: 'Relógio', comp: Clock, size: [4, 3], min: [2, 2], bare: true },
  media: { label: 'Tocando agora', comp: Media, size: [5, 3], min: [3, 2], bare: true, immersive: true },
  todo: { label: 'Tarefas', comp: Todo, size: [4, 5], min: [2, 3] },
  habits: { label: 'Metas do dia', comp: Habits, size: [4, 4], min: [2, 2] },
  agenda: { label: 'Agenda', comp: Agenda, size: [4, 5], min: [2, 2] },
  hardware: { label: 'Sistema', comp: Hardware, size: [6, 3], min: [3, 2] },
  mixer: { label: 'Volume', comp: Mixer, size: [3, 3], min: [2, 2] },
  bluetooth: { label: 'Bluetooth', comp: Bluetooth, size: [3, 2], min: [2, 2] },
  fitToday: { label: 'Academia · Hoje', comp: FitToday, size: [4, 8], min: [3, 5] },
  fitWeek: { label: 'Academia · Semana', comp: FitWeek, size: [5, 4], min: [3, 3] },
  fitTraining: { label: 'Academia · Treino', comp: FitTraining, size: [5, 4], min: [3, 3] },
  fitPlan: { label: 'Academia · Plano', comp: FitPlan, size: [3, 5], min: [3, 4] },
  fitFoods: { label: 'Academia · Alimentos', comp: FitFoods, size: [3, 3], min: [2, 2] },
};
