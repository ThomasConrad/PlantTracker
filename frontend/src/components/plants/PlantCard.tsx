import { Component, Show, For, createMemo } from "solid-js";
import { A } from "@solidjs/router";
import type { Plant } from "@/types";
import { PlantHearts } from "./PlantHearts";

interface PlantCardProps {
  plant: Plant;
}

export const PlantCard: Component<PlantCardProps> = (props) => {
  const overdueTasks = createMemo(() =>
    (props.plant.careTasks || []).filter((t) => t.isDue),
  );

  const overdueCount = createMemo(() => overdueTasks().length);

  return (
    <A href={`/plants/${props.plant.id}`} class="plant-card-full-image group">
      <div class="relative aspect-[4/5] overflow-hidden rounded-2xl shadow-sm">
        <Show
          when={props.plant.previewUrl}
          fallback={
            <div class="h-full w-full bg-gradient-to-br from-primary-100 to-primary-200 flex items-center justify-center">
              <svg
                class="h-16 w-16 text-primary-600 opacity-60"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={1.5}
                  d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"
                />
              </svg>
            </div>
          }
        >
          <img
            src={props.plant.previewUrl!}
            alt={`${props.plant.name} preview`}
            class="h-full w-full object-cover group-hover:scale-105 transition-transform duration-300"
            loading="lazy"
          />
        </Show>

        {/* Gradient overlay for better text contrast */}
        <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-black/10 to-transparent" />

        {/* Overdue badge */}
        <Show when={overdueCount() > 0}>
          <div class="absolute top-3 right-3 flex items-center gap-1 bg-red-500 text-white text-xs font-bold rounded-full px-2 py-0.5 shadow-lg">
            <svg
              class="w-3 h-3"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2.5"
                d="M12 8v4m0 4h.01"
              />
            </svg>
            {overdueCount()}
          </div>
        </Show>

        {/* Health hearts */}
        <div class="absolute top-3 left-3">
          <PlantHearts
            plantId={props.plant.id}
            class="text-sm drop-shadow-lg"
          />
        </div>

        {/* Plant name and genus overlay */}
        <div class="absolute bottom-0 left-0 right-0 p-5">
          <Show when={props.plant.archivedAt}>
            <div class="inline-flex items-center rounded-full bg-black/50 px-2.5 py-1 text-xs font-semibold text-white mb-2">
              Archived
            </div>
          </Show>
          <h3 class="text-white font-bold text-xl leading-tight mb-1 drop-shadow-lg">
            {props.plant.name}
          </h3>
          <p class="text-white/95 text-base italic drop-shadow-md font-medium">
            {props.plant.genus}
          </p>
          {/* Overdue task icons */}
          <Show when={overdueCount() > 0}>
            <div class="flex gap-1.5 mt-2">
              <For each={overdueTasks().slice(0, 4)}>
                {(task) => (
                  <span
                    class="bg-red-500/80 backdrop-blur-sm text-white text-xs rounded-full px-2 py-0.5 flex items-center gap-1"
                    title={`${task.name} overdue${task.daysOverdue ? ` by ${task.daysOverdue}d` : ""}`}
                  >
                    <Show
                      when={task.icon}
                      fallback={<span class="w-3 h-3">!</span>}
                    >
                      <span class="text-xs">{task.icon}</span>
                    </Show>
                    <Show when={task.daysOverdue && task.daysOverdue > 0}>
                      <span class="text-[10px] font-medium">
                        {task.daysOverdue}d
                      </span>
                    </Show>
                  </span>
                )}
              </For>
              <Show when={overdueCount() > 4}>
                <span class="text-white/80 text-xs self-center">
                  +{overdueCount() - 4}
                </span>
              </Show>
            </div>
          </Show>
        </div>
      </div>
    </A>
  );
};
