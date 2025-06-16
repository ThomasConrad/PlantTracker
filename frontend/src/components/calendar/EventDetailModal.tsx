import { Component, Show, For } from 'solid-js';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';
import { formatDate } from '@/utils/date';

type TrackingEntry = components['schemas']['TrackingEntry'];

interface EventDetailModalProps {
  isOpen: boolean;
  onClose: () => void;
  plant: Plant;
  entry: TrackingEntry;
}

export const EventDetailModal: Component<EventDetailModalProps> = (props) => {
  const getActivityTypeLabel = (type: string) => {
    switch (type) {
      case 'watering': return 'Watering';
      case 'fertilizing': return 'Fertilizing';
      case 'customMetric': return 'Measurement';
      case 'note': return 'Note';
      default: return type;
    }
  };

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'watering':
        return (
          <div class="flex-shrink-0 w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center">
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
            </svg>
          </div>
        );
      case 'fertilizing':
        return (
          <div class="flex-shrink-0 w-12 h-12 bg-green-100 rounded-full flex items-center justify-center">
            <svg class="h-6 w-6 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
            </svg>
          </div>
        );
      case 'note':
        return (
          <div class="flex-shrink-0 w-12 h-12 bg-gray-100 rounded-full flex items-center justify-center">
            <svg class="h-6 w-6 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
          </div>
        );
      case 'customMetric':
        return (
          <div class="flex-shrink-0 w-12 h-12 bg-purple-100 rounded-full flex items-center justify-center">
            <svg class="h-6 w-6 text-purple-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v10" />
            </svg>
          </div>
        );
      default:
        return (
          <div class="flex-shrink-0 w-12 h-12 bg-gray-100 rounded-full flex items-center justify-center">
            <svg class="h-6 w-6 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
            </svg>
          </div>
        );
    }
  };

  const handleBackdropClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) {
      props.onClose();
    }
  };

  return (
    <Show when={props.isOpen}>
      <div 
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
        onClick={handleBackdropClick}
      >
        <div class="bg-white rounded-lg shadow-xl max-w-md w-full max-h-96 overflow-y-auto">
          {/* Header */}
          <div class="flex items-center justify-between p-6 border-b border-gray-200">
            <h2 class="text-lg font-semibold text-gray-900">Activity Details</h2>
            <button
              onClick={props.onClose}
              class="text-gray-400 hover:text-gray-600"
            >
              <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Content */}
          <div class="p-6">
            <div class="flex items-start space-x-4">
              {getActivityIcon(props.entry.entryType)}
              
              <div class="flex-1">
                <div class="mb-3">
                  <h3 class="text-lg font-medium text-gray-900 mb-1">
                    {getActivityTypeLabel(props.entry.entryType)}
                  </h3>
                  <p class="text-sm text-gray-600">
                    <span class="font-medium text-blue-600">{props.plant.name}</span>
                    {props.plant.genus && (
                      <span class="text-gray-500"> ({props.plant.genus})</span>
                    )}
                  </p>
                  <p class="text-sm text-gray-500 mt-1">
                    {formatDate(props.entry.timestamp)}
                  </p>
                </div>

                <Show when={props.entry.notes}>
                  <div class="mb-4">
                    <h4 class="text-sm font-medium text-gray-700 mb-2">Notes</h4>
                    <p class="text-sm text-gray-600 bg-gray-50 p-3 rounded-md">
                      {props.entry.notes}
                    </p>
                  </div>
                </Show>

                <Show when={props.entry.entryType === 'customMetric' && props.entry.value}>
                  <div class="mb-4">
                    <h4 class="text-sm font-medium text-gray-700 mb-2">Measurement Value</h4>
                    <p class="text-sm text-gray-600 bg-gray-50 p-3 rounded-md">
                      {typeof props.entry.value === 'string' 
                        ? props.entry.value 
                        : JSON.stringify(props.entry.value)
                      }
                    </p>
                  </div>
                </Show>

                <Show when={props.entry.photoIds && Array.isArray(props.entry.photoIds) && props.entry.photoIds.length > 0}>
                  <div class="mb-4">
                    <h4 class="text-sm font-medium text-gray-700 mb-2">Photos ({(props.entry.photoIds as string[]).length})</h4>
                    <div class="grid grid-cols-2 gap-2">
                      <For each={props.entry.photoIds as string[]}>
                        {(photoId) => (
                          <img
                            src={`/api/v1/plants/${props.plant.id}/photos/${photoId}`}
                            alt="Activity photo"
                            class="w-full h-20 object-cover rounded border cursor-pointer hover:opacity-80 transition-opacity"
                            onClick={() => window.open(`/api/v1/plants/${props.plant.id}/photos/${photoId}`, '_blank')}
                          />
                        )}
                      </For>
                    </div>
                  </div>
                </Show>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};