import { Component, createEffect, createSignal, For, Show } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { PlantCard } from '@/components/plants/PlantCard';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Input } from '@/components/ui/Input';

export const SearchPage: Component = () => {
  const [searchQuery, setSearchQuery] = createSignal('');
  const [sortBy, setSortBy] = createSignal('date_desc');
  const [hasSearched, setHasSearched] = createSignal(false);

  createEffect(() => {
    const query = searchQuery().trim();
    if (query) {
      plantsStore.loadPlants({ 
        search: query,
        sort: sortBy()
      });
      setHasSearched(true);
    } else if (hasSearched()) {
      // If user clears search, load all plants
      plantsStore.loadPlants({ sort: sortBy() });
    }
  });

  const sortOptions = [
    { value: 'date_desc', label: 'Recently Added' },
    { value: 'date_asc', label: 'Oldest First' },
    { value: 'name_asc', label: 'Name A-Z' },
    { value: 'name_desc', label: 'Name Z-A' },
  ];

  return (
    <div class="space-y-6 pb-20">
      <div>
        <h1 class="text-2xl font-bold text-gray-900 mb-2">Search Plants</h1>
        <p class="text-gray-600">Find your plants by name or genus</p>
      </div>

      {/* Search Controls */}
      <div class="space-y-4">
        {/* Search Input */}
        <div class="relative">
          <Input
            placeholder="Search plants by name or genus..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="pl-10 text-base"
          />
          <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            <svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
        </div>

        {/* Sort Dropdown */}
        <div class="flex items-center justify-between">
          <span class="text-sm text-gray-600">
            {plantsStore.plants.length} {plantsStore.plants.length === 1 ? 'result' : 'results'}
          </span>
          <select
            class="text-sm bg-white border border-gray-300 rounded-md px-3 py-2"
            value={sortBy()}
            onChange={(e) => setSortBy(e.currentTarget.value)}
          >
            {sortOptions.map(option => (
              <option value={option.value}>{option.label}</option>
            ))}
          </select>
        </div>
      </div>

      {/* Results */}
      <Show
        when={!plantsStore.loading}
        fallback={
          <div class="flex justify-center py-12">
            <LoadingSpinner size="lg" />
          </div>
        }
      >
        <Show
          when={hasSearched() || searchQuery()}
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
                  d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                />
              </svg>
              <h3 class="mt-2 text-sm font-medium text-gray-900">Search your plants</h3>
              <p class="mt-1 text-sm text-gray-500">
                Start typing to search by plant name or genus
              </p>
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
                    d="M9.172 16.172a4 4 0 015.656 0M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                  />
                </svg>
                <h3 class="mt-2 text-sm font-medium text-gray-900">No plants found</h3>
                <p class="mt-1 text-sm text-gray-500">
                  Try a different search term or check your spelling
                </p>
              </div>
            }
          >
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <For each={plantsStore.plants}>
                {(plant) => <PlantCard plant={plant} />}
              </For>
            </div>
          </Show>
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