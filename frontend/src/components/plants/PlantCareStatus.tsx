import { Component, createSignal, Show, For, onMount } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { Button } from '@/components/ui/Button';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';
import { calculateDaysUntil, isOverdue, formatDate } from '@/utils/date';

type TrackingEntry = components['schemas']['TrackingEntry'];
type CreateTrackingEntryRequest = components['schemas']['CreateTrackingEntryRequest'];
type EntryType = components['schemas']['EntryType'];

interface PlantCareStatusProps {
  plant: Plant;
}

export const PlantCareStatus: Component<PlantCareStatusProps> = (props) => {
  const [recentEntries, setRecentEntries] = createSignal<TrackingEntry[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [submitting, setSubmitting] = createSignal(false);

  const loadRecentEntries = async () => {
    try {
      setLoading(true);
      const response = await plantsStore.getTrackingEntries(props.plant.id);
      // Show only the 3 most recent entries
      setRecentEntries(response.entries.slice(0, 3));
    } catch (error) {
      console.error('Failed to load recent tracking entries:', error);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadRecentEntries();
  });

  const handleQuickAction = async (type: EntryType) => {
    try {
      setSubmitting(true);
      const payload: CreateTrackingEntryRequest = {
        entryType: type,
        timestamp: new Date().toISOString(),
      };
      await plantsStore.createTrackingEntry(props.plant.id, payload);
      await loadRecentEntries(); // Refresh recent entries
    } catch (error) {
      console.error('Failed to create tracking entry:', error);
    } finally {
      setSubmitting(false);
    }
  };

  const wateringDays = () => calculateDaysUntil(props.plant.lastWatered ?? null, props.plant.wateringIntervalDays);
  const fertilizingDays = () => calculateDaysUntil(props.plant.lastFertilized ?? null, props.plant.fertilizingIntervalDays);
  
  const wateringOverdue = () => isOverdue(props.plant.lastWatered ?? null, props.plant.wateringIntervalDays);
  const fertilizingOverdue = () => isOverdue(props.plant.lastFertilized ?? null, props.plant.fertilizingIntervalDays);

  const getStatusColor = (days: number, overdue: boolean) => {
    if (overdue) return 'text-red-600 bg-red-50 border-red-200';
    if (days <= 1) return 'text-yellow-600 bg-yellow-50 border-yellow-200';
    return 'text-green-600 bg-green-50 border-green-200';
  };

  const getStatusText = (days: number, overdue: boolean, type: string) => {
    if (overdue) return `${type} overdue`;
    if (days === 0) return `${type} today`;
    if (days === 1) return `${type} tomorrow`;
    return `${type} in ${days} days`;
  };

  const getEntryIcon = (type: EntryType) => {
    switch (type) {
      case 'watering':
        return (
          <svg class="h-4 w-4 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
          </svg>
        );
      case 'fertilizing':
        return (
          <svg class="h-4 w-4 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
          </svg>
        );
      case 'customMetric':
        return (
          <svg class="h-4 w-4 text-purple-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v10" />
          </svg>
        );
      case 'note':
        return (
          <svg class="h-4 w-4 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
          </svg>
        );
      default:
        return null;
    }
  };

  const getEntryTypeLabel = (type: EntryType) => {
    switch (type) {
      case 'watering': return 'Watered';
      case 'fertilizing': return 'Fertilized';
      case 'customMetric': return 'Measurement';
      case 'note': return 'Note';
      default: return type;
    }
  };

  return (
    <div class="space-y-6">
      {/* Care Status Cards */}
      <div class="card">
        <div class="card-header">
          <h3 class="text-lg font-medium text-gray-900">Care Status</h3>
        </div>
        <div class="card-body">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div class={`p-4 rounded-lg border ${getStatusColor(wateringDays(), wateringOverdue())}`}>
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
                  </svg>
                </div>
                <div class="ml-3">
                  <h4 class="text-sm font-medium">Watering</h4>
                  <p class="text-sm font-semibold">
                    {getStatusText(wateringDays(), wateringOverdue(), 'Water')}
                  </p>
                </div>
              </div>
            </div>

            <div class={`p-4 rounded-lg border ${getStatusColor(fertilizingDays(), fertilizingOverdue())}`}>
              <div class="flex items-center">
                <div class="flex-shrink-0">
                  <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                  </svg>
                </div>
                <div class="ml-3">
                  <h4 class="text-sm font-medium">Fertilizing</h4>
                  <p class="text-sm font-semibold">
                    {getStatusText(fertilizingDays(), fertilizingOverdue(), 'Fertilize')}
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div class="card">
        <div class="card-header">
          <h3 class="text-lg font-medium text-gray-900">Quick Actions</h3>
        </div>
        <div class="card-body">
          <div class="flex flex-wrap gap-3">
            <Button
              variant="primary"
              size="sm"
              onClick={() => handleQuickAction('watering')}
              loading={submitting()}
            >
              <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
              </svg>
              Water Now
            </Button>

            <Button
              variant="primary"
              size="sm"
              onClick={() => handleQuickAction('fertilizing')}
              loading={submitting()}
            >
              <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
              </svg>
              Fertilize Now
            </Button>
          </div>
        </div>
      </div>

      {/* Recent Activities */}
      <div class="card">
        <div class="card-header">
          <h3 class="text-lg font-medium text-gray-900">Recent Activities</h3>
        </div>
        <div class="card-body">
          <Show when={loading()}>
            <div class="text-center py-4">
              <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
              <p class="mt-2 text-sm text-gray-600">Loading recent activities...</p>
            </div>
          </Show>

          <Show when={!loading() && recentEntries().length === 0}>
            <div class="text-center py-4">
              <p class="text-sm text-gray-600">No recent activities</p>
              <p class="text-xs text-gray-500">Use the quick actions above to log care activities</p>
            </div>
          </Show>

          <Show when={!loading() && recentEntries().length > 0}>
            <div class="space-y-3">
              <For each={recentEntries()}>
                {(entry) => (
                  <div class="flex items-center space-x-3 p-3 bg-gray-50 rounded-lg">
                    <div class="flex-shrink-0">
                      {getEntryIcon(entry.entryType)}
                    </div>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center justify-between">
                        <p class="text-sm font-medium text-gray-900">
                          {getEntryTypeLabel(entry.entryType)}
                        </p>
                        <p class="text-xs text-gray-500">
                          {formatDate(entry.timestamp)}
                        </p>
                      </div>
                      <Show when={entry.notes}>
                        <p class="text-sm text-gray-600 truncate">{entry.notes}</p>
                      </Show>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};