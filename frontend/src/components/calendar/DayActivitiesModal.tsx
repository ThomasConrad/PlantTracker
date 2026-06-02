import { Component, Show, For } from "solid-js";
import type { Plant } from "@/types";
import type { components } from "@/types/api-generated";

type TrackingEntry = components["schemas"]["TrackingEntry"];

function deriveEntryType(
  entry: TrackingEntry,
): "care" | "measurement" | "photo" | "note" {
  if (entry.careTaskIds && entry.careTaskIds.length > 0) return "care";
  if (entry.measurements && entry.measurements.length > 0) return "measurement";
  if (entry.photoIds && entry.photoIds.length > 0) return "photo";
  return "note";
}

interface CalendarEvent {
  id: string;
  title: string;
  plant: Plant;
  entry: TrackingEntry;
  date: Date;
  type: "care" | "measurement" | "note" | "photo";
}

interface DayActivitiesModalProps {
  isOpen: boolean;
  onClose: () => void;
  date: Date;
  events: CalendarEvent[];
  onEventClick: (event: CalendarEvent) => void;
}

export const DayActivitiesModal: Component<DayActivitiesModalProps> = (
  props,
) => {
  const getActivityIcon = (type: string) => {
    switch (type) {
      case "care":
        return (
          <div class="flex-shrink-0 w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
            <svg
              class="h-4 w-4 text-blue-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          </div>
        );
      case "measurement":
        return (
          <div class="flex-shrink-0 w-8 h-8 bg-purple-100 rounded-full flex items-center justify-center">
            <svg
              class="h-4 w-4 text-purple-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
              />
            </svg>
          </div>
        );
      case "note":
        return (
          <div class="flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
            <svg
              class="h-4 w-4 text-gray-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
              />
            </svg>
          </div>
        );
      case "photo":
        return (
          <div class="flex-shrink-0 w-8 h-8 bg-indigo-100 rounded-full flex items-center justify-center">
            <svg
              class="h-4 w-4 text-indigo-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"
              />
            </svg>
          </div>
        );
      default:
        return (
          <div class="flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
            <svg
              class="h-4 w-4 text-gray-600"
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
        );
    }
  };

  const getActivityTypeLabel = (type: string) => {
    switch (type) {
      case "care":
        return "Care";
      case "measurement":
        return "Measurement";
      case "note":
        return "Note";
      case "photo":
        return "Photo";
      default:
        return type;
    }
  };

  const handleBackdropClick = (e: MouseEvent) => {
    if (e.target === e.currentTarget) {
      props.onClose();
    }
  };

  const formatDateHeader = (date: Date) => {
    return date.toLocaleDateString("en-GB", {
      weekday: "long",
      year: "numeric",
      month: "long",
      day: "numeric",
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
              <svg
                class="h-6 w-6"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>

          {/* Content */}
          <div class="divide-y divide-gray-200">
            <Show when={props.events.length === 0}>
              <div class="p-8 text-center">
                <svg
                  class="mx-auto h-12 w-12 text-gray-400"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width={2}
                    d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                  />
                </svg>
                <h3 class="mt-2 text-sm font-medium text-gray-900">
                  No activities
                </h3>
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
                    {getActivityIcon(deriveEntryType(event.entry))}

                    <div class="flex-1 min-w-0">
                      <div class="flex items-center justify-between">
                        <div class="flex items-center space-x-2">
                          <h3 class="text-sm font-medium text-gray-900">
                            {getActivityTypeLabel(deriveEntryType(event.entry))}
                          </h3>
                          <span class="text-sm text-gray-500">•</span>
                          <span class="text-sm font-medium text-blue-600">
                            {event.plant.name}
                          </span>
                        </div>
                        <time class="text-xs text-gray-500">
                          {new Date(event.entry.timestamp).toLocaleTimeString(
                            "en-GB",
                            {
                              hour: "2-digit",
                              minute: "2-digit",
                            },
                          )}
                        </time>
                      </div>

                      <Show when={event.entry.notes}>
                        <p class="mt-1 text-sm text-gray-600 truncate">
                          {event.entry.notes}
                        </p>
                      </Show>

                      <Show
                        when={
                          deriveEntryType(event.entry) === "measurement" &&
                          event.entry.measurements?.length
                        }
                      >
                        <p class="mt-1 text-sm text-gray-600">
                          Value:{" "}
                          {JSON.stringify(event.entry.measurements![0].value)}
                        </p>
                      </Show>

                      <Show
                        when={
                          event.entry.photoIds &&
                          Array.isArray(event.entry.photoIds) &&
                          event.entry.photoIds.length > 0
                        }
                      >
                        <p class="mt-1 text-xs text-gray-500">
                          📷 {(event.entry.photoIds as string[]).length}{" "}
                          photo(s)
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
