import { Component, Show, For } from 'solid-js';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';

type CareTaskWithStatus = components['schemas']['CareTaskWithStatus'];

interface PlantStatsProps {
  plant: Plant;
}

export const PlantStats: Component<PlantStatsProps> = (props) => {
  const careTasks = () => (props.plant.careTasks ?? []).filter(t => t.intervalDays != null && !t.archivedAt);

  const getStatusColor = (task: CareTaskWithStatus) => {
    if (!task.isDue) return 'text-green-600 bg-green-50';
    const overdue = (task.daysOverdue ?? 0) > 0;
    if (overdue) return 'text-red-600 bg-red-50';
    return 'text-yellow-600 bg-yellow-50';
  };

  const getStatusText = (task: CareTaskWithStatus) => {
    const days = task.daysOverdue ?? 0;
    if (days > 0) return `${task.name} overdue`;
    if (days === 0) return `${task.name} today`;
    const abs = Math.abs(days);
    if (abs === 1) return `${task.name} tomorrow`;
    return `${task.name} in ${abs} days`;
  };

  return (
    <div class="card">
      <div class="card-header">
        <h3 class="text-lg font-medium text-gray-900">Care Status</h3>
      </div>
      <div class="card-body">
        <Show when={careTasks().length > 0} fallback={
          <div class="text-center py-4">
            <p class="text-sm text-gray-500 italic">No scheduled care tasks</p>
          </div>
        }>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <For each={careTasks()}>
              {(task) => (
                <div class={`p-4 rounded-lg ${getStatusColor(task)}`}>
                  <div class="flex items-center">
                    <div class="flex-shrink-0 text-2xl">
                      {task.icon ?? '🌱'}
                    </div>
                    <div class="ml-3">
                      <h4 class="text-sm font-medium">{task.name}</h4>
                      <p class="text-sm font-semibold">
                        {getStatusText(task)}
                      </p>
                      <p class="text-xs opacity-75">Every {task.intervalDays} days</p>
                    </div>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};
