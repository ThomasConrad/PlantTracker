import { Component, Show, For, createMemo, onMount, onCleanup } from "solid-js";
import type { Plant } from "@/types";
import type { components } from "@/types/api-generated";
import {
  makeTransformScheduler,
  VelocityTracker,
  translateX,
  SNAP_TRANSITION,
  FLING_THRESHOLD,
  SWIPE_FRACTION,
  detectDirection,
} from "@/utils/touch";

type TrackingEntry = components["schemas"]["TrackingEntry"];

interface CalendarEvent {
  id: string;
  title: string;
  plant: Plant;
  entry: TrackingEntry;
  date: Date;
  type: "care" | "measurement" | "note" | "photo";
}

interface MobileDayEventsProps {
  selectedDate: Date;
  events: CalendarEvent[];
  onEventClick: (event: CalendarEvent) => void;
  onDateChange: (date: Date) => void;
}

export const MobileDayEvents: Component<MobileDayEventsProps> = (props) => {
  let scrollRef: HTMLDivElement | undefined;

  // Group events by 10-minute intervals
  const eventsByTime = createMemo(() => {
    const grouped: { [key: string]: CalendarEvent[] } = {};

    props.events.forEach((event) => {
      const eventDate = new Date(event.entry.timestamp);
      const hours = eventDate.getHours();
      const minutes = eventDate.getMinutes();
      const roundedMinutes = Math.floor(minutes / 10) * 10;
      const timeKey = `${hours.toString().padStart(2, "0")}:${roundedMinutes.toString().padStart(2, "0")}`;

      if (!grouped[timeKey]) grouped[timeKey] = [];
      grouped[timeKey].push(event);
    });

    const sortedTimes = Object.keys(grouped).sort();
    return sortedTimes.map((time) => ({
      time,
      events: grouped[time].sort(
        (a, b) =>
          new Date(a.entry.timestamp).getTime() -
          new Date(b.entry.timestamp).getTime(),
      ),
    }));
  });

  const getEventIcon = (type: string) => {
    switch (type) {
      case "care":
        return "✅";
      case "measurement":
        return "📊";
      case "note":
        return "📝";
      case "photo":
        return "📷";
      default:
        return "📝";
    }
  };

  const getEventColor = (type: string) => {
    switch (type) {
      case "care":
        return "border-l-blue-500 bg-blue-50";
      case "measurement":
        return "border-l-purple-500 bg-purple-50";
      case "note":
        return "border-l-gray-500 bg-gray-50";
      case "photo":
        return "border-l-indigo-500 bg-indigo-50";
      default:
        return "border-l-gray-500 bg-gray-50";
    }
  };

  // ---- 3-panel horizontal day swipe ----
  onMount(() => {
    if (!scrollRef) return;
    const el = scrollRef;

    const velocity = new VelocityTracker();
    let tracking = false;
    let direction: "horizontal" | "vertical" | null = null;
    let startX = 0;
    let startY = 0;
    let currentX = 0;
    let containerW = 0;
    let track: HTMLDivElement | null = null;

    const scheduleTrackX = makeTransformScheduler((x) => {
      if (track) translateX(track, x);
    });

    function getAdjacentDate(offset: number): Date {
      const d = new Date(props.selectedDate);
      d.setDate(d.getDate() + offset);
      return d;
    }

    function createSwipeTrack() {
      removeSwipeTrack();
      containerW = el.offsetWidth;
      const scrollRect = el.getBoundingClientRect();

      track = document.createElement("div");
      track.className = "mc-agenda-track";
      track.style.width = containerW * 3 + "px";
      track.style.height = scrollRect.height + "px";
      track.style.position = "fixed";
      track.style.left = scrollRect.left + "px";
      track.style.top = scrollRect.top + "px";
      track.style.zIndex = "30";
      track.style.overflow = "hidden";
      translateX(track, -containerW);

      // Create 3 panels (prev, current, next) with placeholder content
      for (let i = -1; i <= 1; i++) {
        const panel = document.createElement("div");
        panel.className = "mc-agenda-panel";
        panel.style.width = containerW + "px";
        panel.innerHTML =
          i === 0
            ? el.innerHTML
            : '<div class="p-3 text-center text-gray-400 text-sm">Loading...</div>';
        track.appendChild(panel);
      }

      el.style.visibility = "hidden";
      document.body.appendChild(track);
    }

    function removeSwipeTrack() {
      if (track && track.parentNode) {
        track.parentNode.removeChild(track);
      }
      track = null;
      el.style.visibility = "";
    }

    const onTouchStart = (e: TouchEvent) => {
      startX = e.touches[0].clientX;
      startY = e.touches[0].clientY;
      currentX = startX;
      velocity.start(startX);
      tracking = true;
      direction = null;
    };

    const onTouchMove = (e: TouchEvent) => {
      if (!tracking) return;
      const x = e.touches[0].clientX;
      const y = e.touches[0].clientY;
      const dx = x - startX;
      const dy = y - startY;

      if (!direction) {
        direction = detectDirection(dx, dy, 10);
      }

      if (direction === "horizontal") {
        e.preventDefault();
        e.stopPropagation();
        velocity.update(x);
        currentX = x;

        if (!track) {
          createSwipeTrack();
        }
        if (track) {
          (track as HTMLDivElement).style.transition = "none";
          translateX(track, -containerW + dx);
        }

        if (track) {
          scheduleTrackX(-containerW + dx);
        }
      }
    };

    const onTouchEnd = () => {
      if (!tracking) return;
      tracking = false;
      if (direction !== "horizontal" || !track) {
        direction = null;
        return;
      }

      const dx = currentX - startX;
      const v = velocity.velocity;

      let swipeDir: -1 | 0 | 1 = 0;
      if (v > FLING_THRESHOLD || dx > containerW * SWIPE_FRACTION)
        swipeDir = -1; // prev day
      else if (v < -FLING_THRESHOLD || dx < -containerW * SWIPE_FRACTION)
        swipeDir = 1; // next day

      const targetX = -containerW + -swipeDir * containerW;
      track.style.transition = SNAP_TRANSITION;
      translateX(track, targetX);

      if (swipeDir !== 0) {
        // Commit day change after animation starts
        const newDate = getAdjacentDate(swipeDir);
        const animTrack = track;
        track = null;
        el.style.visibility = "";

        const cleanup = () => {
          animTrack.removeEventListener("transitionend", cleanup);
          if (animTrack.parentNode) animTrack.parentNode.removeChild(animTrack);
        };
        animTrack.addEventListener("transitionend", cleanup);
        setTimeout(cleanup, 400);

        props.onDateChange(newDate);
      } else {
        // Snap back
        const snapTrack = track;
        const cleanup = () => {
          snapTrack.removeEventListener("transitionend", cleanup);
          removeSwipeTrack();
        };
        snapTrack.addEventListener("transitionend", cleanup);
        setTimeout(cleanup, 400);
      }

      direction = null;
    };

    const onTouchCancel = () => {
      tracking = false;
      direction = null;
      scheduleTrackX.cancel();
      removeSwipeTrack();
    };

    el.addEventListener("touchstart", onTouchStart, { passive: true });
    el.addEventListener("touchmove", onTouchMove, { passive: false });
    el.addEventListener("touchend", onTouchEnd);
    el.addEventListener("touchcancel", onTouchCancel);

    onCleanup(() => {
      el.removeEventListener("touchstart", onTouchStart);
      el.removeEventListener("touchmove", onTouchMove);
      el.removeEventListener("touchend", onTouchEnd);
      el.removeEventListener("touchcancel", onTouchCancel);
      removeSwipeTrack();
    });
  });

  return (
    <div class="mc-agenda-scroll" ref={scrollRef}>
      <Show
        when={props.events.length > 0}
        fallback={
          <div class="mc-empty">
            <div class="mc-empty-title">No activities</div>
            <div class="mc-empty-sub">
              No plant care activities recorded for this day.
            </div>
          </div>
        }
      >
        <div class="mc-agenda-content">
          <For each={eventsByTime()}>
            {({ time, events }) => (
              <div class="mc-agenda-hour">
                <div class="mc-agenda-tick">
                  <span class="mc-agenda-tick-label">{time}</span>
                  <span class="mc-agenda-tick-line"></span>
                </div>

                <For each={events}>
                  {(event) => (
                    <div
                      class={`mc-event-card border-l-4 ${getEventColor(event.type)}`}
                      onClick={() => props.onEventClick(event)}
                    >
                      <div class="mc-event-icon">
                        {getEventIcon(event.type)}
                      </div>
                      <div class="mc-event-info">
                        <div class="mc-event-title">{event.plant.name}</div>
                        <div class="mc-event-subtitle">
                          {event.type === "measurement"
                            ? "Measurement"
                            : event.type}
                          <Show
                            when={
                              event.entry.measurements &&
                              event.entry.measurements.length > 0
                            }
                          >
                            <span class="font-medium ml-1">
                              {String(event.entry.measurements![0].value)}
                            </span>
                          </Show>
                        </div>
                        <Show when={event.entry.notes}>
                          <div class="mc-event-notes">
                            {String(event.entry.notes)}
                          </div>
                        </Show>
                      </div>
                      <div class="mc-event-chevron">
                        <svg
                          class="h-4 w-4 text-gray-400"
                          fill="none"
                          viewBox="0 0 24 24"
                          stroke="currentColor"
                        >
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width={2}
                            d="M9 5l7 7-7 7"
                          />
                        </svg>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};
