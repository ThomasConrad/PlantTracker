import { Component, createSignal, Show, For, onMount } from "solid-js";
import { plantsStore } from "@/stores/plants";
import { Button } from "@/components/ui/Button";
import { TrackingEntryForm } from "./TrackingEntryForm";
import type { Plant } from "@/types";
import type { components } from "@/types/api-generated";
import { formatDate } from "@/utils/date";

type TrackingEntry = components["schemas"]["TrackingEntry"];
type CreateTrackingEntryRequest =
  components["schemas"]["CreateTrackingEntryRequest"];
type CareTaskWithStatus = components["schemas"]["CareTaskWithStatus"];

interface PlantCareStatusProps {
  plant: Plant;
}

export const PlantCareStatus: Component<PlantCareStatusProps> = (props) => {
  const [recentEntries, setRecentEntries] = createSignal<TrackingEntry[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [submitting, setSubmitting] = createSignal<string | null>(null);
  const [showTrackingForm, setShowTrackingForm] = createSignal(false);

  const loadRecentEntries = async () => {
    try {
      setLoading(true);
      const response = await plantsStore.getTrackingEntries(props.plant.id);
      setRecentEntries(response.entries.slice(0, 5));
    } catch (error) {
      console.error("Failed to load recent tracking entries:", error);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadRecentEntries();
  });

  const handleLogCareTask = async (task: CareTaskWithStatus) => {
    try {
      setSubmitting(task.id);
      const payload: CreateTrackingEntryRequest = {
        careTaskIds: [task.id],
        timestamp: new Date().toISOString(),
      };
      await plantsStore.createTrackingEntry(props.plant.id, payload);
      await loadRecentEntries();
    } catch (error) {
      console.error("Failed to log care task:", error);
    } finally {
      setSubmitting(null);
    }
  };

  const openTrackingForm = () => {
    setShowTrackingForm(true);
  };

  const closeTrackingForm = () => {
    setShowTrackingForm(false);
    loadRecentEntries();
  };

  const handleTrackingSubmit = async (data: CreateTrackingEntryRequest) => {
    await plantsStore.createTrackingEntry(props.plant.id, data);
  };

  const careTasks = () => props.plant.careTasks ?? [];
  const scheduledTasks = () =>
    careTasks().filter((t) => t.intervalDays != null && !t.archivedAt);
  const unscheduledTasks = () =>
    careTasks().filter((t) => t.intervalDays == null && !t.archivedAt);

  const getTaskStatusColor = (task: CareTaskWithStatus) => {
    if (!task.isDue) return "text-green-600 bg-green-50 border-green-200";
    const overdue = (task.daysOverdue ?? 0) > 0;
    if (overdue) return "text-red-600 bg-red-50 border-red-200";
    return "text-yellow-600 bg-yellow-50 border-yellow-200";
  };

  const getTaskStatusText = (task: CareTaskWithStatus) => {
    if (task.daysOverdue == null && !task.isDue) return "No schedule";
    const days = task.daysOverdue ?? 0;
    if (days > 0) return `${days} day${days > 1 ? "s" : ""} overdue`;
    if (days === 0) return "Due today";
    const abs = Math.abs(days);
    if (abs === 1) return "Due tomorrow";
    return `Due in ${abs} days`;
  };

  const getEntryIcon = (entry: TrackingEntry) => {
    if (entry.careTaskIds && entry.careTaskIds.length > 0) {
      return (
        <svg
          class="h-4 w-4 text-blue-500"
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
      );
    }
    if (entry.measurements && entry.measurements.length > 0) {
      return (
        <svg
          class="h-4 w-4 text-purple-500"
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
      );
    }
    if (entry.photoIds && entry.photoIds.length > 0) {
      return (
        <svg
          class="h-4 w-4 text-indigo-500"
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
      );
    }
    return (
      <svg
        class="h-4 w-4 text-gray-500"
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
    );
  };

  const getEntryLabel = (entry: TrackingEntry) => {
    if (entry.careTaskIds && entry.careTaskIds.length > 0) {
      const names = entry.careTaskIds
        .map((id) => careTasks().find((t) => t.id === id)?.name)
        .filter(Boolean);
      return names.length > 0 ? names.join(", ") : "Care";
    }
    if (entry.measurements && entry.measurements.length > 0)
      return "Measurement";
    if (entry.photoIds && entry.photoIds.length > 0) return "Photo";
    return "Note";
  };

  return (
    <div class="bg-white shadow-sm rounded-xl sm:rounded-2xl border border-gray-200 overflow-hidden">
      {/* Card Header */}
      <div class="px-4 sm:px-6 py-4 sm:py-5 border-b border-gray-100 bg-gray-50/50">
        <div class="flex items-center space-x-3">
          <div class="flex-shrink-0">
            <div class="w-8 h-8 sm:w-10 sm:h-10 bg-primary-100 rounded-lg flex items-center justify-center">
              <svg
                class="h-4 w-4 sm:h-5 sm:w-5 text-primary-600"
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
        {/* Scheduled Care Tasks */}
        <Show when={scheduledTasks().length > 0}>
          <div>
            <h3 class="text-base font-semibold text-gray-900 mb-4 flex items-center">
              <svg
                class="h-5 w-5 text-gray-400 mr-2"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              Care Schedule
            </h3>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
              <For each={scheduledTasks()}>
                {(task) => (
                  <div
                    class={`p-4 rounded-lg border ${getTaskStatusColor(task)}`}
                  >
                    <div class="flex items-center justify-between">
                      <div class="flex items-center min-w-0">
                        <span class="text-lg mr-2 flex-shrink-0">
                          {task.icon ?? "🌱"}
                        </span>
                        <div class="min-w-0">
                          <h4 class="text-sm font-medium truncate">
                            {task.name}
                          </h4>
                          <p class="text-xs font-semibold">
                            {getTaskStatusText(task)}
                          </p>
                          <p class="text-xs opacity-75">
                            Every {task.intervalDays} days
                          </p>
                        </div>
                      </div>
                      <Button
                        variant="primary"
                        size="sm"
                        onClick={() => handleLogCareTask(task)}
                        loading={submitting() === task.id}
                        class="ml-2 flex-shrink-0"
                      >
                        Done
                      </Button>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </div>
        </Show>

        {/* Unscheduled Quick Actions */}
        <Show when={unscheduledTasks().length > 0}>
          <div>
            <h3 class="text-base font-semibold text-gray-900 mb-4 flex items-center">
              <svg
                class="h-5 w-5 text-gray-400 mr-2"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M13 10V3L4 14h7v7l9-11h-7z"
                />
              </svg>
              Quick Actions
            </h3>
            <div class="flex flex-wrap gap-2">
              <For each={unscheduledTasks()}>
                {(task) => (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleLogCareTask(task)}
                    loading={submitting() === task.id}
                  >
                    <span class="mr-1">{task.icon ?? "🌱"}</span>
                    {task.name}
                  </Button>
                )}
              </For>
            </div>
          </div>
        </Show>

        {/* No care tasks fallback */}
        <Show when={careTasks().filter((t) => !t.archivedAt).length === 0}>
          <div class="text-center py-6 bg-gray-50 rounded-lg">
            <p class="text-sm text-gray-600 font-medium">
              No care tasks configured
            </p>
            <p class="text-xs text-gray-500 mt-1">
              Add care tasks to track watering, fertilizing, and more
            </p>
          </div>
        </Show>

        {/* Create Tracking Entry */}
        <div class="border-t border-gray-200 pt-4">
          <Button
            variant="outline"
            size="sm"
            onClick={openTrackingForm}
            class="flex items-center justify-center w-full"
          >
            <svg
              class="mr-2 h-4 w-4"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M12 4v16m8-8H4"
              />
            </svg>
            Log Custom Entry
          </Button>
        </div>

        {/* Divider */}
        <div class="border-t border-gray-200"></div>

        {/* Recent Activities Section */}
        <div>
          <h3 class="text-base font-semibold text-gray-900 mb-4 flex items-center">
            <svg
              class="h-5 w-5 text-gray-400 mr-2"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            Recent Activities
          </h3>

          <Show when={loading()}>
            <div class="text-center py-6">
              <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
              <p class="mt-2 text-sm text-gray-600">
                Loading recent activities...
              </p>
            </div>
          </Show>

          <Show when={!loading() && recentEntries().length === 0}>
            <div class="text-center py-6 bg-gray-50 rounded-lg">
              <svg
                class="mx-auto h-8 w-8 text-gray-400 mb-2"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={1.5}
                  d="M9 5H7a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
                />
              </svg>
              <p class="text-sm text-gray-600 font-medium">
                No recent activities
              </p>
              <p class="text-xs text-gray-500 mt-1">
                Use the care tasks above to log activities
              </p>
            </div>
          </Show>

          <Show when={!loading() && recentEntries().length > 0}>
            <div class="space-y-3">
              <For each={recentEntries()}>
                {(entry) => (
                  <div class="flex items-center space-x-3 p-3 bg-gray-50 rounded-lg hover:bg-gray-100 transition-colors duration-150">
                    <div class="flex-shrink-0">{getEntryIcon(entry)}</div>
                    <div class="flex-1 min-w-0">
                      <div class="flex items-center justify-between">
                        <p class="text-sm font-medium text-gray-900">
                          {getEntryLabel(entry)}
                        </p>
                        <p class="text-xs text-gray-500">
                          {formatDate(entry.timestamp)}
                        </p>
                      </div>
                      <Show when={entry.notes}>
                        <p class="text-sm text-gray-600 truncate">
                          {entry.notes}
                        </p>
                      </Show>
                      <Show
                        when={
                          entry.photoIds &&
                          Array.isArray(entry.photoIds) &&
                          (entry.photoIds as string[]).length > 0
                        }
                      >
                        <div class="mt-2 flex flex-wrap gap-1">
                          <For each={entry.photoIds as string[]}>
                            {(photoId) => (
                              <img
                                src={`/api/v1/plants/${props.plant.id}/photos/${photoId}`}
                                alt="Activity photo"
                                class="w-8 h-8 object-cover rounded border cursor-pointer hover:opacity-80 transition-opacity"
                                onClick={() =>
                                  window.open(
                                    `/api/v1/plants/${props.plant.id}/photos/${photoId}`,
                                    "_blank",
                                  )
                                }
                              />
                            )}
                          </For>
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
