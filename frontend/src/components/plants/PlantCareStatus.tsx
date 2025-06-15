import { Component, createSignal, Show, For, onMount } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { Button } from '@/components/ui/Button';
import { TrackingEntryForm } from './TrackingEntryForm';
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
  const [showTrackingForm, setShowTrackingForm] = createSignal(false);

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

  const openTrackingForm = () => {
    setShowTrackingForm(true);
  };

  const closeTrackingForm = () => {
    setShowTrackingForm(false);
    loadRecentEntries(); // Refresh entries when form is closed
  };

  const handleTrackingSubmit = async (data: CreateTrackingEntryRequest) => {
    await plantsStore.createTrackingEntry(props.plant.id, data);
  };

  const wateringDays = () => {
    const interval = props.plant.wateringSchedule?.intervalDays;
    return interval ? calculateDaysUntil(props.plant.lastWatered ?? null, interval) : null;
  };
  const fertilizingDays = () => {
    const interval = props.plant.fertilizingSchedule?.intervalDays;
    return interval ? calculateDaysUntil(props.plant.lastFertilized ?? null, interval) : null;
  };
  
  const wateringOverdue = () => {
    const interval = props.plant.wateringSchedule?.intervalDays;
    return interval ? isOverdue(props.plant.lastWatered ?? null, interval) : false;
  };
  const fertilizingOverdue = () => {
    const interval = props.plant.fertilizingSchedule?.intervalDays;
    return interval ? isOverdue(props.plant.lastFertilized ?? null, interval) : false;
  };

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
    <div class="bg-white shadow-sm rounded-xl sm:rounded-2xl border border-gray-200 overflow-hidden">
      {/* Card Header */}
      <div class="px-4 sm:px-6 py-4 sm:py-5 border-b border-gray-100 bg-gray-50/50">
        <div class="flex items-center space-x-3">
          <div class="flex-shrink-0">
            <div class="w-8 h-8 sm:w-10 sm:h-10 bg-primary-100 rounded-lg flex items-center justify-center">
              <svg class="h-4 w-4 sm:h-5 sm:w-5 text-primary-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
          </div>
          <div>
            <h2 class="text-base sm:text-lg font-semibold text-gray-900">
              Plant Care Dashboard
            </h2>
            <p class="text-xs sm:text-sm text-gray-500">
              Monitor care status and log activities
            </p>
          </div>
        </div>
      </div>

      <div class="p-4 sm:p-6 space-y-8">
        {/* Care Status Section */}
        <div>
          <h3 class="text-base font-semibold text-gray-900 mb-4 flex items-center">
            <svg class="h-5 w-5 text-gray-400 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v10" />
            </svg>
            Care Status
          </h3>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <Show when={wateringDays() !== null} fallback={
              <div class="p-4 rounded-lg border border-gray-200 bg-gray-50">
                <div class="flex items-center">
                  <div class="flex-shrink-0">
                    <svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
                    </svg>
                  </div>
                  <div class="ml-3">
                    <h4 class="text-sm font-medium text-gray-600">Watering</h4>
                    <p class="text-sm text-gray-500 italic">No schedule set</p>
                  </div>
                </div>
              </div>
            }>
              <div class={`p-4 rounded-lg border ${getStatusColor(wateringDays()!, wateringOverdue())}`}>
                <div class="flex items-center">
                  <div class="flex-shrink-0">
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
                    </svg>
                  </div>
                  <div class="ml-3">
                    <h4 class="text-sm font-medium">Watering</h4>
                    <p class="text-sm font-semibold">
                      {getStatusText(wateringDays()!, wateringOverdue(), 'Water')}
                    </p>
                  </div>
                </div>
              </div>
            </Show>

            <Show when={fertilizingDays() !== null} fallback={
              <div class="p-4 rounded-lg border border-gray-200 bg-gray-50">
                <div class="flex items-center">
                  <div class="flex-shrink-0">
                    <svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                    </svg>
                  </div>
                  <div class="ml-3">
                    <h4 class="text-sm font-medium text-gray-600">Fertilizing</h4>
                    <p class="text-sm text-gray-500 italic">No schedule set</p>
                  </div>
                </div>
              </div>
            }>
              <div class={`p-4 rounded-lg border ${getStatusColor(fertilizingDays()!, fertilizingOverdue())}`}>
                <div class="flex items-center">
                  <div class="flex-shrink-0">
                    <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                    </svg>
                  </div>
                  <div class="ml-3">
                    <h4 class="text-sm font-medium">Fertilizing</h4>
                    <p class="text-sm font-semibold">
                      {getStatusText(fertilizingDays()!, fertilizingOverdue(), 'Fertilize')}
                    </p>
                  </div>
                </div>
              </div>
            </Show>
          </div>
        </div>

        {/* Divider */}
        <div class="border-t border-gray-200"></div>

        {/* Quick Actions Section */}
        <div>
          <h3 class="text-base font-semibold text-gray-900 mb-4 flex items-center">
            <svg class="h-5 w-5 text-gray-400 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
            Quick Actions
          </h3>
          <div class="space-y-4">
            {/* Quick Action Buttons */}
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
                Quick Water
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
                Quick Fertilize
              </Button>
            </div>

            {/* Create Tracking Entry */}
            <div class="border-t border-gray-100 pt-4">
              <Button
                variant="outline"
                size="sm"
                onClick={openTrackingForm}
                class="flex items-center justify-center w-full"
              >
                <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 4v16m8-8H4" />
                </svg>
                Create Tracking Entry
              </Button>
            </div>
          </div>
        </div>

        {/* Divider */}
        <div class="border-t border-gray-200"></div>

        {/* Recent Activities Section */}
        <div>
          <h3 class="text-base font-semibold text-gray-900 mb-4 flex items-center">
            <svg class="h-5 w-5 text-gray-400 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            Recent Activities
          </h3>

          <Show when={loading()}>
            <div class="text-center py-6">
              <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
              <p class="mt-2 text-sm text-gray-600">Loading recent activities...</p>
            </div>
          </Show>

          <Show when={!loading() && recentEntries().length === 0}>
            <div class="text-center py-6 bg-gray-50 rounded-lg">
              <svg class="mx-auto h-8 w-8 text-gray-400 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={1.5} d="M9 5H7a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
              <p class="text-sm text-gray-600 font-medium">No recent activities</p>
              <p class="text-xs text-gray-500 mt-1">Use the quick actions above to log care activities</p>
            </div>
          </Show>

          <Show when={!loading() && recentEntries().length > 0}>
            <div class="space-y-3">
              <For each={recentEntries()}>
                {(entry) => (
                  <div class="flex items-center space-x-3 p-3 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors duration-150">
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
                      <Show when={entry.value || entry.notes}>
                        <div class="text-sm text-gray-600 space-y-1">
                          <Show when={entry.value}>
                            <div class="flex items-center space-x-2">
                              <Show when={entry.entryType === 'watering' && entry.value && typeof entry.value === 'object' && 'amount' in entry.value}>
                                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                                  {(entry.value as any).amount} {(entry.value as any).unit}
                                </span>
                              </Show>
                              <Show when={entry.entryType === 'fertilizing' && entry.value && typeof entry.value === 'object' && 'type' in entry.value}>
                                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
                                  {(entry.value as any).type}
                                  <Show when={'amount' in (entry.value as any)}>
                                    {' '}• {(entry.value as any).amount} {(entry.value as any).unit}
                                  </Show>
                                  <Show when={'dilution' in (entry.value as any)}>
                                    {' '}• {(entry.value as any).dilution}
                                  </Show>
                                </span>
                              </Show>
                            </div>
                          </Show>
                          <Show when={entry.notes}>
                            <p class="truncate">{entry.notes}</p>
                          </Show>
                        </div>
                      </Show>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>

      {/* Tracking Entry Form Modal */}
      <Show when={showTrackingForm()}>
        <TrackingEntryForm
          plant={props.plant}
          onClose={closeTrackingForm}
          onSuccess={closeTrackingForm}
          onSubmit={handleTrackingSubmit}
        />
      </Show>
    </div>
  );
};