import { Component, createEffect, createSignal, For, Show } from 'solid-js';
import { A } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { PlantCard } from '@/components/plants/PlantCard';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

export const PlantsPage: Component = () => {
  const [searchQuery] = createSignal('');
  const [sortBy, setSortBy] = createSignal('date_desc');

  createEffect(() => {
    plantsStore.loadPlants({ 
      search: searchQuery() || undefined,
      sort: sortBy()
    });
  });

  // Group plants by year when sorting by date
  const groupedPlants = () => {
    const plants = plantsStore.plants;
    const isDateSort = sortBy().includes('date');
    
    if (!isDateSort || plants.length === 0) {
      return [{ year: null, plants }];
    }

    const groups: { [key: string]: typeof plants } = {};
    
    plants.forEach(plant => {
      const year = plant.createdAt ? new Date(plant.createdAt).getFullYear().toString() : 'Unknown';
      if (!groups[year]) {
        groups[year] = [];
      }
      groups[year].push(plant);
    });

    // Convert to array and sort years
    const sortedGroups = Object.entries(groups)
      .map(([year, plants]) => ({ year, plants }))
      .sort((a, b) => {
        if (a.year === 'Unknown') return 1;
        if (b.year === 'Unknown') return -1;
        return sortBy() === 'date_desc' 
          ? parseInt(b.year) - parseInt(a.year)
          : parseInt(a.year) - parseInt(b.year);
      });

    return sortedGroups;
  };

  return (
    <div class="pb-20 sm:pb-6">
      {/* Header Section */}
      <div class="px-6 pt-6 pb-4">
        <A
          href="/plants/new"
          class="w-full flex items-center justify-center gap-3 bg-primary-600 text-white py-4 rounded-2xl font-semibold text-lg shadow-sm hover:bg-primary-700 transition-colors"
        >
          <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={2.5}>
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
          </svg>
          Add Plant
        </A>
        
        {/* Plant count and sort */}
        <div class="flex items-center justify-between mt-6">
          <div class="flex items-center gap-2">
            <span class="text-2xl font-bold text-gray-900">
              {plantsStore.plants.length}
            </span>
            <span class="text-gray-500 text-lg">
              {plantsStore.plants.length === 1 ? 'plant' : 'plants'}
            </span>
          </div>
          
          <div class="flex items-center gap-2">
            <span class="text-sm text-gray-500 font-medium">Sort by</span>
            <select
              class="text-sm bg-gray-50 border border-gray-200 rounded-lg px-3 py-2 font-medium text-gray-700 focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
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
      </div>

      <Show
        when={!plantsStore.loading}
        fallback={
          <div class="flex justify-center py-16">
            <div class="flex flex-col items-center gap-4">
              <LoadingSpinner size="lg" />
              <p class="text-gray-500 font-medium">Loading your plants...</p>
            </div>
          </div>
        }
      >
        <Show
          when={plantsStore.plants.length > 0}
          fallback={
            <div class="px-6 py-16 text-center">
              <div class="w-24 h-24 mx-auto mb-6 bg-gray-100 rounded-full flex items-center justify-center">
                <svg
                  class="h-12 w-12 text-gray-400"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  stroke-width={1.5}
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
                  />
                </svg>
              </div>
              <h3 class="text-xl font-semibold text-gray-900 mb-2">No plants yet</h3>
              <p class="text-gray-500 mb-8 max-w-sm mx-auto leading-relaxed">
                {searchQuery() ? 'No plants match your search. Try a different term.' : 'Start your plant journey by adding your first green companion.'}
              </p>
              {!searchQuery() && (
                <A 
                  href="/plants/new" 
                  class="inline-flex items-center gap-2 bg-primary-600 text-white px-6 py-3 rounded-xl font-semibold hover:bg-primary-700 transition-colors"
                >
                  <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width={2}>
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 4v16m8-8H4" />
                  </svg>
                  Add Your First Plant
                </A>
              )}
            </div>
          }
        >
          <div class="px-6">
            <For each={groupedPlants()}>
              {(group) => (
                <div class="mb-8">
                  {/* Year Header (only show for date sorting) */}
                  <Show when={group.year !== null}>
                    <div class="mb-6">
                      <h2 class="text-2xl font-bold text-gray-900 mb-1">{group.year}</h2>
                      <div class="h-1 w-12 bg-primary-600 rounded-full"></div>
                    </div>
                  </Show>
                  
                  {/* Plants Grid */}
                  <div class="grid grid-cols-1 h-full sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 sm:gap-6">
                    <For each={group.plants}>
                      {(plant) => <PlantCard plant={plant} />}
                    </For>
                  </div>
                </div>
              )}
            </For>
          </div>
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