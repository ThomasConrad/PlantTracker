import { Component, Show, For, createSignal, createEffect } from "solid-js";
import { useNavigate } from "@solidjs/router";
import type { Plant } from "@/types";
import { plantsStore } from "@/stores/plants";
import { PlantHearts } from "./PlantHearts";
import { Button } from "@/components/ui/Button";
import { calculateDaysUntil, formatDate, isOverdue } from "@/utils/date";

interface PlantDetailSheetProps {
  plant: Plant;
  onDismiss: () => void;
}

export const PlantDetailSheet: Component<PlantDetailSheetProps> = (props) => {
  const navigate = useNavigate();
  const [quickActionLoading, setQuickActionLoading] = createSignal<
    Record<string, boolean>
  >({});
  const [quickActionError, setQuickActionError] = createSignal<string | null>(
    null,
  );

  // Load full plant details (for care tasks etc)
  const [fullPlant, setFullPlant] = createSignal<Plant | null>(null);

  createEffect(() => {
    const id = props.plant.id;
    if (id) {
      // Load detailed plant data (includes care tasks with status)
      plantsStore.loadPlant(id).then(() => {
        setFullPlant(plantsStore.selectedPlant);
      });
    }
  });

  const plant = () => fullPlant() || props.plant;

  const activeTasks = () =>
    (plant().careTasks || []).filter((t) => !t.archivedAt);

  const taskStatuses = () =>
    activeTasks().map((task) => ({
      task,
      status: getCareStatus(
        task.lastPerformed ?? null,
        task.intervalDays ?? null,
        task.name,
      ),
      overdue:
        Boolean(task.intervalDays) &&
        isOverdue(task.lastPerformed ?? null, task.intervalDays!),
    }));

  const getCareStatus = (
    lastDate: string | null,
    intervalDays: number | null,
    actionLabel: string,
  ) => {
    if (!intervalDays) {
      return {
        title: `No ${actionLabel.toLowerCase()} schedule`,
        detail: "Enable this in Edit Plant.",
        statusClass: "border-gray-200 bg-gray-50 text-gray-700",
        badgeLabel: "No schedule",
        badgeClass: "bg-gray-100 text-gray-700",
      };
    }

    const overdue = isOverdue(lastDate, intervalDays);
    const days = calculateDaysUntil(lastDate, intervalDays);

    if (overdue) {
      return {
        title: `${actionLabel} overdue`,
        detail: "Due now.",
        statusClass: "border-red-200 bg-red-50 text-red-800",
        badgeLabel: "Overdue",
        badgeClass: "bg-red-100 text-red-700",
      };
    }

    if (days === 0) {
      return {
        title: `${actionLabel} today`,
        detail: "You are on schedule.",
        statusClass: "border-amber-200 bg-amber-50 text-amber-800",
        badgeLabel: "Due today",
        badgeClass: "bg-amber-100 text-amber-700",
      };
    }

    if (days === 1) {
      return {
        title: `${actionLabel} tomorrow`,
        detail: "You are on schedule.",
        statusClass: "border-emerald-200 bg-emerald-50 text-emerald-800",
        badgeLabel: "Due tomorrow",
        badgeClass: "bg-emerald-100 text-emerald-700",
      };
    }

    return {
      title: `${actionLabel} in ${days} days`,
      detail: "You are on schedule.",
      statusClass: "border-emerald-200 bg-emerald-50 text-emerald-800",
      badgeLabel: `In ${days} days`,
      badgeClass: "bg-emerald-100 text-emerald-700",
    };
  };

  const handleQuickCare = async (careTaskId: string) => {
    try {
      setQuickActionError(null);
      setQuickActionLoading((prev) => ({ ...prev, [careTaskId]: true }));
      await plantsStore.createTrackingEntry(props.plant.id, {
        careTaskIds: [careTaskId],
        timestamp: new Date().toISOString(),
      });
      // Reload to get updated status
      await plantsStore.loadPlant(props.plant.id);
      setFullPlant(plantsStore.selectedPlant);
    } catch (error) {
      console.error("Failed to create quick care entry:", error);
      setQuickActionError("Could not save. Please try again.");
    } finally {
      setQuickActionLoading((prev) => ({ ...prev, [careTaskId]: false }));
    }
  };

  return (
    <div class="space-y-5 pb-24">
      {/* Plant hero section */}
      <div class="px-5 pt-1">
        <div class="flex items-start gap-4">
          {/* Thumbnail */}
          <div class="flex-shrink-0 w-20 h-20 rounded-xl overflow-hidden shadow-sm">
            <Show
              when={plant().previewUrl}
              fallback={
                <div class="w-full h-full bg-gradient-to-br from-primary-100 to-primary-200 flex items-center justify-center">
                  <svg
                    class="h-8 w-8 text-primary-600 opacity-60"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width={1.5}
                      d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"
                    />
                  </svg>
                </div>
              }
            >
              <img
                src={plant().previewUrl!}
                alt={plant().name}
                class="w-full h-full object-cover"
              />
            </Show>
          </div>

          {/* Name and details */}
          <div class="flex-1 min-w-0">
            <h2 class="text-xl font-bold text-gray-900 truncate">
              {plant().name}
            </h2>
            <p class="text-sm text-gray-500 italic">{plant().genus}</p>
            <div class="mt-1">
              <PlantHearts plantId={plant().id} class="text-sm" />
            </div>
          </div>

          {/* Quick nav to full detail */}
          <button
            onClick={() => {
              props.onDismiss();
              navigate(`/plants/${props.plant.id}`);
            }}
            class="flex-shrink-0 p-2 rounded-full bg-gray-100 text-gray-600 hover:bg-gray-200 transition-colors"
            aria-label="View full details"
          >
            <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M9 5l7 7-7 7"
              />
            </svg>
          </button>
        </div>
      </div>

      {/* Quick care buttons */}
      <Show when={activeTasks().length > 0}>
        <div class="px-5">
          <div
            class={`grid gap-2.5 ${activeTasks().length > 1 ? "grid-cols-2" : "grid-cols-1"}`}
          >
            <For each={taskStatuses()}>
              {({ task, overdue }) => (
                <Button
                  variant={overdue ? "danger" : "primary"}
                  size="md"
                  class="min-h-[44px] w-full"
                  onClick={() => handleQuickCare(task.id)}
                  loading={quickActionLoading()[task.id]}
                >
                  {`${task.icon || ""} ${task.name} now`.trim()}
                </Button>
              )}
            </For>
          </div>
          <Show when={quickActionError()}>
            <p class="text-xs text-red-600 mt-2">{quickActionError()}</p>
          </Show>
        </div>
      </Show>

      {/* Care status cards */}
      <Show when={taskStatuses().length > 0}>
        <div class="px-5 space-y-2.5">
          <h3 class="text-sm font-semibold text-gray-500 uppercase tracking-wide">
            Care Status
          </h3>
          <For each={taskStatuses()}>
            {({ task, status }) => (
              <div class={`rounded-xl border p-3 ${status.statusClass}`}>
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0">
                    <div class="flex items-center gap-2">
                      <span class="text-sm">{task.icon || "🌱"}</span>
                      <p class="text-sm font-semibold">{status.title}</p>
                    </div>
                    <Show when={task.lastPerformed}>
                      <p class="text-xs mt-1 opacity-80">
                        Last: {formatDate(task.lastPerformed!)}
                      </p>
                    </Show>
                  </div>
                  <span
                    class={`inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-semibold ${status.badgeClass}`}
                  >
                    {status.badgeLabel}
                  </span>
                </div>
              </div>
            )}
          </For>
        </div>
      </Show>

      {/* Action links */}
      <div class="px-5 grid grid-cols-3 gap-2.5">
        <button
          onClick={() => {
            props.onDismiss();
            navigate(`/plants/${props.plant.id}`);
          }}
          class="flex flex-col items-center gap-1.5 p-3 rounded-xl bg-gray-50 border border-gray-200 text-gray-700 hover:bg-gray-100 transition-colors"
        >
          <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width={1.5}
              d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
            />
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width={1.5}
              d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
            />
          </svg>
          <span class="text-xs font-medium">Details</span>
        </button>

        <button
          onClick={() => {
            props.onDismiss();
            navigate(`/plants/${props.plant.id}/photos`);
          }}
          class="flex flex-col items-center gap-1.5 p-3 rounded-xl bg-gray-50 border border-gray-200 text-gray-700 hover:bg-gray-100 transition-colors"
        >
          <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width={1.5}
              d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
            />
          </svg>
          <span class="text-xs font-medium">Photos</span>
        </button>

        <button
          onClick={() => {
            props.onDismiss();
            navigate(`/plants/${props.plant.id}/coach`);
          }}
          class="flex flex-col items-center gap-1.5 p-3 rounded-xl bg-gray-50 border border-gray-200 text-gray-700 hover:bg-gray-100 transition-colors"
        >
          <svg class="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width={1.5}
              d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"
            />
          </svg>
          <span class="text-xs font-medium">Coach</span>
        </button>
      </div>

      {/* Edit link */}
      <div class="px-5">
        <button
          onClick={() => {
            props.onDismiss();
            navigate(`/plants/${props.plant.id}/edit`);
          }}
          class="w-full flex items-center justify-center gap-2 py-3 rounded-xl border border-gray-200 bg-white text-gray-700 font-medium text-sm hover:bg-gray-50 transition-colors"
        >
          <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width={2}
              d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
            />
          </svg>
          Edit Plant
        </button>
      </div>
    </div>
  );
};
