import { Component, createEffect, createSignal, For, Show } from 'solid-js';
import { A } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { PlantCard } from '@/components/plants/PlantCard';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

export const PlantsPage: Component = () => {
  const [searchQuery, setSearchQuery] = createSignal('');
  const [sortBy, setSortBy] = createSignal('date_desc');
  const [mobileColumns, setMobileColumns] = createSignal(1); // Default to 1 column on mobile
  const MIN_CARD_WIDTH = 120; // Minimum card width in pixels
  
  // Pinch-to-zoom gesture handling
  let lastDistance = 0;
  let isGesturing = false;
  
  const getDistance = (touches: TouchList) => {
    if (touches.length < 2) return 0;
    const touch1 = touches[0];
    const touch2 = touches[1];
    return Math.sqrt(
      Math.pow(touch2.clientX - touch1.clientX, 2) + 
      Math.pow(touch2.clientY - touch1.clientY, 2)
    );
  };
  
  const getMaxColumns = () => {
    // Calculate max columns based on screen width and minimum card size
    const screenWidth = window.innerWidth - 32; // Account for padding (16px on each side)
    const gapWidth = 12; // 3 * 4px gap between cards
    return Math.floor((screenWidth + gapWidth) / (MIN_CARD_WIDTH + gapWidth));
  };
  
  const handleTouchStart = (e: TouchEvent) => {
    if (e.touches.length === 2) {
      lastDistance = getDistance(e.touches);
      isGesturing = true;
      e.preventDefault();
    }
  };
  
  const handleTouchMove = (e: TouchEvent) => {
    if (e.touches.length === 2 && isGesturing) {
      e.preventDefault();
      const currentDistance = getDistance(e.touches);
      const deltaDistance = currentDistance - lastDistance;
      
      // More sensitive pinch detection for smoother experience
      if (Math.abs(deltaDistance) > 15) {
        const currentCols = mobileColumns();
        const maxCols = getMaxColumns();
        
        if (deltaDistance > 0 && currentCols < maxCols) {
          // Pinch out - more columns (smaller cards)
          setMobileColumns(currentCols + 1);
        } else if (deltaDistance < 0 && currentCols > 1) {
          // Pinch in - fewer columns (larger cards)
          setMobileColumns(currentCols - 1);
        }
        
        lastDistance = currentDistance;
      }
    }
  };
  
  const handleTouchEnd = (e: TouchEvent) => {
    if (e.touches.length < 2) {
      isGesturing = false;
      lastDistance = 0;
    }
  };
  
  // Generate grid classes based on mobile column count
  const getGridClasses = () => {
    const cols = mobileColumns();
    let mobileColClass = '';
    
    // Generate dynamic grid classes
    if (cols === 1) mobileColClass = 'grid-cols-1';
    else if (cols === 2) mobileColClass = 'grid-cols-2';
    else if (cols === 3) mobileColClass = 'grid-cols-3';
    else if (cols === 4) mobileColClass = 'grid-cols-4';
    else if (cols === 5) mobileColClass = 'grid-cols-5';
    else if (cols === 6) mobileColClass = 'grid-cols-6';
    else if (cols === 7) mobileColClass = 'grid-cols-7';
    else if (cols === 8) mobileColClass = 'grid-cols-8';
    else mobileColClass = `grid-cols-${cols}`;
    
    return `grid ${mobileColClass} sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-3 sm:gap-4 px-4 sm:px-0`;
  };

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
          <div 
            class={getGridClasses()}
            onTouchStart={handleTouchStart}
            onTouchMove={handleTouchMove}
            onTouchEnd={handleTouchEnd}
          >
            <For each={plantsStore.plants}>
              {(plant) => <PlantCard plant={plant} mobileColumns={mobileColumns()} />}
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