import { Component, createSignal, Show, For, createMemo } from 'solid-js';
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

interface MobileDayEventsProps {
  selectedDate: Date;
  events: CalendarEvent[];
  onEventClick: (event: CalendarEvent) => void;
  onDateChange: (date: Date) => void;
}

export const MobileDayEvents: Component<MobileDayEventsProps> = (props) => {
  const [startX, setStartX] = createSignal(0);
  const [swiping, setSwiping] = createSignal(false);
  const [swipeOffset, setSwipeOffset] = createSignal(0);
  const [animating, setAnimating] = createSignal(false);

  // Group events by 10-minute intervals
  const eventsByTime = createMemo(() => {
    const grouped: { [key: string]: CalendarEvent[] } = {};

    props.events.forEach(event => {
      const eventDate = new Date(event.entry.timestamp);
      const hours = eventDate.getHours();
      const minutes = eventDate.getMinutes();

      // Round down to nearest 10-minute interval
      const roundedMinutes = Math.floor(minutes / 10) * 10;
      const timeKey = `${hours.toString().padStart(2, '0')}:${roundedMinutes.toString().padStart(2, '0')}`;

      if (!grouped[timeKey]) {
        grouped[timeKey] = [];
      }
      grouped[timeKey].push(event);
    });

    // Sort by time and ensure events within each group are sorted by actual timestamp
    const sortedTimes = Object.keys(grouped).sort();
    return sortedTimes.map(time => ({
      time,
      events: grouped[time].sort((a, b) =>
        new Date(a.entry.timestamp).getTime() - new Date(b.entry.timestamp).getTime()
      )
    }));
  });

  const getEventIcon = (type: string) => {
    switch (type) {
      case 'watering':
        return '💧';
      case 'fertilizing':
        return '🌱';
      case 'note':
        return '📝';
      case 'customMetric':
        return '📊';
      default:
        return '📝';
    }
  };

  const getEventColor = (type: string) => {
    switch (type) {
      case 'watering':
        return 'border-l-blue-500 bg-blue-50';
      case 'fertilizing':
        return 'border-l-green-500 bg-green-50';
      case 'note':
        return 'border-l-gray-500 bg-gray-50';
      case 'customMetric':
        return 'border-l-purple-500 bg-purple-50';
      default:
        return 'border-l-gray-500 bg-gray-50';
    }
  };

  const formatTime = (timeKey: string) => {
    // timeKey is already in HH:MM format for the 10-minute interval
    return timeKey;
  };

  // Swipe handling for day navigation
  const handleTouchStart = (e: TouchEvent) => {
    setStartX(e.touches[0].clientX);
    setSwiping(true);
    setSwipeOffset(0);
  };

  const handleTouchMove = (e: TouchEvent) => {
    if (!swiping()) return;
    
    const currentX = e.touches[0].clientX;
    const diffX = currentX - startX();
    
    // Limit the swipe offset to prevent excessive drag
    const maxOffset = 100;
    const limitedOffset = Math.max(-maxOffset, Math.min(maxOffset, diffX));
    setSwipeOffset(limitedOffset);
  };

  const handleTouchEnd = (e: TouchEvent) => {
    if (!swiping()) return;

    const endX = e.changedTouches[0].clientX;
    const diffX = startX() - endX;

    setSwiping(false);
    setAnimating(true);

    // Minimum swipe distance
    if (Math.abs(diffX) > 50) {
      const currentDate = props.selectedDate;
      const newDate = new Date(currentDate);

      if (diffX > 0) {
        // Swipe left - next day
        newDate.setDate(currentDate.getDate() + 1);
        props.onDateChange(newDate);
        setSwipeOffset(0);
      } else {
        // Swipe right - previous day
        newDate.setDate(currentDate.getDate() - 1);
        props.onDateChange(newDate);
        setSwipeOffset(0);
      }
    } else {
      // Animate back to center position smoothly
      setSwipeOffset(0);
    }
    
    // Turn off animation after transition completes
    setTimeout(() => {
      setAnimating(false);
    }, 200);
  };

  const goToPreviousDay = () => {
    const currentDate = props.selectedDate;
    const newDate = new Date(currentDate);
    newDate.setDate(currentDate.getDate() - 1);
    props.onDateChange(newDate);
  };

  const goToNextDay = () => {
    const currentDate = props.selectedDate;
    const newDate = new Date(currentDate);
    newDate.setDate(currentDate.getDate() + 1);
    props.onDateChange(newDate);
  };

  return (
    <div
      class="h-full flex flex-col bg-white"
      onTouchStart={handleTouchStart}
      onTouchMove={handleTouchMove}
      onTouchEnd={handleTouchEnd}
    >
      {/* Compact Day Header */}
      <div class="flex-none px-3 py-1.5 border-b border-gray-200 bg-white">
        <div class="flex items-center justify-between">
          <button
            onClick={goToPreviousDay}
            class="p-1.5 text-gray-400 hover:text-gray-600 rounded-full"
          >
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
            </svg>
          </button>

          <div class="text-center">
            <p class="text-xs text-gray-500">
              {props.events.length} {props.events.length === 1 ? 'activity' : 'activities'}
            </p>
          </div>

          <button
            onClick={goToNextDay}
            class="p-1.5 text-gray-400 hover:text-gray-600 rounded-full"
          >
            <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
            </svg>
          </button>
        </div>
      </div>

      {/* Events List */}
      <div 
        class={`flex-1 overflow-y-auto ${animating() ? 'transition-transform duration-[200ms] ease-out' : ''}`}
        style={`transform: translateX(${swipeOffset()}px)`}
      >
        <Show
          when={props.events.length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-full p-8 text-center">
              <div class="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4">
                <svg class="h-8 w-8 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 002 2v12a2 2 0 002 2z" />
                </svg>
              </div>
              <h4 class="text-lg font-medium text-gray-900 mb-2">No activities</h4>
              <p class="text-gray-500">
                No plant care activities recorded for this day.
              </p>
            </div>
          }
        >
          <div class="p-3 space-y-3">
            <For each={eventsByTime()}>
              {({ time, events }) => (
                <div class="space-y-1.5">
                  {/* Time Header */}
                  <div class="flex items-center">
                    <div class="text-xs font-medium text-gray-500">
                      {formatTime(time)}
                    </div>
                    <div class="flex-1 h-px bg-gray-200 ml-2"></div>
                  </div>

                  {/* Events at this time */}
                  <div class="space-y-1.5">
                    <For each={events}>
                      {(event) => (
                        <div
                          class={`border-l-4 p-2.5 rounded-r-lg cursor-pointer transition-colors ${getEventColor(event.type)} hover:shadow-md`}
                          onClick={() => props.onEventClick(event)}
                        >
                          <div class="flex items-start space-x-2.5">
                            <div class="text-base flex-shrink-0">
                              {getEventIcon(event.type)}
                            </div>

                            <div class="flex-1 min-w-0">
                              <div class="flex items-center justify-between">
                                <h4 class="text-sm font-medium text-gray-900 truncate">
                                  {event.plant.name}
                                </h4>
                                <span class="text-xs text-gray-500 italic ml-2">
                                  {event.plant.genus}
                                </span>
                              </div>

                              <p class="text-sm text-gray-600 capitalize mt-1">
                                {event.type === 'customMetric' ? 'Measurement' : event.type}
                                <Show when={event.entry.value}>
                                  <span class="ml-2 font-medium">
                                    {typeof event.entry.value === 'string'
                                      ? event.entry.value
                                      : String(event.entry.value)
                                    }
                                  </span>
                                </Show>
                              </p>

                              <Show when={event.entry.notes}>
                                <p class="text-sm text-gray-500 mt-2 line-clamp-2">
                                  {String(event.entry.notes)}
                                </p>
                              </Show>
                            </div>

                            <div class="flex-shrink-0">
                              <svg class="h-4 w-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
                              </svg>
                            </div>
                          </div>
                        </div>
                      )}
                    </For>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>

    </div>
  );
};