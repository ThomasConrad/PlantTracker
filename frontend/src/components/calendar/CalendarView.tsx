import { Component, createSignal, Show, For, onMount, onCleanup, createMemo, createEffect } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { authStore } from '@/stores/auth';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';
import { EventDetailModal } from './EventDetailModal';
import { DayActivitiesModal } from './DayActivitiesModal';
import { MobileDayEvents } from './MobileDayEvents';
import {
  makeTransformScheduler,
  VelocityTracker,
  translateX,
  translateY,
  SNAP_TRANSITION,
  FLING_THRESHOLD,
  SWIPE_FRACTION,
  detectDirection,
} from '@/utils/touch';

type TrackingEntry = components['schemas']['TrackingEntry'];

function deriveEntryType(entry: TrackingEntry): 'care' | 'measurement' | 'photo' | 'note' {
  if (entry.careTaskIds && entry.careTaskIds.length > 0) return 'care';
  if (entry.measurements && entry.measurements.length > 0) return 'measurement';
  if (entry.photoIds && entry.photoIds.length > 0) return 'photo';
  return 'note';
}

interface CalendarEvent {
  id: string;
  title: string;
  plant: Plant;
  entry: TrackingEntry;
  date: Date;
  type: 'care' | 'measurement' | 'note' | 'photo';
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
  const [sheetOpen, setSheetOpen] = createSignal(false);
  
  const weekStartOffset = createMemo(() => (authStore.user?.firstDayOfWeek === 'monday' ? 1 : 0));

  // DOM refs for imperative touch handling
  let gridContainerRef: HTMLDivElement | undefined;
  let gridTrackRef: HTMLDivElement | undefined;
  let overlayRef: HTMLDivElement | undefined;
  let overlayScrollRef: HTMLDivElement | undefined;

  const getCalendarStartForMonth = (date: Date) => {
    const firstDay = new Date(date.getFullYear(), date.getMonth(), 1);
    const dayOfWeek = firstDay.getDay();
    const offset = (dayOfWeek - weekStartOffset() + 7) % 7;
    const start = new Date(firstDay);
    start.setDate(start.getDate() - offset);
    return start;
  };

  const firstDayOfMonth = createMemo(() => {
    const date = new Date(currentDate());
    return new Date(date.getFullYear(), date.getMonth(), 1);
  });

  const calendarStart = createMemo(() => {
    return getCalendarStartForMonth(firstDayOfMonth());
  });

  const calendarDays = createMemo(() => {
    const days: Date[] = [];
    const start = calendarStart();
    for (let i = 0; i < 42; i++) {
      const day = new Date(start);
      day.setDate(start.getDate() + i);
      days.push(day);
    }
    selectedDate(); // force reactivity
    return days;
  });

  const prevMonthDays = createMemo(() => {
    const days: Date[] = [];
    const prevMonth = new Date(currentDate());
    prevMonth.setMonth(prevMonth.getMonth() - 1);
    const start = getCalendarStartForMonth(prevMonth);
    for (let i = 0; i < 42; i++) {
      const day = new Date(start);
      day.setDate(start.getDate() + i);
      days.push(day);
    }
    return days;
  });

  const nextMonthDays = createMemo(() => {
    const days: Date[] = [];
    const nextMonth = new Date(currentDate());
    nextMonth.setMonth(nextMonth.getMonth() + 1);
    const start = getCalendarStartForMonth(nextMonth);
    for (let i = 0; i < 42; i++) {
      const day = new Date(start);
      day.setDate(start.getDate() + i);
      days.push(day);
    }
    return days;
  });

  const getEventsForDate = (date: Date) => {
    const dateStr = date.toDateString();
    return events().filter(event => event.date.toDateString() === dateStr);
  };

  const isCurrentMonth = (date: Date) => {
    const current = currentDate();
    return date.getMonth() === current.getMonth() && date.getFullYear() === current.getFullYear();
  };

  const isPrevMonth = (date: Date) => {
    const prev = new Date(currentDate());
    prev.setMonth(prev.getMonth() - 1);
    return date.getMonth() === prev.getMonth() && date.getFullYear() === prev.getFullYear();
  };

  const isNextMonth = (date: Date) => {
    const next = new Date(currentDate());
    next.setMonth(next.getMonth() + 1);
    return date.getMonth() === next.getMonth() && date.getFullYear() === next.getFullYear();
  };

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
      
      const plantsToShow = props.selectedPlants?.length 
        ? plants.filter(plant => props.selectedPlants!.includes(plant.id))
        : plants;

      for (const plant of plantsToShow) {
        try {
          const response = await plantsStore.getTrackingEntries(plant.id);
          for (const entry of response.entries) {
            const entryType = deriveEntryType(entry);
            if (props.selectedTypes?.length && !props.selectedTypes.includes(entryType)) {
              continue;
            }
            allEvents.push({
              id: entry.id,
              title: getEventTitle(entry, plant),
              plant,
              entry,
              date: new Date(entry.timestamp),
              type: entryType
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
    switch (deriveEntryType(entry)) {
      case 'care': return `✅ ${plant.name}`;
      case 'measurement': return `📊 ${plant.name}`;
      case 'note': return `📝 ${plant.name}`;
      case 'photo': return `📷 ${plant.name}`;
      default: return plant.name;
    }
  };

  const getEventColor = (type: string) => {
    switch (type) {
      case 'care': return 'bg-blue-100 text-blue-800 border-blue-200';
      case 'measurement': return 'bg-purple-100 text-purple-800 border-purple-200';
      case 'note': return 'bg-gray-100 text-gray-800 border-gray-200';
      case 'photo': return 'bg-indigo-100 text-indigo-800 border-indigo-200';
      default: return 'bg-gray-100 text-gray-800 border-gray-200';
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

  const handleEventClick = (event: CalendarEvent) => {
    setSelectedEvent(event);
    setShowEventDetail(true);
  };

  const handleDateClick = (date: Date) => {
    setSelectedDate(date);
    
    const clickedMonth = date.getMonth();
    const clickedYear = date.getFullYear();
    const currentMonth = currentDate().getMonth();
    const currentYear = currentDate().getFullYear();
    
    if (clickedMonth !== currentMonth || clickedYear !== currentYear) {
      setCurrentDate(new Date(clickedYear, clickedMonth, 1));
    }
    
    if (isMobile()) {
      return;
    }
    
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

  // ---- Imperative touch setup for mobile ----
  const setupMobileGestures = () => {
    if (!gridContainerRef || !gridTrackRef || !overlayRef || !overlayScrollRef) return;

    const container = gridContainerRef;
    const track = gridTrackRef;
    const overlay = overlayRef;
    const overlayScroll = overlayScrollRef;

    // ======== CALENDAR GRID SWIPE (3-panel rotation) ========
    const gridVelocity = new VelocityTracker();
    const scheduleTrackX = makeTransformScheduler(x => translateX(track, x));
    let gridDragging = false;
    let gridSwiping = false;
    let gridDirection: 'horizontal' | 'vertical' | null = null;
    let gridStartX = 0;
    let gridStartY = 0;
    let gridCurrentX = 0;
    let gridContainerW = 0;
    let gridAnimating = false;

    function resetTrackPosition() {
      gridContainerW = container.offsetWidth;
      const panels = track.children as HTMLCollectionOf<HTMLElement>;
      for (let i = 0; i < panels.length; i++) {
        panels[i].style.width = gridContainerW + 'px';
      }
      track.style.width = (gridContainerW * 3) + 'px';
      track.style.transition = 'none';
      translateX(track, -gridContainerW);
    }

    function commitGridSwipe(swipeDir: -1 | 1) {
      // Change month
      const current = currentDate();
      const newDate = new Date(current);
      newDate.setMonth(current.getMonth() + swipeDir);
      setCurrentDate(newDate);
    }

    const onGridTouchStart = (e: TouchEvent) => {
      if (gridAnimating) {
        gridAnimating = false;
        track.style.transition = 'none';
        translateX(track, -container.offsetWidth);
      }
      gridStartX = e.touches[0].clientX;
      gridStartY = e.touches[0].clientY;
      gridCurrentX = gridStartX;
      gridVelocity.start(gridStartX);
      gridDragging = true;
      gridSwiping = false;
      gridDirection = null;
      gridContainerW = container.offsetWidth;
    };

    const onGridTouchMove = (e: TouchEvent) => {
      if (!gridDragging) return;
      gridCurrentX = e.touches[0].clientX;
      const dx = gridCurrentX - gridStartX;
      const dy = e.touches[0].clientY - gridStartY;

      if (!gridDirection) {
        gridDirection = detectDirection(dx, dy);
      }
      if (gridDirection === 'vertical') return;
      if (gridDirection !== 'horizontal') return;

      e.preventDefault();
      gridVelocity.update(gridCurrentX);

      if (!gridSwiping) {
        gridSwiping = true;
        track.style.transition = 'none';
      }
      scheduleTrackX(-gridContainerW + dx);
    };

    const onGridTouchEnd = () => {
      if (!gridDragging) return;
      gridDragging = false;
      scheduleTrackX.cancel();
      if (!gridSwiping) return;
      gridSwiping = false;

      const dx = gridCurrentX - gridStartX;
      const velocity = gridVelocity.velocity;

      let swipeDir: -1 | 0 | 1 = 0;
      if (velocity > FLING_THRESHOLD || dx > gridContainerW * SWIPE_FRACTION) swipeDir = -1;
      else if (velocity < -FLING_THRESHOLD || dx < -gridContainerW * SWIPE_FRACTION) swipeDir = 1;

      if (swipeDir === 0) {
        // Snap back
        gridAnimating = true;
        track.style.transition = SNAP_TRANSITION;
        translateX(track, -gridContainerW);
        const onEnd = () => {
          track.removeEventListener('transitionend', onEnd);
          if (gridAnimating) { gridAnimating = false; track.style.transition = 'none'; }
        };
        track.addEventListener('transitionend', onEnd);
        setTimeout(onEnd, 400);
        return;
      }

      // Commit month change, then animate from current visual offset to center
      const preRotateX = -gridContainerW + dx;
      commitGridSwipe(swipeDir);

      // After SolidJS re-renders the panels, we need to position correctly.
      // Since we use reactive <For>, the panels will re-render. We set the
      // track to show the "arrival" position via animation.
      requestAnimationFrame(() => {
        resetTrackPosition();
        // Start from the offset that visually continues the swipe
        translateX(track, preRotateX + (swipeDir * gridContainerW));
        track.offsetWidth; // force reflow
        gridAnimating = true;
        track.style.transition = SNAP_TRANSITION;
        translateX(track, -gridContainerW);
        const onEnd = () => {
          track.removeEventListener('transitionend', onEnd);
          if (gridAnimating) { gridAnimating = false; track.style.transition = 'none'; }
        };
        track.addEventListener('transitionend', onEnd);
        setTimeout(onEnd, 400);
      });

      updateOverlayPosition();
    };

    const onGridTouchCancel = () => {
      gridDragging = false;
      gridSwiping = false;
      gridDirection = null;
      scheduleTrackX.cancel();
      track.style.transition = 'none';
      translateX(track, -gridContainerW);
    };

    container.addEventListener('touchstart', onGridTouchStart, { passive: true });
    container.addEventListener('touchmove', onGridTouchMove, { passive: false });
    container.addEventListener('touchend', onGridTouchEnd);
    container.addEventListener('touchcancel', onGridTouchCancel);

    // ======== SLIDE-UP OVERLAY PANEL ========
    const HEADER_HEIGHT = 56; // px from top for full-open position
    const scheduleOverlayY = makeTransformScheduler(y => {
      const viewportH = window.visualViewport?.height || window.innerHeight;
      overlay.style.height = Math.max(120, viewportH - HEADER_HEIGHT) + 'px';
      translateY(overlay, y);
    });

    function getGridBottom() {
      return container.getBoundingClientRect().bottom;
    }

    function getMidTop() {
      return getGridBottom();
    }

    function getFullTop() {
      return HEADER_HEIGHT;
    }

    function clampOverlayTop(y: number) {
      return Math.max(getFullTop(), Math.min(getMidTop(), y));
    }

    function updateOverlayPosition() {
      const target = sheetOpen() ? getFullTop() : getMidTop();
      overlay.style.transition = SNAP_TRANSITION;
      const viewportH = window.visualViewport?.height || window.innerHeight;
      overlay.style.height = Math.max(120, viewportH - HEADER_HEIGHT) + 'px';
      translateY(overlay, target);
    }

    function getLiveOverlayTop() {
      return overlay.getBoundingClientRect().top;
    }

    // Vertical drag on overlay
    const overlayVelocity = new VelocityTracker();
    let overlayDragging = false;
    let overlayPending = false;
    let overlayDirection: 'horizontal' | 'vertical' | null = null;
    let overlayStartY = 0;
    let overlayStartX = 0;
    let overlayStartTop = 0;
    let overlayLatestTop = 0;

    function snapOverlay(velocity: number, currentTop: number) {
      overlay.style.transition = SNAP_TRANSITION;
      const midTop = getMidTop();
      const fullTop = getFullTop();
      const midPoint = (fullTop + midTop) / 2;

      let snapTo: number;
      if (velocity < -FLING_THRESHOLD) {
        snapTo = fullTop;
      } else if (velocity > FLING_THRESHOLD) {
        snapTo = midTop;
      } else {
        snapTo = currentTop < midPoint ? fullTop : midTop;
      }

      const viewportH = window.visualViewport?.height || window.innerHeight;
      overlay.style.height = Math.max(120, viewportH - HEADER_HEIGHT) + 'px';
      translateY(overlay, snapTo);
      setSheetOpen(snapTo <= fullTop);
    }

    const onOverlayTouchStart = (e: TouchEvent) => {
      scheduleOverlayY.cancel();
      overlayStartX = e.touches[0].clientX;
      overlayStartY = e.touches[0].clientY;
      overlayStartTop = getLiveOverlayTop();
      overlayLatestTop = overlayStartTop;
      overlayVelocity.start(overlayStartY);
      overlayDragging = false;
      overlayPending = true;
      overlayDirection = null;
    };

    const onOverlayTouchMove = (e: TouchEvent) => {
      if (!overlayPending) return;
      const x = e.touches[0].clientX;
      const y = e.touches[0].clientY;
      const dx = x - overlayStartX;
      const dy = y - overlayStartY;

      if (!overlayDirection) {
        overlayDirection = detectDirection(dx, dy);
      }
      if (overlayDirection === 'horizontal') return;
      if (overlayDirection !== 'vertical') return;

      // Allow native scroll inside the agenda list when sheet is fully open
      if (!overlayDragging && (e.target as HTMLElement)?.closest('.mc-agenda-scroll')) {
        const scrollEl = overlayScroll;
        const maxScroll = scrollEl.scrollHeight - scrollEl.clientHeight;
        const atTop = getLiveOverlayTop() <= getFullTop() + 2;
        const canScrollUp = scrollEl.scrollTop > 0 && dy > 0;
        const canScrollDown = scrollEl.scrollTop < maxScroll && dy < 0;
        if (atTop && maxScroll > 1 && (canScrollUp || canScrollDown)) {
          // Let native scroll handle it
          overlayStartY = y;
          overlayStartTop = getLiveOverlayTop();
          overlayLatestTop = overlayStartTop;
          overlayVelocity.start(y);
          return;
        }
        if (!atTop || dy > 0) scrollEl.scrollTop = 0;
      }

      if (!overlayDragging) {
        overlayDragging = true;
        overlay.style.transition = 'none';
        overlayStartY = y;
        overlayStartTop = getLiveOverlayTop();
        overlayLatestTop = overlayStartTop;
        overlayVelocity.start(y);
      }

      e.preventDefault();
      overlayVelocity.update(y);
      overlayLatestTop = clampOverlayTop(overlayStartTop + (y - overlayStartY));
      scheduleOverlayY(overlayStartTop + (y - overlayStartY));
    };

    const onOverlayTouchEnd = () => {
      if (!overlayPending) return;
      overlayPending = false;
      if (!overlayDragging) return;
      overlayDragging = false;
      scheduleOverlayY.cancel();
      const viewportH = window.visualViewport?.height || window.innerHeight;
      overlay.style.height = Math.max(120, viewportH - HEADER_HEIGHT) + 'px';
      translateY(overlay, overlayLatestTop);
      snapOverlay(overlayVelocity.velocity, overlayLatestTop);
    };

    overlay.addEventListener('touchstart', onOverlayTouchStart, { passive: true });
    overlay.addEventListener('touchmove', onOverlayTouchMove, { passive: false });
    overlay.addEventListener('touchend', onOverlayTouchEnd);
    overlay.addEventListener('touchcancel', onOverlayTouchEnd);

    // Also allow dragging from the calendar grid area to move the overlay
    const onGridOverlayTouchStart = (e: TouchEvent) => {
      // Reuse overlay drag logic but don't interfere with horizontal grid swipe
      scheduleOverlayY.cancel();
      overlayStartX = e.touches[0].clientX;
      overlayStartY = e.touches[0].clientY;
      overlayStartTop = getLiveOverlayTop();
      overlayLatestTop = overlayStartTop;
      overlayVelocity.start(overlayStartY);
      overlayDragging = false;
      overlayPending = true;
      overlayDirection = null;
    };

    // We bind this to the weekdays row (just above grid) for a secondary drag surface
    const weekdaysEl = container.previousElementSibling as HTMLElement | null;
    if (weekdaysEl) {
      weekdaysEl.addEventListener('touchstart', onGridOverlayTouchStart, { passive: true });
      weekdaysEl.addEventListener('touchmove', onOverlayTouchMove, { passive: false });
      weekdaysEl.addEventListener('touchend', onOverlayTouchEnd);
      weekdaysEl.addEventListener('touchcancel', onOverlayTouchEnd);
    }

    // Initial position (no animation)
    requestAnimationFrame(() => {
      const midTop = getMidTop();
      const viewportH = window.visualViewport?.height || window.innerHeight;
      overlay.style.height = Math.max(120, viewportH - HEADER_HEIGHT) + 'px';
      translateY(overlay, midTop);
      // Enable transitions after initial paint
      requestAnimationFrame(() => {
        overlay.style.transition = SNAP_TRANSITION;
      });
    });

    // Reset track panel widths on first paint
    requestAnimationFrame(resetTrackPosition);

    // Cleanup function
    return () => {
      container.removeEventListener('touchstart', onGridTouchStart);
      container.removeEventListener('touchmove', onGridTouchMove);
      container.removeEventListener('touchend', onGridTouchEnd);
      container.removeEventListener('touchcancel', onGridTouchCancel);
      overlay.removeEventListener('touchstart', onOverlayTouchStart);
      overlay.removeEventListener('touchmove', onOverlayTouchMove);
      overlay.removeEventListener('touchend', onOverlayTouchEnd);
      overlay.removeEventListener('touchcancel', onOverlayTouchEnd);
      if (weekdaysEl) {
        weekdaysEl.removeEventListener('touchstart', onGridOverlayTouchStart);
        weekdaysEl.removeEventListener('touchmove', onOverlayTouchMove);
        weekdaysEl.removeEventListener('touchend', onOverlayTouchEnd);
        weekdaysEl.removeEventListener('touchcancel', onOverlayTouchEnd);
      }
    };
  };

  let cleanupGestures: (() => void) | undefined;

  onMount(() => {
    loadCalendarData();
    
    const checkMobile = () => {
      const isMobileDevice = window.innerWidth < 640;
      setIsMobile(isMobileDevice);
    };
    
    checkMobile();
    window.addEventListener('resize', checkMobile);
    
    // Setup mobile gestures after DOM is ready
    requestAnimationFrame(() => {
      if (isMobile()) {
        cleanupGestures = setupMobileGestures();
      }
    });

    onCleanup(() => {
      window.removeEventListener('resize', checkMobile);
      cleanupGestures?.();
    });
  });

  // Re-setup gestures when mobile state changes
  createEffect(() => {
    if (isMobile()) {
      requestAnimationFrame(() => {
        cleanupGestures?.();
        cleanupGestures = setupMobileGestures();
      });
    } else {
      cleanupGestures?.();
      cleanupGestures = undefined;
    }
  });

  // Reload when selected plants change
  createEffect(() => {
    props.selectedPlants;
    props.selectedTypes;
    loadCalendarData();
  });

  // Update overlay position when sheet state or grid content changes
  createEffect(() => {
    if (isMobile() && overlayRef && gridContainerRef) {
      currentDate(); // react to month changes
      requestAnimationFrame(() => {
        if (overlayRef) {
          const HEADER_HEIGHT = 56;
          const target = sheetOpen() ? HEADER_HEIGHT : gridContainerRef!.getBoundingClientRect().bottom;
          overlayRef.style.transition = SNAP_TRANSITION;
          const viewportH = window.visualViewport?.height || window.innerHeight;
          overlayRef.style.height = Math.max(120, viewportH - HEADER_HEIGHT) + 'px';
          translateY(overlayRef, target);
        }
      });
    }
  });

  const selectedDateEvents = createMemo(() => {
    const selected = selectedDate();
    if (!selected) return [];
    return getEventsForDate(selected);
  });

  const monthNames = [
    'January', 'February', 'March', 'April', 'May', 'June',
    'July', 'August', 'September', 'October', 'November', 'December'
  ];

  const dayNames = createMemo(() =>
    weekStartOffset() === 1
      ? ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun']
      : ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
  );
  const dayNamesShort = createMemo(() =>
    weekStartOffset() === 1 ? ['M', 'T', 'W', 'T', 'F', 'S', 'S'] : ['S', 'M', 'T', 'W', 'T', 'F', 'S']
  );

  const getActivityDots = (date: Date) => {
    const dayEvents = getEventsForDate(date);
    const activities = { care: false, measurement: false, note: false, photo: false };
    dayEvents.forEach(event => { activities[event.type] = true; });
    return activities;
  };

  const formatDateLabel = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      weekday: 'long', month: 'long', day: 'numeric', year: 'numeric'
    }).format(date);
  };

  // Render a calendar grid panel for a given set of days and "isCurrentMonth" checker
  const renderCalendarGrid = (days: Date[], isCurrentMonthFn: (d: Date) => boolean) => (
    <div class="mc-grid-panel">
      <div class="grid grid-cols-7 gap-0.5">
        <For each={days}>
          {(day) => {
            const isCurrentMonthDay = isCurrentMonthFn(day);
            const isTodayDay = isToday(day);
            const isSelectedDay = selectedDate()?.toDateString() === day.toDateString();
            const activities = getActivityDots(day);
            
            return (
              <button 
                class={`mc-day ${isSelectedDay ? 'selected' : ''} ${isTodayDay && !isSelectedDay ? 'today' : ''} ${!isCurrentMonthDay ? 'muted' : ''}`}
                onClick={() => handleDateClick(day)}
              >
                <span class="mc-day-num">
                  {day.getDate()}
                </span>
                <span class="mc-dots">
                  <Show when={activities.care}>
                    <i class="mc-dot bg-blue-500"></i>
                  </Show>
                  <Show when={activities.measurement}>
                    <i class="mc-dot bg-purple-500"></i>
                  </Show>
                  <Show when={activities.note}>
                    <i class="mc-dot bg-gray-400"></i>
                  </Show>
                  <Show when={activities.photo}>
                    <i class="mc-dot bg-indigo-500"></i>
                  </Show>
                </span>
              </button>
            );
          }}
        </For>
      </div>
    </div>
  );

  const handleOverlayHandleClick = () => {
    setSheetOpen(!sheetOpen());
    // Position update is handled by the createEffect above
  };

  return (
    <div class="responsive-container">
      {/* Mobile Layout */}
      <Show when={isMobile()}>
        <div class="mc-mobile-root">
          {/* Header */}
          <div class="mc-header">
            <span class="mc-month-title" onClick={() => setShowMonthPicker(true)}>
              {monthNames[currentDate().getMonth()]} {currentDate().getFullYear()}
            </span>
          </div>

          {/* Weekday Headers */}
          <div class="mc-weekdays">
            <For each={dayNamesShort()}>
              {(day) => <span>{day}</span>}
            </For>
          </div>

          {/* Calendar Grid Container (swipeable) */}
          <div class="mc-grid-container" ref={gridContainerRef}>
            <div class="mc-grid-track" ref={gridTrackRef}>
              {renderCalendarGrid(prevMonthDays(), isPrevMonth)}
              {renderCalendarGrid(calendarDays(), isCurrentMonth)}
              {renderCalendarGrid(nextMonthDays(), isNextMonth)}
            </div>
          </div>

          {/* Slide-up Agenda Overlay */}
          <div class="mc-agenda-overlay" ref={overlayRef}>
            <button class="mc-agenda-handle" onClick={handleOverlayHandleClick}></button>
            <div class="mc-agenda-header-row">
              <div>
                <div class="mc-agenda-title">Activities</div>
                <div class="mc-agenda-date">
                  {selectedDate() ? formatDateLabel(selectedDate()!) : ''}
                </div>
              </div>
            </div>
            <div class="mc-agenda-scroll" ref={overlayScrollRef}>
              <MobileDayEvents 
                selectedDate={selectedDate()!}
                events={selectedDateEvents()}
                onEventClick={handleEventClick}
                onDateChange={(date) => {
                  setSelectedDate(date);
                  // Navigate month if needed
                  const clickedMonth = date.getMonth();
                  const clickedYear = date.getFullYear();
                  const currentMonth = currentDate().getMonth();
                  const currentYear = currentDate().getFullYear();
                  if (clickedMonth !== currentMonth || clickedYear !== currentYear) {
                    setCurrentDate(new Date(clickedYear, clickedMonth, 1));
                  }
                }}
              />
            </div>
          </div>
        </div>
      </Show>

      {/* Month Picker Modal */}
      <Show when={showMonthPicker()}>
        <div 
          class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50"
          onClick={() => setShowMonthPicker(false)}
        >
          <div 
            class="bg-white rounded-lg shadow-xl max-w-sm w-full"
            onClick={(e) => e.stopPropagation()}
          >
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
            <button onClick={() => navigateMonth('prev')} class="nav-button">
              <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <button onClick={() => navigateMonth('next')} class="nav-button">
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
          <div class="p-6 overflow-hidden">
            <div class="grid grid-cols-7 gap-px mb-2">
              <For each={dayNames()}>
                {(day) => (
                  <div class="p-2 text-center text-sm font-medium text-gray-700">
                    {day}
                  </div>
                )}
              </For>
            </div>

            <div class="grid grid-cols-7 gap-px bg-gray-200 rounded-lg overflow-hidden" style="height: calc(6 * 6rem);">
              <For each={calendarDays()}>
                {(day) => {
                  const dayEvents = getEventsForDate(day);
                  const isCurrentMonthDay = isCurrentMonth(day);
                  const isTodayDay = isToday(day);
                  
                  return (
                    <div 
                      class={`calendar-cell ${!isCurrentMonthDay ? 'calendar-cell-disabled' : ''}`}
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
                              onClick={(e) => { e.stopPropagation(); handleEventClick(event); }}
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
          onClose={() => { setShowEventDetail(false); setSelectedEvent(null); }}
          plant={selectedEvent()!.plant}
          entry={selectedEvent()!.entry}
        />
      </Show>

      {/* Day Activities Modal */}
      <Show when={selectedDate()}>
        <DayActivitiesModal
          isOpen={showDayActivities()}
          onClose={() => { setShowDayActivities(false); setSelectedDate(null); }}
          date={selectedDate()!}
          events={getEventsForDate(selectedDate()!)}
          onEventClick={handleDayEventClick}
        />
      </Show>
    </div>
  );
};
