import { Component, Show, For } from 'solid-js';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';

type TrackingEntry = components['schemas']['TrackingEntry'];

interface CalendarEvent {
  id: string;
  title: string;
  plant: Plant;
  entry: TrackingEntry;
  date: Date;
  type: 'watering' | 'fertilizing' | 'note' | 'customMetric';
}

interface DayActivitiesModalProps {
  isOpen: boolean;
  onClose: () => void;
  date: Date;
  events: CalendarEvent[];
  onEventClick: (event: CalendarEvent) => void;
}

export const DayActivitiesModal: Component<DayActivitiesModalProps> = (props) => {
  const getActivityIcon = (type: string) => {
    switch (type) {
      case 'watering':
        return (
          <div class="flex-shrink-0 w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
            <svg class="h-4 w-4 text-blue-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
            </svg>
          </div>
        );
      case 'fertilizing':
        return (
          <div class="flex-shrink-0 w-8 h-8 bg-green-100 rounded-full flex items-center justify-center">
            <svg class="h-4 w-4 text-green-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
            </svg>
          </div>
        );
      case 'note':
        return (
          <div class="flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
            <svg class="h-4 w-4 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
            </svg>
          </div>
        );
      case 'customMetric':
        return (
          <div class="flex-shrink-0 w-8 h-8 bg-purple-100 rounded-full flex items-center justify-center">
            <svg class="h-4 w-4 text-purple-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v10" />
            </svg>
          </div>
        );
      default:
        return (
          <div class="flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
            <svg class="h-4 w-4 text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
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

  const handleBackdropClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) {
      props.onClose();
    }
  };

  const formatDateHeader = (date: Date) => {
    return date.toLocaleDateString('en-GB', { 
      weekday: 'long', 
      year: 'numeric', 
      month: 'long', 
      day: 'numeric' 
    });
  };

  return (
    <Show when={props.isOpen}>
      <div 
        class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
        onClick={handleBackdropClick}
      >
        <div class="bg-white rounded-lg shadow-xl max-w-lg w-full max-h-96 overflow-y-auto">
          {/* Header */}
          <div class="flex items-center justify-between p-6 border-b border-gray-200">
            <div>
              <h2 class="text-lg font-semibold text-gray-900">
                Activities for {formatDateHeader(props.date)}
              </h2>
              <p class="text-sm text-gray-500 mt-1">
                {props.events.length} activities
              </p>
            </div>
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
          <div class="divide-y divide-gray-200">
            <Show when={props.events.length === 0}>
              <div class="p-8 text-center">
                <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                </svg>
                <h3 class="mt-2 text-sm font-medium text-gray-900">No activities</h3>
                <p class="mt-1 text-sm text-gray-500">
                  No plant care activities recorded for this date.
                </p>
              </div>
            </Show>

            <For each={props.events}>
              {(event) => (
                <div 
                  class="p-4 hover:bg-gray-50 cursor-pointer"
                  onClick={() => props.onEventClick(event)}
                >
                  <div class="flex items-start space-x-3">
                    {getActivityIcon(event.entry.entryType)}
                    
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center justify-between">
                        <div class="flex items-center space-x-2">
                          <h3 class="text-sm font-medium text-gray-900">
                            {getActivityTypeLabel(event.entry.entryType)}
                          </h3>
                          <span class="text-sm text-gray-500">•</span>
                          <span class="text-sm font-medium text-blue-600">
                            {event.plant.name}
                          </span>
                        </div>
                        <time class="text-xs text-gray-500">
                          {new Date(event.entry.timestamp).toLocaleTimeString('en-GB', {
                            hour: '2-digit',
                            minute: '2-digit'
                          })}
                        </time>
                      </div>
                      
                      <Show when={event.entry.notes}>
                        <p class="mt-1 text-sm text-gray-600 truncate">
                          {event.entry.notes}
                        </p>
                      </Show>
                      
                      <Show when={event.entry.entryType === 'customMetric' && event.entry.value}>
                        <p class="mt-1 text-sm text-gray-600">
                          Value: {typeof event.entry.value === 'string' 
                            ? event.entry.value 
                            : JSON.stringify(event.entry.value)
                          }
                        </p>
                      </Show>
                      
                      <Show when={event.entry.photoIds && Array.isArray(event.entry.photoIds) && event.entry.photoIds.length > 0}>
                        <p class="mt-1 text-xs text-gray-500">
                          📷 {(event.entry.photoIds as string[]).length} photo(s)
                        </p>
                      </Show>
                    </div>
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      </div>
    </Show>
  );
};