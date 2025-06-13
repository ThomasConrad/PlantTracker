import { Component, createEffect, createSignal, For, Show } from 'solid-js';
import { A } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { PlantCard } from '@/components/plants/PlantCard';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

export const PlantsPage: Component = () => {
  const [searchQuery, setSearchQuery] = createSignal('');
  const [sortBy, setSortBy] = createSignal('date_desc');

  createEffect(() => {
    plantsStore.loadPlants({ 
      search: searchQuery() || undefined,
      sort: sortBy()
    });
  });

  return (
    <div class="space-y-4 sm:space-y-6 pb-20 sm:pb-6">
      <div class="px-4 sm:px-0 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div class="flex items-center gap-6">
          <p class="text-gray-600 font-medium">
            {plantsStore.plants.length} {plantsStore.plants.length === 1 ? 'plant' : 'plants'}
          </p>
          
          <div class="flex items-center gap-2">
            <span class="text-sm text-gray-500">Sort by</span>
            <select
              class="text-sm border-none bg-transparent font-medium text-gray-700 focus:outline-none cursor-pointer"
              value={sortBy()}
              onChange={(e) => setSortBy(e.currentTarget.value)}
            >
              <option value="date_desc">Recently Added</option>
              <option value="date_asc">Oldest First</option>
              <option value="name_asc">Name A-Z</option>
              <option value="name_desc">Name Z-A</option>
            </select>
          </div>
        </div>
        
        <A
          href="/plants/new"
          class="btn btn-primary btn-md"
        >
          <svg class="mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 4v16m8-8H4" />
          </svg>
          Add Plant
        </A>
      </div>

      <div class="px-4 sm:px-0">
        <div class="relative max-w-md">
          <input
            type="text"
            placeholder="Search plants..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="input pl-10"
          />
          <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            <svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
        </div>
      </div>

      <Show
        when={!plantsStore.loading}
        fallback={
          <div class="flex justify-center py-12">
            <LoadingSpinner size="lg" />
          </div>
        }
      >
        <Show
          when={plantsStore.plants.length > 0}
          fallback={
            <div class="text-center py-12">
              <svg
                class="mx-auto h-12 w-12 text-gray-400"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M12 6v6m0 0v6m0-6h6m-6 0H6"
                />
              </svg>
              <h3 class="mt-2 text-sm font-medium text-gray-900">No plants</h3>
              <p class="mt-1 text-sm text-gray-500">
                {searchQuery() ? 'No plants match your search.' : 'Get started by adding your first plant.'}
              </p>
              {!searchQuery() && (
                <div class="mt-6">
                  <A href="/plants/new" class="btn btn-primary">
                    <svg class="mr-2 h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 4v16m8-8H4" />
                    </svg>
                    Add Plant
                  </A>
                </div>
              )}
            </div>
          }
        >
          <div class="grid grid-cols-1 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3 sm:gap-4 px-4 sm:px-0">
            <For each={plantsStore.plants}>
              {(plant) => <PlantCard plant={plant} />}
            </For>
          </div>
        </Show>
      </Show>

      {plantsStore.error && (
        <div class="bg-red-50 border border-red-200 rounded-md p-4">
          <p class="text-sm text-red-600">{plantsStore.error}</p>
        </div>
      )}
    </div>
  );
};