import { Component, createSignal, Show, For, onMount, createMemo } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';
import { formatDate } from '@/utils/date';
import { EventDetailModal } from './EventDetailModal';

type TrackingEntry = components['schemas']['TrackingEntry'];

interface ActivityEvent {
  id: string;
  plant: Plant;
  entry: TrackingEntry;
  date: Date;
}

interface ActivityListViewProps {
  selectedPlants?: string[]; // Plant IDs to filter by
  selectedTypes?: string[]; // Entry types to filter by
}

export const ActivityListView: Component<ActivityListViewProps> = (props) => {
  const [activities, setActivities] = createSignal<ActivityEvent[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [sortBy, setSortBy] = createSignal<'date' | 'plant' | 'type'>('date');
  const [sortOrder, setSortOrder] = createSignal<'asc' | 'desc'>('desc');
  const [selectedActivity, setSelectedActivity] = createSignal<ActivityEvent | null>(null);
  const [showEventDetail, setShowEventDetail] = createSignal(false);
  const [isMobile, setIsMobile] = createSignal(false);

  // Filter and sort activities
  const filteredActivities = createMemo(() => {
    let filtered = activities();

    // Filter by selected plants
    if (props.selectedPlants?.length) {
      filtered = filtered.filter(activity => 
        props.selectedPlants!.includes(activity.plant.id)
      );
    }

    // Filter by selected types
    if (props.selectedTypes?.length) {
      filtered = filtered.filter(activity => 
        props.selectedTypes!.includes(activity.entry.entryType)
      );
    }

    // Sort activities
    const sorted = [...filtered].sort((a, b) => {
      let comparison = 0;
      
      switch (sortBy()) {
        case 'date':
          comparison = a.date.getTime() - b.date.getTime();
          break;
        case 'plant':
          comparison = a.plant.name.localeCompare(b.plant.name);
          break;
        case 'type':
          comparison = a.entry.entryType.localeCompare(b.entry.entryType);
          break;
      }
      
      return sortOrder() === 'desc' ? -comparison : comparison;
    });

    return sorted;
  });

  const loadActivities = async () => {
    try {
      setLoading(true);
      await plantsStore.loadPlants();
      
      const allActivities: ActivityEvent[] = [];
      const plants = plantsStore.plants;

      // Load tracking entries for each plant
      for (const plant of plants) {
        try {
          const response = await plantsStore.getTrackingEntries(plant.id);
          
          for (const entry of response.entries) {
            allActivities.push({
              id: entry.id,
              plant,
              entry,
              date: new Date(entry.timestamp)
            });
          }
        } catch (error) {
          console.error(`Failed to load entries for plant ${plant.id}:`, error);
        }
      }
      
      setActivities(allActivities);
    } catch (error) {
      console.error('Failed to load activities:', error);
    } finally {
      setLoading(false);
    }
  };

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'watering':
        return (
          <div class="activity-icon-watering flex-shrink-0">
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
            </svg>
          </div>
        );
      case 'fertilizing':
        return (
          <div class="activity-icon-fertilizing flex-shrink-0">
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
            </svg>
          </div>
        );
      case 'note':
        return (
          <div class="activity-icon-default flex-shrink-0">
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
          </div>
        );
      case 'customMetric':
        return (
          <div class="activity-icon-custom flex-shrink-0">
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v10" />
            </svg>
          </div>
        );
      default:
        return (
          <div class="activity-icon-default flex-shrink-0">
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
            </svg>
          </div>
        );
    }
  };

  const getActivityTypeLabel = (type: string) => {
    switch (type) {
      case 'watering': return 'Watered';
      case 'fertilizing': return 'Fertilized';
      case 'customMetric': return 'Measurement';
      case 'note': return 'Note';
      default: return type;
    }
  };

  const handleSort = (newSortBy: 'date' | 'plant' | 'type') => {
    if (sortBy() === newSortBy) {
      setSortOrder(sortOrder() === 'asc' ? 'desc' : 'asc');
    } else {
      setSortBy(newSortBy);
      setSortOrder('desc');
    }
  };

  const getSortIcon = (column: 'date' | 'plant' | 'type') => {
    if (sortBy() !== column) {
      return (
        <svg class="h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
        </svg>
      );
    }
    
    return sortOrder() === 'asc' ? (
      <svg class="h-4 w-4 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M5 15l7-7 7 7" />
      </svg>
    ) : (
      <svg class="h-4 w-4 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19 9l-7 7-7-7" />
      </svg>
    );
  };

  const handleActivityClick = (activity: ActivityEvent) => {
    setSelectedActivity(activity);
    setShowEventDetail(true);
  };

  onMount(() => {
    loadActivities();
    
    // Detect mobile device
    const checkMobile = () => {
      const isMobileDevice = window.innerWidth < 640; // sm breakpoint
      setIsMobile(isMobileDevice);
    };
    
    checkMobile();
    window.addEventListener('resize', checkMobile);
    
    return () => window.removeEventListener('resize', checkMobile);
  });

  // Reload when filters change
  createMemo(() => {
    if (props.selectedPlants || props.selectedTypes) {
      // No need to reload, just filter existing data
    }
  });

  return (
    <div class={isMobile() ? 'activity-container-mobile' : 'activity-container-desktop'}>
      {/* Header - Simplified on Mobile */}
      <div class={isMobile() ? 'activity-header-mobile' : 'activity-header-desktop'}>
        <div class="flex items-center justify-between">
          <h2 class={isMobile() ? 'activity-title-mobile' : 'activity-title-desktop'}>
            Activity List
          </h2>
          <div class="text-sm text-gray-500">
            {filteredActivities().length} activities
          </div>
        </div>
      </div>

      <Show when={loading()}>
        <div class="flex justify-center py-8">
          <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        </div>
      </Show>

      <Show when={!loading()}>
        {/* Sort Controls */}
        <div class={isMobile() ? 'activity-sort-mobile' : 'activity-sort-desktop'}>
          <div class={isMobile() ? 'activity-sort-container-mobile' : 'activity-sort-container-desktop'}>
            <span class={isMobile() ? 'activity-sort-label-mobile' : 'activity-sort-label-desktop'}>Sort by:</span>
            
            <button
              onClick={() => handleSort('date')}
              class={`${isMobile() ? 'activity-sort-button-mobile' : 'activity-sort-button-desktop'} ${
                sortBy() === 'date' 
                  ? 'activity-sort-button-active' 
                  : 'activity-sort-button-inactive'
              }`}
            >
              <span>Date</span>
              {getSortIcon('date')}
            </button>
            
            <button
              onClick={() => handleSort('plant')}
              class={`${isMobile() ? 'activity-sort-button-mobile' : 'activity-sort-button-desktop'} ${
                sortBy() === 'plant' 
                  ? 'activity-sort-button-active' 
                  : 'activity-sort-button-inactive'
              }`}
            >
              <span>Plant</span>
              {getSortIcon('plant')}
            </button>
            
            <button
              onClick={() => handleSort('type')}
              class={`${isMobile() ? 'activity-sort-button-mobile' : 'activity-sort-button-desktop'} ${
                sortBy() === 'type' 
                  ? 'activity-sort-button-active' 
                  : 'activity-sort-button-inactive'
              }`}
            >
              <span>Type</span>
              {getSortIcon('type')}
            </button>
          </div>
        </div>

        {/* Activity List - constrained with flex-1 and overflow-y-auto */}
        <div class={isMobile() ? 'activity-list-mobile' : 'activity-list-desktop'}>
          <Show when={filteredActivities().length === 0}>
            <div class={isMobile() ? 'activity-empty-mobile' : 'activity-empty-desktop'}>
              <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <h3 class="mt-2 text-sm font-medium text-gray-900">No activities found</h3>
              <p class="mt-1 text-sm text-gray-500">
                {props.selectedPlants?.length || props.selectedTypes?.length 
                  ? 'Try adjusting your filters to see more activities.'
                  : 'Start tracking your plant care activities to see them here.'
                }
              </p>
            </div>
          </Show>

          <For each={filteredActivities()}>
            {(activity) => (
              <div 
                class={isMobile() ? 'activity-item-mobile' : 'activity-item-desktop'}
                onClick={() => handleActivityClick(activity)}
              >
                <div class={isMobile() ? 'activity-content-mobile' : 'activity-content-desktop'}>
                  {getActivityIcon(activity.entry.entryType)}
                  
                  <div class="flex-1 min-w-0">
                    <div class={isMobile() ? 'activity-details-mobile' : 'activity-details-desktop'}>
                      <div class={isMobile() ? 'activity-meta-mobile' : 'activity-meta-desktop'}>
                        <h3 class="font-medium text-gray-900 text-sm">
                          {getActivityTypeLabel(activity.entry.entryType)}
                        </h3>
                        <span class={`text-gray-500 ${isMobile() ? 'text-xs' : 'text-sm'}`}>•</span>
                        <span class="font-medium text-blue-600 text-sm">
                          {activity.plant.name}
                        </span>
                      </div>
                      <time class={`text-gray-500 ${isMobile() ? 'text-xs' : 'text-sm'}`}>
                        {formatDate(activity.entry.timestamp)}
                      </time>
                    </div>
                    
                    <Show when={activity.entry.notes}>
                      <p class="mt-1 text-sm text-gray-600">{activity.entry.notes}</p>
                    </Show>
                    
                    <Show when={activity.entry.entryType === 'customMetric' && activity.entry.value}>
                      <p class="mt-1 text-sm text-gray-600">
                        Value: {typeof activity.entry.value === 'string' 
                          ? activity.entry.value 
                          : JSON.stringify(activity.entry.value)
                        }
                      </p>
                    </Show>
                    
                    <Show when={activity.entry.photoIds && Array.isArray(activity.entry.photoIds) && activity.entry.photoIds.length > 0}>
                      <p class="mt-1 text-xs text-gray-500">
                        📷 {(activity.entry.photoIds as string[]).length} photo(s) attached
                      </p>
                    </Show>
                  </div>
                </div>
              </div>
            )}
          </For>
        </div>
      </Show>

      {/* Event Detail Modal */}
      <Show when={selectedActivity()}>
        <EventDetailModal
          isOpen={showEventDetail()}
          onClose={() => {
            setShowEventDetail(false);
            setSelectedActivity(null);
          }}
          plant={selectedActivity()!.plant}
          entry={selectedActivity()!.entry}
        />
      </Show>
    </div>
  );
};