import { Component, createSignal } from 'solid-js';
import { Input } from '@/components/ui/Input';

interface PlantControlsProps {
  searchQuery: string;
  sortBy: string;
  onSearchChange: (query: string) => void;
  onSortChange: (sort: string) => void;
}

export const PlantControls: Component<PlantControlsProps> = (props) => {
  const [isFiltersOpen, setIsFiltersOpen] = createSignal(false);

  const sortOptions = [
    { value: 'date_desc', label: 'Recently Added' },
    { value: 'date_asc', label: 'Oldest First' },
    { value: 'name_asc', label: 'Name A-Z' },
    { value: 'name_desc', label: 'Name Z-A' },
  ];


  return (
    <div class="space-y-4 px-4 sm:px-0">
      {/* Search and Filter Toggle */}
      <div class="flex flex-col sm:flex-row gap-4">
        {/* Search Input */}
        <div class="relative flex-1 max-w-md">
          <Input
            placeholder="Search plants..."
            value={props.searchQuery}
            onInput={(e) => props.onSearchChange(e.currentTarget.value)}
            class="pl-10"
          />
          <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
            <svg class="h-5 w-5 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
          </div>
        </div>

        {/* Mobile Filter Toggle */}
        <button
          class="sm:hidden btn btn-secondary"
          onClick={() => setIsFiltersOpen(!isFiltersOpen())}
        >
          <svg class="h-5 w-5 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.414A1 1 0 013 6.707V4z" />
          </svg>
          Filters
        </button>

        {/* Desktop Sort Dropdown */}
        <div class="hidden sm:block relative">
          <select
            class="btn btn-secondary pr-8 appearance-none"
            value={props.sortBy}
            onChange={(e) => props.onSortChange(e.currentTarget.value)}
          >
            {sortOptions.map(option => (
              <option value={option.value}>{option.label}</option>
            ))}
          </select>
          <svg class="absolute right-2 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-500 pointer-events-none" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19 9l-7 7-7-7" />
          </svg>
        </div>
      </div>

      {/* Mobile Filters Panel */}
      <div class={`sm:hidden overflow-hidden transition-all duration-200 ${isFiltersOpen() ? 'max-h-40' : 'max-h-0'}`}>
        <div class="py-3 border-t border-gray-200">
          <label class="block text-sm font-medium text-gray-700 mb-2">Sort by</label>
          <select
            class="w-full btn btn-secondary"
            value={props.sortBy}
            onChange={(e) => props.onSortChange(e.currentTarget.value)}
          >
            {sortOptions.map(option => (
              <option value={option.value}>{option.label}</option>
            ))}
          </select>
        </div>
      </div>
    </div>
  );
};