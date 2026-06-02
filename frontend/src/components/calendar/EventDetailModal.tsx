import { Component, Show, For } from 'solid-js';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';
import { formatDate } from '@/utils/date';

type TrackingEntry = components['schemas']['TrackingEntry'];

function deriveEntryType(entry: TrackingEntry): 'care' | 'measurement' | 'photo' | 'note' {
  if (entry.careTaskIds && entry.careTaskIds.length > 0) return 'care';
  if (entry.measurements && entry.measurements.length > 0) return 'measurement';
  if (entry.photoIds && entry.photoIds.length > 0) return 'photo';
  return 'note';
}

interface EventDetailModalProps {
  isOpen: boolean;
  onClose: () => void;
  plant: Plant;
  entry: TrackingEntry;
}

export const EventDetailModal: Component<EventDetailModalProps> = (props) => {
  const getActivityTypeLabel = (type: string) => {
    switch (type) {
      case 'care': return 'Care';
      case 'measurement': return 'Measurement';
      case 'note': return 'Note';
      case 'photo': return 'Photo';
      default: return type;
    }
  };

  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'care':
        return (
          <div class="flex-shrink-0 w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center">
            <svg class="h-6 w-6 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </div>
        );
      case 'measurement':
        return (
          <div class="flex-shrink-0 w-12 h-12 bg-purple-100 rounded-full flex items-center justify-center">
            <svg class="h-6 w-6 text-purple-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
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
      case 'photo':
        return (
          <div class="flex-shrink-0 w-12 h-12 bg-indigo-100 rounded-full flex items-center justify-center">
            <svg class="h-6 w-6 text-indigo-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
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
              {getActivityIcon(deriveEntryType(props.entry))}
              
              <div class="flex-1">
                <div class="mb-3">
                  <h3 class="text-lg font-medium text-gray-900 mb-1">
                    {getActivityTypeLabel(deriveEntryType(props.entry))}
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

                <Show when={deriveEntryType(props.entry) === 'measurement' && props.entry.measurements?.length}>
                  <div class="mb-4">
                    <h4 class="text-sm font-medium text-gray-700 mb-2">Measurement Value</h4>
                    <p class="text-sm text-gray-600 bg-gray-50 p-3 rounded-md">
                      {JSON.stringify(props.entry.measurements![0].value)}
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