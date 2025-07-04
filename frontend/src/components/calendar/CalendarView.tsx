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
  const [calendarSwipeOffset, setCalendarSwipeOffset] = createSignal(0);
  const [calendarAnimating, setCalendarAnimating] = createSignal(false);

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

  // Get previous month calendar days for seamless scrolling
  const prevMonthDays = createMemo(() => {
    const days: Date[] = [];
    const prevMonth = new Date(currentDate());
    prevMonth.setMonth(prevMonth.getMonth() - 1);
    
    const firstDayOfPrevMonth = new Date(prevMonth.getFullYear(), prevMonth.getMonth(), 1);
    const dayOfWeek = firstDayOfPrevMonth.getDay();
    const start = new Date(firstDayOfPrevMonth);
    start.setDate(start.getDate() - dayOfWeek);
    
    for (let i = 0; i < 42; i++) {
      const day = new Date(start);
      day.setDate(start.getDate() + i);
      days.push(day);
    }
    
    return days;
  });

  // Get next month calendar days for seamless scrolling
  const nextMonthDays = createMemo(() => {
    const days: Date[] = [];
    const nextMonth = new Date(currentDate());
    nextMonth.setMonth(nextMonth.getMonth() + 1);
    
    const firstDayOfNextMonth = new Date(nextMonth.getFullYear(), nextMonth.getMonth(), 1);
    const dayOfWeek = firstDayOfNextMonth.getDay();
    const start = new Date(firstDayOfNextMonth);
    start.setDate(start.getDate() - dayOfWeek);
    
    for (let i = 0; i < 42; i++) {
      const day = new Date(start);
      day.setDate(start.getDate() + i);
      days.push(day);
    }
    
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

  // Check if date is in previous month
  const isPrevMonth = (date: Date) => {
    const prev = new Date(currentDate());
    prev.setMonth(prev.getMonth() - 1);
    return date.getMonth() === prev.getMonth() && 
           date.getFullYear() === prev.getFullYear();
  };

  // Check if date is in next month
  const isNextMonth = (date: Date) => {
    const next = new Date(currentDate());
    next.setMonth(next.getMonth() + 1);
    return date.getMonth() === next.getMonth() && 
           date.getFullYear() === next.getFullYear();
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

  // Calendar swipe handling for month navigation
  const handleCalendarTouchStart = (e: TouchEvent) => {
    setCalendarStartX(e.touches[0].clientX);
    setCalendarSwiping(true);
    setCalendarSwipeOffset(0);
  };

  const handleCalendarTouchMove = (e: TouchEvent) => {
    if (!calendarSwiping()) return;
    
    const currentX = e.touches[0].clientX;
    const diffX = currentX - calendarStartX();
    
    // Allow full swipe width for seamless transition
    const containerWidth = 300; // Approximate calendar width
    const limitedOffset = Math.max(-containerWidth, Math.min(containerWidth, diffX));
    setCalendarSwipeOffset(limitedOffset);
  };

  const handleCalendarTouchEnd = (e: TouchEvent) => {
    if (!calendarSwiping()) return;
    
    const endX = e.changedTouches[0].clientX;
    const diffX = calendarStartX() - endX;
    
    setCalendarSwiping(false);
    setCalendarAnimating(true);
    
    // Threshold for month change - 30% of screen width
    const threshold = window.innerWidth * 0.3;
    
    if (Math.abs(diffX) > threshold) {
      if (diffX > 0) {
        // Swipe left - next month
        navigateMonth('next');
        setCalendarSwipeOffset(0);
      } else {
        // Swipe right - previous month
        navigateMonth('prev');
        setCalendarSwipeOffset(0);
      }
    } else {
      // Animate back to center position smoothly
      setCalendarSwipeOffset(0);
    }
    
    // Turn off animation after transition completes
    setTimeout(() => {
      setCalendarAnimating(false);
    }, 250);
  };

  const handleEventClick = (event: CalendarEvent) => {
    setSelectedEvent(event);
    setShowEventDetail(true);
  };

  const handleDateClick = (date: Date) => {
    setSelectedDate(date);
    
    // Auto-navigate to the month of the clicked date if it's different
    const clickedMonth = date.getMonth();
    const clickedYear = date.getFullYear();
    const currentMonth = currentDate().getMonth();
    const currentYear = currentDate().getFullYear();
    
    if (clickedMonth !== currentMonth || clickedYear !== currentYear) {
      setCurrentDate(new Date(clickedYear, clickedMonth, 1));
    }
    
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
          <div class="responsive-container">
      {/* Mobile Layout */}
      <Show when={isMobile()}>
        {/* Compact Calendar - Top 40% */}
        <div class="flex-none bg-white h-[40vh] overflow-hidden">
          <div class="h-full flex flex-col">
            {/* Compact Header */}
            <div class="flex items-center justify-between px-3 py-1.5 border-b border-gray-200">
              <button
                onClick={() => navigateMonth('prev')}
                class="nav-button-compact"
              >
                <svg class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
                </svg>
              </button>
              
              <button
                class="text-sm font-semibold text-gray-900 hover:text-blue-600 transition-colors"
                onClick={() => setShowMonthPicker(true)}
              >
                {monthNames[currentDate().getMonth()]} {currentDate().getFullYear()}
              </button>
              
              <button
                onClick={() => navigateMonth('next')}
                class="nav-button-compact"
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
                class="flex-1 px-1 pb-1"
                onTouchStart={handleCalendarTouchStart}
                onTouchMove={handleCalendarTouchMove}
                onTouchEnd={handleCalendarTouchEnd}
              >
                {/* Compact Day Headers */}
                <div class="grid grid-cols-7 gap-0.5 mb-0.5">
                  <For each={dayNamesShort}>
                    {(day) => (
                      <div class="py-0.5 px-1 text-center text-xs font-medium text-gray-600">
                        {day}
                      </div>
                    )}
                  </For>
                </div>

                {/* Seamless Three-Month Calendar Container */}
                <div class="relative overflow-hidden">
                  <div 
                    class={`flex ${calendarAnimating() ? 'transition-transform duration-[250ms] ease-out' : ''}`}
                    style={`transform: translateX(calc(-100% + ${calendarSwipeOffset()}px))`}
                  >
                    {/* Previous Month */}
                    <div class="w-full flex-shrink-0">
                      <div class="grid grid-cols-7 gap-0.5">
                        <For each={prevMonthDays()}>
                          {(day) => {
                            const isCurrentMonthDay = isPrevMonth(day);
                            const isTodayDay = isToday(day);
                            const isSelectedDay = selectedDate()?.toDateString() === day.toDateString();
                            const activities = getActivityDots(day);
                            
                            return (
                              <div 
                                class={`h-10 flex flex-col items-center justify-center text-center cursor-pointer rounded-lg transition-colors ${
                                  isSelectedDay ? 'bg-blue-100 border-2 border-blue-500' :
                                  !isCurrentMonthDay ? 'text-gray-300' : 'hover:bg-gray-50'
                                }`}
                                onClick={() => handleDateClick(day)}
                              >
                                <div class={`text-xs font-medium flex items-center justify-center w-6 h-6 ${
                                  isTodayDay && !isSelectedDay ? 'bg-gray-900 text-white rounded-full' :
                                  isTodayDay && isSelectedDay ? 'bg-blue-700 text-white rounded-full' :
                                  isSelectedDay ? 'text-blue-700' :
                                  !isCurrentMonthDay ? 'text-gray-300' : 'text-gray-900'
                                }`}>
                                  {day.getDate()}
                                </div>
                                
                                <div class="flex space-x-0.5 mt-0">
                                  <Show when={activities.watering}>
                                    <div class="activity-dot bg-blue-500"></div>
                                  </Show>
                                  <Show when={activities.fertilizing}>
                                    <div class="activity-dot bg-green-500"></div>
                                  </Show>
                                  <Show when={activities.note}>
                                    <div class="activity-dot bg-gray-400"></div>
                                  </Show>
                                  <Show when={activities.customMetric}>
                                    <div class="activity-dot bg-purple-500"></div>
                                  </Show>
                                </div>
                              </div>
                            );
                          }}
                        </For>
                      </div>
                    </div>

                    {/* Current Month */}
                    <div class="w-full flex-shrink-0">
                      <div class="grid grid-cols-7 gap-0.5">
                        <For each={calendarDays()}>
                          {(day) => {
                            const isCurrentMonthDay = isCurrentMonth(day);
                            const isTodayDay = isToday(day);
                            const isSelectedDay = selectedDate()?.toDateString() === day.toDateString();
                            const activities = getActivityDots(day);
                            
                            return (
                              <div 
                                class={`h-10 flex flex-col items-center justify-center text-center cursor-pointer rounded-lg transition-colors ${
                                  isSelectedDay ? 'bg-blue-100 border-2 border-blue-500' :
                                  !isCurrentMonthDay ? 'text-gray-300' : 'hover:bg-gray-50'
                                }`}
                                onClick={() => handleDateClick(day)}
                              >
                                <div class={`text-xs font-medium flex items-center justify-center w-6 h-6 ${
                                  isTodayDay && !isSelectedDay ? 'bg-gray-900 text-white rounded-full' :
                                  isTodayDay && isSelectedDay ? 'bg-blue-700 text-white rounded-full' :
                                  isSelectedDay ? 'text-blue-700' :
                                  !isCurrentMonthDay ? 'text-gray-300' : 'text-gray-900'
                                }`}>
                                  {day.getDate()}
                                </div>
                                
                                <div class="flex space-x-0.5 mt-0">
                                  <Show when={activities.watering}>
                                    <div class="activity-dot bg-blue-500"></div>
                                  </Show>
                                  <Show when={activities.fertilizing}>
                                    <div class="activity-dot bg-green-500"></div>
                                  </Show>
                                  <Show when={activities.note}>
                                    <div class="activity-dot bg-gray-400"></div>
                                  </Show>
                                  <Show when={activities.customMetric}>
                                    <div class="activity-dot bg-purple-500"></div>
                                  </Show>
                                </div>
                              </div>
                            );
                          }}
                        </For>
                      </div>
                    </div>

                    {/* Next Month */}
                    <div class="w-full flex-shrink-0">
                      <div class="grid grid-cols-7 gap-0.5">
                        <For each={nextMonthDays()}>
                          {(day) => {
                            const isCurrentMonthDay = isNextMonth(day);
                            const isTodayDay = isToday(day);
                            const isSelectedDay = selectedDate()?.toDateString() === day.toDateString();
                            const activities = getActivityDots(day);
                            
                            return (
                              <div 
                                class={`h-10 flex flex-col items-center justify-center text-center cursor-pointer rounded-lg transition-colors ${
                                  isSelectedDay ? 'bg-blue-100 border-2 border-blue-500' :
                                  !isCurrentMonthDay ? 'text-gray-300' : 'hover:bg-gray-50'
                                }`}
                                onClick={() => handleDateClick(day)}
                              >
                                <div class={`text-xs font-medium flex items-center justify-center w-6 h-6 ${
                                  isTodayDay && !isSelectedDay ? 'bg-gray-900 text-white rounded-full' :
                                  isTodayDay && isSelectedDay ? 'bg-blue-700 text-white rounded-full' :
                                  isSelectedDay ? 'text-blue-700' :
                                  !isCurrentMonthDay ? 'text-gray-300' : 'text-gray-900'
                                }`}>
                                  {day.getDate()}
                                </div>
                                
                                <div class="flex space-x-0.5 mt-0">
                                  <Show when={activities.watering}>
                                    <div class="activity-dot bg-blue-500"></div>
                                  </Show>
                                  <Show when={activities.fertilizing}>
                                    <div class="activity-dot bg-green-500"></div>
                                  </Show>
                                  <Show when={activities.note}>
                                    <div class="activity-dot bg-gray-400"></div>
                                  </Show>
                                  <Show when={activities.customMetric}>
                                    <div class="activity-dot bg-purple-500"></div>
                                  </Show>
                                </div>
                              </div>
                            );
                          }}
                        </For>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </Show>
          </div>
        </div>
        
        {/* Day Events - Bottom 3/5 */}
        <div class="flex-1 bg-gray-50 overflow-auto">
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
          class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
          onClick={() => setShowMonthPicker(false)}
        >
          <div 
            class="bg-white rounded-lg shadow-xl max-w-sm w-full"
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
                class="nav-button-compact hover:bg-gray-100"
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
                class="nav-button-compact hover:bg-gray-100"
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
              class="px-3 py-1 text-sm font-medium text-blue-600 hover:text-blue-700 border border-blue-300 rounded-md hover:bg-blue-50 transition-colors"
            >
              Today
            </button>
          </div>

          <div class="flex items-center space-x-2">
            <button
              onClick={() => navigateMonth('prev')}
              class="nav-button"
            >
              <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <button
              onClick={() => navigateMonth('next')}
              class="nav-button"
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
          {/* Calendar Grid - Fixed height to prevent overflow */}
          <div class="p-6 overflow-hidden">
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

            {/* Calendar Days - Fixed grid height */}
            <div class="grid grid-cols-7 gap-px bg-gray-200 rounded-lg overflow-hidden" style="height: calc(6 * 6rem);">
              <For each={calendarDays()}>
                {(day) => {
                  const dayEvents = getEventsForDate(day);
                  const isCurrentMonthDay = isCurrentMonth(day);
                  const isTodayDay = isToday(day);
                  
                  return (
                    <div 
                      class={`calendar-cell ${
                        !isCurrentMonthDay ? 'calendar-cell-disabled' : ''
                      }`}
                      onClick={() => handleDateClick(day)}
                    >
                      <div class={isTodayDay ? 'calendar-date-today' : 'calendar-date'}>
                        {day.getDate()}
                      </div>
                      
                      <div class="space-y-1 overflow-hidden">
                        <For each={dayEvents.slice(0, 3)}>
                          {(event) => (
                            <div 
                              class={`calendar-event ${getEventColor(event.type)}`}
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
                          <div class="text-xs text-gray-500 pl-1 truncate">
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