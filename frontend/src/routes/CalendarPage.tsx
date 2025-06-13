import { Component, createSignal, Show, For, onMount } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { CalendarView } from '@/components/calendar/CalendarView';
import { ActivityListView } from '@/components/calendar/ActivityListView';
import type { Plant } from '@/types';

export const CalendarPage: Component = () => {
  const [viewMode, setViewMode] = createSignal<'calendar' | 'list'>('calendar');
  const [selectedPlants, setSelectedPlants] = createSignal<string[]>([]);
  const [selectedTypes, setSelectedTypes] = createSignal<string[]>([]);
  const [showFilters, setShowFilters] = createSignal(false);

  onMount(() => {
    plantsStore.loadPlants();
  });

  const togglePlantFilter = (plantId: string) => {
    setSelectedPlants(prev => 
      prev.includes(plantId) 
        ? prev.filter(id => id !== plantId)
        : [...prev, plantId]
    );
  };

  const toggleTypeFilter = (type: string) => {
    setSelectedTypes(prev => 
      prev.includes(type) 
        ? prev.filter(t => t !== type)
        : [...prev, type]
    );
  };

  const clearAllFilters = () => {
    setSelectedPlants([]);
    setSelectedTypes([]);
  };

  const entryTypes = [
    { id: 'watering', label: 'Watering', color: 'text-blue-600 bg-blue-100' },
    { id: 'fertilizing', label: 'Fertilizing', color: 'text-green-600 bg-green-100' },
    { id: 'note', label: 'Notes', color: 'text-gray-600 bg-gray-100' },
    { id: 'customMetric', label: 'Measurements', color: 'text-purple-600 bg-purple-100' }
  ];

  return (
    <div class="h-full flex flex-col">
      {/* Header - Hidden on Mobile */}
      <div class="hidden sm:flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
        <div>
          <h1 class="text-2xl font-bold text-gray-900">Plant Calendar</h1>
          <p class="text-gray-600">Track all your plant care activities</p>
        </div>

        <div class="flex items-center space-x-4">
          {/* View Mode Toggle */}
          <div class="flex rounded-lg border border-gray-300 overflow-hidden">
            <button
              onClick={() => setViewMode('calendar')}
              class={`px-4 py-2 text-sm font-medium ${
                viewMode() === 'calendar'
                  ? 'bg-blue-600 text-white'
                  : 'bg-white text-gray-700 hover:bg-gray-50'
              }`}
            >
              <svg class="mr-2 h-4 w-4 inline" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
              Calendar
            </button>
            <button
              onClick={() => setViewMode('list')}
              class={`px-4 py-2 text-sm font-medium ${
                viewMode() === 'list'
                  ? 'bg-blue-600 text-white'
                  : 'bg-white text-gray-700 hover:bg-gray-50'
              }`}
            >
              <svg class="mr-2 h-4 w-4 inline" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M4 6h16M4 10h16M4 14h16M4 18h16" />
              </svg>
              List
            </button>
          </div>

          {/* Filter Toggle */}
          <button
            onClick={() => setShowFilters(!showFilters())}
            class={`flex items-center px-4 py-2 text-sm font-medium rounded-lg border ${
              showFilters()
                ? 'bg-blue-50 text-blue-700 border-blue-200'
                : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
            }`}
          >
            <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.207A1 1 0 013 6.5V4z" />
            </svg>
            Filters
            <Show when={selectedPlants().length > 0 || selectedTypes().length > 0}>
              <span class="ml-2 bg-blue-600 text-white text-xs rounded-full px-2 py-0.5">
                {selectedPlants().length + selectedTypes().length}
              </span>
            </Show>
          </button>
        </div>
      </div>

      {/* Filters Panel - Hidden on Mobile */}
      <Show when={showFilters()}>
        <div class="hidden sm:block bg-white rounded-lg shadow p-6">
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Plant Filters */}
            <div>
              <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-medium text-gray-900">Filter by Plants</h3>
                <Show when={selectedPlants().length > 0}>
                  <button
                    onClick={() => setSelectedPlants([])}
                    class="text-sm text-blue-600 hover:text-blue-700"
                  >
                    Clear plants
                  </button>
                </Show>
              </div>
              
              <div class="space-y-2 max-h-60 overflow-y-auto">
                <For each={plantsStore.plants}>
                  {(plant: Plant) => (
                    <label class="flex items-center">
                      <input
                        type="checkbox"
                        checked={selectedPlants().includes(plant.id)}
                        onChange={() => togglePlantFilter(plant.id)}
                        class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                      />
                      <span class="ml-3 text-sm text-gray-700">
                        {plant.name} <span class="text-gray-500 italic">({plant.genus})</span>
                      </span>
                    </label>
                  )}
                </For>
                
                <Show when={plantsStore.plants.length === 0}>
                  <p class="text-sm text-gray-500">No plants found</p>
                </Show>
              </div>
            </div>

            {/* Activity Type Filters */}
            <div>
              <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-medium text-gray-900">Filter by Activity Type</h3>
                <Show when={selectedTypes().length > 0}>
                  <button
                    onClick={() => setSelectedTypes([])}
                    class="text-sm text-blue-600 hover:text-blue-700"
                  >
                    Clear types
                  </button>
                </Show>
              </div>
              
              <div class="space-y-2">
                <For each={entryTypes}>
                  {(type) => (
                    <label class="flex items-center">
                      <input
                        type="checkbox"
                        checked={selectedTypes().includes(type.id)}
                        onChange={() => toggleTypeFilter(type.id)}
                        class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                      />
                      <span class={`ml-3 px-2 py-1 rounded-full text-xs font-medium ${type.color}`}>
                        {type.label}
                      </span>
                    </label>
                  )}
                </For>
              </div>
            </div>
          </div>

          {/* Clear All Filters */}
          <Show when={selectedPlants().length > 0 || selectedTypes().length > 0}>
            <div class="mt-6 pt-6 border-t border-gray-200">
              <button
                onClick={clearAllFilters}
                class="flex items-center px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200"
              >
                <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
                Clear All Filters
              </button>
            </div>
          </Show>
        </div>
      </Show>

      {/* Main Content */}
      <div class="flex-1 min-h-0 sm:mt-6">
        {/* Mobile: Always show calendar view */}
        <div class="sm:hidden h-full">
          <CalendarView 
            selectedPlants={selectedPlants().length > 0 ? selectedPlants() : undefined}
            selectedTypes={selectedTypes().length > 0 ? selectedTypes() : undefined}
          />
        </div>
        
        {/* Desktop: Show view mode toggle */}
        <div class="hidden sm:block h-full">
          <Show when={viewMode() === 'calendar'}>
            <CalendarView 
              selectedPlants={selectedPlants().length > 0 ? selectedPlants() : undefined}
              selectedTypes={selectedTypes().length > 0 ? selectedTypes() : undefined}
            />
          </Show>

          <Show when={viewMode() === 'list'}>
            <ActivityListView 
              selectedPlants={selectedPlants().length > 0 ? selectedPlants() : undefined}
              selectedTypes={selectedTypes().length > 0 ? selectedTypes() : undefined}
            />
          </Show>
        </div>
      </div>
    </div>
  );
};