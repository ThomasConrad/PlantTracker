import { Component, createEffect, Show } from 'solid-js';
import { A, useParams } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { CoachChat } from '@/components/plants/CoachChat';

export const PlantCoachPage: Component = () => {
  const params = useParams();

  createEffect(() => {
    if (params.id) {
      plantsStore.loadPlant(params.id);
    }
  });

  const plant = () => plantsStore.selectedPlant;

  return (
    <div class="flex flex-col h-[calc(100vh-4rem)]">
      {/* Header */}
      <div class="flex items-center gap-3 px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <A
          href={`/plants/${params.id}`}
          class="p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        >
          <svg class="w-5 h-5 text-gray-600 dark:text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
        </A>
        <div class="flex-1 min-w-0">
          <h1 class="text-lg font-semibold text-gray-900 dark:text-gray-100 truncate">
            Plant Coach
          </h1>
          <Show when={plant()}>
            <p class="text-sm text-gray-500 dark:text-gray-400 truncate">{plant()!.name}</p>
          </Show>
        </div>
      </div>

      {/* Chat */}
      <Show
        when={plant() && !plantsStore.loading}
        fallback={
          <div class="flex-1 flex items-center justify-center">
            <LoadingSpinner size="lg" />
          </div>
        }
      >
        <CoachChat plantId={params.id} plantName={plant()!.name} />
      </Show>
    </div>
  );
};
