import { Component, createEffect, createSignal, For, Show } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { PlantCard } from '@/components/plants/PlantCard';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

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
    <div class="pb-20">
      {/* Header Section */}
      <div class="px-6 pt-6 pb-6">
        <div class="text-center mb-8">
          <div class="w-16 h-16 mx-auto mb-4 bg-primary-50 rounded-2xl flex items-center justify-center">
            <svg class="h-8 w-8 text-primary-600" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={2}>
              <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
          <h1 class="text-2xl font-bold text-gray-900 mb-2">Search Plants</h1>
          <p class="text-gray-500">Find your plants by name or genus</p>
        </div>

        {/* Search Input */}
        <div class="relative mb-6">
          <input
            type="text"
            placeholder="Search plants by name or genus..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="w-full pl-12 pr-4 py-4 text-lg bg-gray-50 border border-gray-200 rounded-2xl focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500 transition-colors"
          />
          <div class="absolute inset-y-0 left-0 pl-4 flex items-center pointer-events-none">
            <svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={2}>
              <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
        </div>

        {/* Results count and sort */}
        <Show when={hasSearched() || searchQuery()}>
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-2xl font-bold text-gray-900">
                {plantsStore.plants.length}
              </span>
              <span class="text-gray-500 text-lg">
                {plantsStore.plants.length === 1 ? 'result' : 'results'}
              </span>
            </div>
            
            <div class="flex items-center gap-2">
              <span class="text-sm text-gray-500 font-medium">Sort by</span>
              <select
                class="text-sm bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 font-medium text-gray-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                value={sortBy()}
                onChange={(e) => setSortBy(e.currentTarget.value)}
              >
                {sortOptions.map(option => (
                  <option value={option.value}>{option.label}</option>
                ))}
              </select>
            </div>
          </div>
        </Show>
      </div>

      {/* Results */}
      <Show
        when={!plantsStore.loading}
        fallback={
          <div class="flex justify-center py-16">
            <div class="flex flex-col items-center gap-4">
              <LoadingSpinner size="lg" />
              <p class="text-gray-500 font-medium">Searching plants...</p>
            </div>
          </div>
        }
      >
        <Show
          when={hasSearched() || searchQuery()}
          fallback={
            <div class="px-6 py-16 text-center">
              <div class="w-24 h-24 mx-auto mb-6 bg-gray-100 rounded-full flex items-center justify-center">
                <svg class="h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={1.5}>
                  <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
              <h3 class="text-xl font-semibold text-gray-900 mb-2">Ready to search</h3>
              <p class="text-gray-500 max-w-sm mx-auto leading-relaxed">
                Start typing in the search box above to find your plants by name or genus
              </p>
            </div>
          }
        >
          <Show
            when={plantsStore.plants.length > 0}
            fallback={
              <div class="px-6 py-16 text-center">
                <div class="w-24 h-24 mx-auto mb-6 bg-gray-100 rounded-full flex items-center justify-center">
                  <svg class="h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={1.5}>
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9.172 16.172a4 4 0 015.656 0M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                </div>
                <h3 class="text-xl font-semibold text-gray-900 mb-2">No plants found</h3>
                <p class="text-gray-500 max-w-sm mx-auto leading-relaxed">
                  No plants match "{searchQuery()}". Try a different search term or check your spelling.
                </p>
              </div>
            }
          >
            <div class="px-6">
              <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 sm:gap-6">
                <For each={plantsStore.plants}>
                  {(plant) => <PlantCard plant={plant} />}
                </For>
              </div>
            </div>
          </Show>
        </Show>
      </Show>

      {plantsStore.error && (
        <div class="mx-6 mb-6 bg-red-50 border border-red-200 rounded-xl p-4">
          <div class="flex items-center gap-3">
            <svg class="h-5 w-5 text-red-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <p class="text-sm text-red-700 font-medium">{plantsStore.error}</p>
          </div>
        </div>
      )}
    </div>
  );
};