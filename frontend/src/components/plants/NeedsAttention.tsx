import { Component, createMemo, For, Show, createSignal } from 'solid-js';
import { A } from '@solidjs/router';
import type { Plant } from '@/types';
import { plantsStore } from '@/stores/plants';

interface NeedsAttentionProps {
  plants: Plant[];
}

interface OverdueItem {
  plant: Plant;
  taskId: string;
  taskName: string;
  taskIcon: string | null | undefined;
  daysOverdue: number;
}

export const NeedsAttention: Component<NeedsAttentionProps> = (props) => {
  const [loggingIds, setLoggingIds] = createSignal<Set<string>>(new Set());

  const overdueItems = createMemo(() => {
    const items: OverdueItem[] = [];
    for (const plant of props.plants) {
      if (plant.archivedAt) continue;
      for (const task of plant.careTasks || []) {
        if (task.isDue && task.daysOverdue != null && task.daysOverdue >= 0) {
          items.push({
            plant,
            taskId: task.id,
            taskName: task.name,
            taskIcon: task.icon,
            daysOverdue: task.daysOverdue,
          });
        }
      }
    }
    // Sort: most overdue first
    items.sort((a, b) => b.daysOverdue - a.daysOverdue);
    return items;
  });

  const handleQuickLog = async (item: OverdueItem, e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const key = `${item.plant.id}-${item.taskId}`;
    setLoggingIds((prev) => new Set([...prev, key]));

    try {
      await plantsStore.createTrackingEntry(item.plant.id, {
        timestamp: new Date().toISOString(),
        careTaskIds: [item.taskId],
      });
    } catch {
      // silently fail — user can retry
    } finally {
      setLoggingIds((prev) => {
        const next = new Set(prev);
        next.delete(key);
        return next;
      });
    }
  };

  const isLogging = (item: OverdueItem) =>
    loggingIds().has(`${item.plant.id}-${item.taskId}`);

  return (
    <Show when={overdueItems().length > 0}>
      <div class="px-4 sm:px-6 mb-6">
        <div class="max-w-7xl mx-auto">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
            <h2 class="text-sm font-semibold text-gray-700 uppercase tracking-wide">
              Needs Attention
            </h2>
            <span class="text-xs text-gray-400 font-medium">
              {overdueItems().length} {overdueItems().length === 1 ? 'task' : 'tasks'}
            </span>
          </div>

          <div class="space-y-2">
            <For each={overdueItems().slice(0, 8)}>
              {(item) => (
                <div class="flex items-center gap-3 bg-white border border-gray-100 rounded-xl px-4 py-3 shadow-sm hover:shadow-md transition-shadow">
                  {/* Plant thumbnail */}
                  <A href={`/plants/${item.plant.id}`} class="flex-shrink-0">
                    <Show
                      when={item.plant.previewUrl}
                      fallback={
                        <div class="w-10 h-10 rounded-full bg-primary-100 flex items-center justify-center">
                          <span class="text-primary-600 text-lg">
                            {item.plant.name.charAt(0)}
                          </span>
                        </div>
                      }
                    >
                      <img
                        src={item.plant.previewUrl!}
                        alt={item.plant.name}
                        class="w-10 h-10 rounded-full object-cover"
                      />
                    </Show>
                  </A>

                  {/* Info */}
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="text-sm font-medium text-gray-900 truncate">
                        {item.plant.name}
                      </span>
                      <span class="text-red-500 text-xs font-medium whitespace-nowrap">
                        {item.daysOverdue === 0 ? 'Due today' : `${item.daysOverdue}d overdue`}
                      </span>
                    </div>
                    <div class="flex items-center gap-1 text-xs text-gray-500">
                      <Show when={item.taskIcon}>
                        <span>{item.taskIcon}</span>
                      </Show>
                      <span>{item.taskName}</span>
                    </div>
                  </div>

                  {/* Quick-log button */}
                  <button
                    onClick={(e) => handleQuickLog(item, e)}
                    disabled={isLogging(item)}
                    class="flex-shrink-0 w-9 h-9 rounded-full bg-primary-50 hover:bg-primary-100 text-primary-600 flex items-center justify-center transition-colors disabled:opacity-50"
                    title={`Log ${item.taskName}`}
                  >
                    <Show
                      when={!isLogging(item)}
                      fallback={
                        <svg class="w-4 h-4 animate-spin" viewBox="0 0 24 24" fill="none">
                          <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                          <path d="M4 12a8 8 0 018-8" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
                        </svg>
                      }
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2.5" d="M5 13l4 4L19 7" />
                      </svg>
                    </Show>
                  </button>
                </div>
              )}
            </For>

            <Show when={overdueItems().length > 8}>
              <p class="text-xs text-gray-400 text-center pt-1">
                +{overdueItems().length - 8} more tasks need attention
              </p>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
};
