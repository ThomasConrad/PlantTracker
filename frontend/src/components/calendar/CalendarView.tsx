import { Component, createSignal, Show, For, onMount, createMemo, createEffect } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';
import { EventDetailModal } from './EventDetailModal';
import { DayActivitiesModal } from './DayActivitiesModal';
import { MobileDayEvents } from './MobileDayEvents';

type TrackingEntry = components['schemas']['TrackingEntry'];

interface CalendarEvent {
  id: string;
  title: string;
  plant: Plant;
  entry: TrackingEntry;
  date: Date;
  type: 'watering' | 'fertilizing' | 'note' | 'customMetric';
}

interface CalendarViewProps {
  selectedPlants?: string[]; // Plant IDs to filter by
  selectedTypes?: string[]; // Entry types to filter by
}

export const CalendarView: Component<CalendarViewProps> = (props) => {
  const [currentDate, setCurrentDate] = createSignal(new Date());
  const [events, setEvents] = createSignal<CalendarEvent[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [selectedEvent, setSelectedEvent] = createSignal<CalendarEvent | null>(null);
  const [selectedDate, setSelectedDate] = createSignal<Date | null>(new Date());
  const [showEventDetail, setShowEventDetail] = createSignal(false);
  const [showDayActivities, setShowDayActivities] = createSignal(false);
  const [isMobile, setIsMobile] = createSignal(false);
  const [showMonthPicker, setShowMonthPicker] = createSignal(false);
  
  // Swipe handling for calendar
  const [calendarStartX, setCalendarStartX] = createSignal(0);
  const [calendarSwiping, setCalendarSwiping] = createSignal(false);

  // Get the first day of the current month
  const firstDayOfMonth = createMemo(() => {
    const date = new Date(currentDate());
    return new Date(date.getFullYear(), date.getMonth(), 1);
  });


  // Get the first day of the calendar (may be from previous month)
  const calendarStart = createMemo(() => {
    const firstDay = firstDayOfMonth();
    const dayOfWeek = firstDay.getDay();
    const start = new Date(firstDay);
    start.setDate(start.getDate() - dayOfWeek);
    return start;
  });

  // Get calendar days (42 days = 6 weeks) - reactive to selectedDate for highlighting
  const calendarDays = createMemo(() => {
    const days: Date[] = [];
    const start = calendarStart();
    
    for (let i = 0; i < 42; i++) {
      const day = new Date(start);
      day.setDate(start.getDate() + i);
      days.push(day);
    }
    
    // Force reactivity by accessing selectedDate
    selectedDate();
    
    return days;
  });

  // Get events for a specific date
  const getEventsForDate = (date: Date) => {
    const dateStr = date.toDateString();
    return events().filter(event => 
      event.date.toDateString() === dateStr
    );
  };

  // Check if date is in current month
  const isCurrentMonth = (date: Date) => {
    const current = currentDate();
    return date.getMonth() === current.getMonth() && 
           date.getFullYear() === current.getFullYear();
  };

  // Check if date is today
  const isToday = (date: Date) => {
    const today = new Date();
    return date.toDateString() === today.toDateString();
  };

  const loadCalendarData = async () => {
    try {
      setLoading(true);
      await plantsStore.loadPlants();
      
      const allEvents: CalendarEvent[] = [];
      const plants = plantsStore.plants;
      
      // Filter plants if specified
      const plantsToShow = props.selectedPlants?.length 
        ? plants.filter(plant => props.selectedPlants!.includes(plant.id))
        : plants;

      // Load tracking entries for each plant
      for (const plant of plantsToShow) {
        try {
          const response = await plantsStore.getTrackingEntries(plant.id);
          
          for (const entry of response.entries) {
            // Filter by selected types if specified
            if (props.selectedTypes?.length && !props.selectedTypes.includes(entry.entryType)) {
              continue;
            }
            
            allEvents.push({
              id: entry.id,
              title: getEventTitle(entry, plant),
              plant,
              entry,
              date: new Date(entry.timestamp),
              type: entry.entryType as 'watering' | 'fertilizing' | 'note' | 'customMetric'
            });
          }
        } catch (error) {
          console.error(`Failed to load entries for plant ${plant.id}:`, error);
        }
      }
      
      setEvents(allEvents);
    } catch (error) {
      console.error('Failed to load calendar data:', error);
    } finally {
      setLoading(false);
    }
  };

  const getEventTitle = (entry: TrackingEntry, plant: Plant) => {
    switch (entry.entryType) {
      case 'watering':
        return `💧 ${plant.name}`;
      case 'fertilizing':
        return `🌱 ${plant.name}`;
      case 'note':
        return `📝 ${plant.name}`;
      case 'customMetric':
        return `📊 ${plant.name}`;
      default:
        return plant.name;
    }
  };

  const getEventColor = (type: string) => {
    switch (type) {
      case 'watering':
        return 'bg-blue-100 text-blue-800 border-blue-200';
      case 'fertilizing':
        return 'bg-green-100 text-green-800 border-green-200';
      case 'note':
        return 'bg-gray-100 text-gray-800 border-gray-200';
      case 'customMetric':
        return 'bg-purple-100 text-purple-800 border-purple-200';
      default:
        return 'bg-gray-100 text-gray-800 border-gray-200';
    }
  };

  const navigateMonth = (direction: 'prev' | 'next') => {
    const current = currentDate();
    const newDate = new Date(current);
    
    if (direction === 'prev') {
      newDate.setMonth(current.getMonth() - 1);
    } else {
      newDate.setMonth(current.getMonth() + 1);
    }
    
    setCurrentDate(newDate);
  };

  const goToToday = () => {
    setCurrentDate(new Date());
  };

  // Calendar swipe handling
  const handleCalendarTouchStart = (e: TouchEvent) => {
    setCalendarStartX(e.touches[0].clientX);
    setCalendarSwiping(true);
  };

  const handleCalendarTouchEnd = (e: TouchEvent) => {
    if (!calendarSwiping()) return;
    
    const endX = e.changedTouches[0].clientX;
    const diffX = calendarStartX() - endX;
    
    // Minimum swipe distance
    if (Math.abs(diffX) > 50) {
      if (diffX > 0) {
        // Swipe left - next month
        navigateMonth('next');
      } else {
        // Swipe right - previous month
        navigateMonth('prev');
      }
    }
    
    setCalendarSwiping(false);
  };

  const handleEventClick = (event: CalendarEvent) => {
    setSelectedEvent(event);
    setShowEventDetail(true);
  };

  const handleDateClick = (date: Date) => {
    setSelectedDate(date);
    if (isMobile()) {
      // On mobile, always select the date (events shown in bottom view)
      return;
    }
    
    // On desktop, show modal if there are events
    const dayEvents = getEventsForDate(date);
    if (dayEvents.length > 0) {
      setShowDayActivities(true);
    }
  };

  const handleDayEventClick = (event: CalendarEvent) => {
    setShowDayActivities(false);
    setSelectedEvent(event);
    setShowEventDetail(true);
  };

  onMount(() => {
    loadCalendarData();
    
    // Detect mobile device
    const checkMobile = () => {
      const isMobileDevice = window.innerWidth < 640; // sm breakpoint
      setIsMobile(isMobileDevice);
    };
    
    checkMobile();
    window.addEventListener('resize', checkMobile);
    
    return () => window.removeEventListener('resize', checkMobile);
  });

  // Reload when selected plants change
  createEffect(() => {
    loadCalendarData();
  });

  // Create a reactive memo for selected date events to ensure updates
  const selectedDateEvents = createMemo(() => {
    const selected = selectedDate();
    if (!selected) return [];
    return getEventsForDate(selected);
  });

  // Remove automatic month update - let users navigate manually

  const monthNames = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];

  const dayNames = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  const dayNamesShort = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];

  // Get activity dots for a date
  const getActivityDots = (date: Date) => {
    const dayEvents = getEventsForDate(date);
    const activities = {
      watering: false,
      fertilizing: false,
      note: false,
      customMetric: false
    };
    
    dayEvents.forEach(event => {
      activities[event.type] = true;
    });
    
    return activities;
  };

  return (
    <div class={`${isMobile() ? 'h-screen flex flex-col' : 'bg-white rounded-lg shadow'}`}>
      {/* Mobile Layout */}
      <Show when={isMobile()}>
        {/* Compact Calendar - Top 2/5 */}
        <div class="flex-none overflow-hidden" style="height: 40vh;">
          <div class="bg-white h-full flex flex-col">
            {/* Compact Header */}
            <div class="flex items-center justify-between px-3 py-2 border-b border-gray-200">
              <button
                onClick={() => navigateMonth('prev')}
                class="p-1.5 text-gray-400 hover:text-gray-600 rounded-full"
              >
                <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
                </svg>
              </button>
              
              <button
                class="text-base font-semibold text-gray-900 hover:text-blue-600 transition-colors"
                onClick={() => setShowMonthPicker(true)}
              >
                {monthNames[currentDate().getMonth()]} {currentDate().getFullYear()}
              </button>
              
              <button
                onClick={() => navigateMonth('next')}
                class="p-1.5 text-gray-400 hover:text-gray-600 rounded-full"
              >
                <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
                </svg>
              </button>
            </div>

            <Show when={loading()}>
              <div class="flex justify-center py-4">
                <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
              </div>
            </Show>

            <Show when={!loading()}>
              <div 
                class="flex-1 p-1.5"
                onTouchStart={handleCalendarTouchStart}
                onTouchEnd={handleCalendarTouchEnd}
              >
                {/* Compact Day Headers */}
                <div class="grid grid-cols-7 gap-0.5 mb-1">
                  <For each={dayNamesShort}>
                    {(day) => (
                      <div class="p-1 text-center text-xs font-medium text-gray-600">
                        {day}
                      </div>
                    )}
                  </For>
                </div>

                {/* Compact Calendar Days */}
                <div class="grid grid-cols-7 gap-0.5">
                  <For each={calendarDays()}>
                    {(day) => {
                      const isCurrentMonthDay = isCurrentMonth(day);
                      const isTodayDay = isToday(day);
                      // Highlight selected date regardless of which month it's in
                      const isSelectedDay = selectedDate()?.toDateString() === day.toDateString();
                      const activities = getActivityDots(day);
                      
                      return (
                        <div 
                          class={`aspect-square flex flex-col items-center justify-center text-center cursor-pointer rounded-lg transition-colors ${
                            isSelectedDay ? 'bg-blue-100 border-2 border-blue-500' :
                            !isCurrentMonthDay ? 'text-gray-300' : 'hover:bg-gray-50'
                          }`}
                          onClick={() => handleDateClick(day)}
                        >
                          <div class={`text-sm font-medium flex items-center justify-center w-8 h-8 ${
                            isTodayDay && !isSelectedDay ? 'bg-gray-900 text-white rounded-full' :
                            isTodayDay && isSelectedDay ? 'bg-blue-700 text-white rounded-full' :
                            isSelectedDay ? 'text-blue-700' :
                            !isCurrentMonthDay ? 'text-gray-300' : 'text-gray-900'
                          }`}>
                            {day.getDate()}
                          </div>
                          
                          {/* Activity Dots */}
                          <div class="flex space-x-0.5 mt-0.5">
                            <Show when={activities.watering}>
                              <div class="w-1.5 h-1.5 bg-blue-500 rounded-full"></div>
                            </Show>
                            <Show when={activities.fertilizing}>
                              <div class="w-1.5 h-1.5 bg-green-500 rounded-full"></div>
                            </Show>
                            <Show when={activities.note}>
                              <div class="w-1.5 h-1.5 bg-gray-400 rounded-full"></div>
                            </Show>
                            <Show when={activities.customMetric}>
                              <div class="w-1.5 h-1.5 bg-purple-500 rounded-full"></div>
                            </Show>
                          </div>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </div>
            </Show>
          </div>
        </div>
        
        {/* Day Events - Bottom 3/5 */}
        <div class="flex-1 bg-gray-50">
          <MobileDayEvents 
            selectedDate={selectedDate()!}
            events={selectedDateEvents()}
            onEventClick={handleEventClick}
            onDateChange={setSelectedDate}
          />
        </div>
      </Show>

      {/* Month Picker Modal - Unified Design */}
      <Show when={showMonthPicker()}>
        <div 
          class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50 animate-in fade-in duration-200"
          onClick={() => setShowMonthPicker(false)}
        >
          <div 
            class="bg-white rounded-lg shadow-xl max-w-xs w-full transform transition-all duration-200 ease-out animate-in zoom-in-95"
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header with Year Navigation */}
            <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200">
              <button
                onClick={() => {
                  const newDate = new Date(currentDate());
                  newDate.setFullYear(currentDate().getFullYear() - 1);
                  setCurrentDate(newDate);
                }}
                class="p-1.5 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100 transition-colors"
              >
                <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
                </svg>
              </button>
              
              <div class="text-center">
                <div class="text-xs text-gray-500 uppercase tracking-wide">Year</div>
                <div class="text-lg font-semibold text-gray-900">{currentDate().getFullYear()}</div>
              </div>
              
              <button
                onClick={() => {
                  const newDate = new Date(currentDate());
                  newDate.setFullYear(currentDate().getFullYear() + 1);
                  setCurrentDate(newDate);
                }}
                class="p-1.5 text-gray-400 hover:text-gray-600 rounded-full hover:bg-gray-100 transition-colors"
              >
                <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
                </svg>
              </button>
            </div>
            
            {/* Month Grid */}
            <div class="p-4">
              <div class="grid grid-cols-3 gap-2">
                <For each={monthNames}>
                  {(month, index) => (
                    <button
                      class={`p-3 text-sm font-medium rounded-lg transition-all duration-150 ${
                        index() === currentDate().getMonth()
                          ? 'bg-blue-100 text-blue-700 border-2 border-blue-500'
                          : 'text-gray-700 hover:bg-gray-100 border-2 border-transparent'
                      }`}
                      onClick={() => {
                        const newDate = new Date(currentDate());
                        newDate.setMonth(index());
                        setCurrentDate(newDate);
                        setShowMonthPicker(false);
                      }}
                    >
                      {month.slice(0, 3)}
                    </button>
                  )}
                </For>
              </div>
            </div>
            
            {/* Actions */}
            <div class="px-4 pb-4 border-t border-gray-200 pt-4">
              <button
                class="w-full px-4 py-2 text-gray-600 font-medium rounded-lg hover:bg-gray-100 transition-colors"
                onClick={() => setShowMonthPicker(false)}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* Desktop Layout */}
      <Show when={!isMobile()}>
        {/* Calendar Header */}
        <div class="flex items-center justify-between p-6 border-b border-gray-200">
          <div class="flex items-center space-x-4">
            <h2 class="text-xl font-semibold text-gray-900">
              {monthNames[currentDate().getMonth()]} {currentDate().getFullYear()}
            </h2>
            <button
              onClick={goToToday}
              class="px-3 py-1 text-sm font-medium text-blue-600 hover:text-blue-700 border border-blue-300 rounded-md hover:bg-blue-50"
            >
              Today
            </button>
          </div>

          <div class="flex items-center space-x-2">
            <button
              onClick={() => navigateMonth('prev')}
              class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-full"
            >
              <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <button
              onClick={() => navigateMonth('next')}
              class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-full"
            >
              <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>
        </div>

        <Show when={loading()}>
          <div class="flex justify-center py-8">
            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          </div>
        </Show>

        <Show when={!loading()}>
          {/* Calendar Grid */}
          <div class="p-6">
            {/* Day Headers */}
            <div class="grid grid-cols-7 gap-px mb-2">
              <For each={dayNames}>
                {(day) => (
                  <div class="p-2 text-center text-sm font-medium text-gray-700">
                    {day}
                  </div>
                )}
              </For>
            </div>

            {/* Calendar Days */}
            <div class="grid grid-cols-7 gap-px bg-gray-200 rounded-lg overflow-hidden">
              <For each={calendarDays()}>
                {(day) => {
                  const dayEvents = getEventsForDate(day);
                  const isCurrentMonthDay = isCurrentMonth(day);
                  const isTodayDay = isToday(day);
                  
                  return (
                    <div 
                      class={`min-h-[100px] bg-white p-2 cursor-pointer hover:bg-gray-50 ${
                        !isCurrentMonthDay ? 'bg-gray-50 text-gray-400' : ''
                      }`}
                      onClick={() => handleDateClick(day)}
                    >
                      <div class={`text-sm font-medium mb-1 ${
                        isTodayDay ? 'bg-blue-600 text-white rounded-full w-6 h-6 flex items-center justify-center' : ''
                      }`}>
                        {day.getDate()}
                      </div>
                      
                      <div class="space-y-1">
                        <For each={dayEvents.slice(0, 3)}>
                          {(event) => (
                            <div 
                              class={`text-xs p-1 rounded border truncate cursor-pointer hover:opacity-80 ${getEventColor(event.type)}`}
                              title={`${event.title}${event.entry.notes ? ': ' + event.entry.notes : ''}`}
                              onClick={(e) => {
                                e.stopPropagation();
                                handleEventClick(event);
                              }}
                            >
                              {event.title}
                            </div>
                          )}
                        </For>
                        
                        <Show when={dayEvents.length > 3}>
                          <div class="text-xs text-gray-500 pl-1">
                            +{dayEvents.length - 3} more
                          </div>
                        </Show>
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>
          </div>
        </Show>
      </Show>

      {/* Event Detail Modal */}
      <Show when={selectedEvent()}>
        <EventDetailModal
          isOpen={showEventDetail()}
          onClose={() => {
            setShowEventDetail(false);
            setSelectedEvent(null);
          }}
          plant={selectedEvent()!.plant}
          entry={selectedEvent()!.entry}
        />
      </Show>

      {/* Day Activities Modal */}
      <Show when={selectedDate()}>
        <DayActivitiesModal
          isOpen={showDayActivities()}
          onClose={() => {
            setShowDayActivities(false);
            setSelectedDate(null);
          }}
          date={selectedDate()!}
          events={getEventsForDate(selectedDate()!)}
          onEventClick={handleDayEventClick}
        />
      </Show>
    </div>
  );
};