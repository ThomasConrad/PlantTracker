import { Component, Show, createEffect } from "solid-js";
import { A, useParams } from "@solidjs/router";
import { PhotoGallery } from "@/components/plants/PhotoGallery";
import { LoadingSpinner } from "@/components/ui/LoadingSpinner";
import { plantsStore } from "@/stores/plants";

export const PlantPhotosPage: Component = () => {
  const params = useParams();

  createEffect(() => {
    if (params.id && plantsStore.selectedPlant?.id !== params.id) {
      void plantsStore.loadPlant(params.id);
    }
  });

  return (
    <div class="min-h-full pb-20 sm:pb-8">
      <div class="px-4 sm:px-6 pt-4 sm:pt-6 pb-6">
        <div class="max-w-5xl mx-auto">
          <A
            href={`/plants/${params.id}`}
            class="inline-flex items-center gap-2 p-2 -ml-2 text-gray-500 hover:text-gray-700 hover:bg-gray-50 rounded-lg transition-colors duration-200"
            aria-label="Back to plant"
          >
            <svg
              class="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M15 19l-7-7 7-7"
              />
            </svg>
            <span class="text-sm font-medium">
              <Show
                when={plantsStore.selectedPlant?.id === params.id}
                fallback="Back to plant"
              >
                Back to {plantsStore.selectedPlant?.name}
              </Show>
            </span>
          </A>
        </div>
      </div>

      <div class="px-4 sm:px-6">
        <div class="max-w-5xl mx-auto">
          <Show
            when={
              !plantsStore.loading ||
              plantsStore.selectedPlant?.id === params.id
            }
            fallback={
              <div class="flex justify-center py-16">
                <LoadingSpinner />
              </div>
            }
          >
            <PhotoGallery plantId={params.id} mode="full" />
          </Show>
        </div>
      </div>
    </div>
  );
};
