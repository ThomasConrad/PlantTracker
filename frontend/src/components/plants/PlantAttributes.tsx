import { Component, createSignal, For, Show, onMount } from "solid-js";
import { apiClient } from "@/api/client";
import type { components } from "@/types";

type PlantAttribute = components["schemas"]["PlantAttribute"];

interface Props {
  plantId: string;
}

export const PlantAttributes: Component<Props> = (props) => {
  const [attributes, setAttributes] = createSignal<PlantAttribute[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [expanded, setExpanded] = createSignal(false);

  onMount(async () => {
    try {
      const response = await apiClient.request<{
        attributes: PlantAttribute[];
      }>(`/plants/${props.plantId}/attributes`);
      setAttributes(response.attributes);
    } catch {
      // Silently fail — attributes are supplementary
    } finally {
      setLoading(false);
    }
  });

  const visibleAttributes = () =>
    expanded() ? attributes() : attributes().slice(0, 4);

  const hasMore = () => attributes().length > 4;

  return (
    <Show when={!loading() && attributes().length > 0}>
      <section class="bg-white shadow-sm rounded-xl border border-gray-200 overflow-hidden">
        <div class="px-4 py-3 bg-gray-50/60 border-b border-gray-100">
          <h2 class="text-base font-semibold text-gray-900">
            Requirements & Info
          </h2>
        </div>
        <div class="p-4">
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <For each={visibleAttributes()}>
              {(attr) => (
                <div class="flex items-start gap-2.5 rounded-lg bg-gray-50 border border-gray-100 px-3 py-2.5">
                  <Show when={attr.icon}>
                    <span class="text-base flex-shrink-0 mt-0.5">
                      {attr.icon}
                    </span>
                  </Show>
                  <div class="min-w-0 flex-1">
                    <p class="text-xs font-medium text-gray-500 leading-tight">
                      {attr.label}
                    </p>
                    <p class="text-sm text-gray-900 leading-snug mt-0.5">
                      {attr.value}
                    </p>
                  </div>
                </div>
              )}
            </For>
          </div>
          <Show when={hasMore()}>
            <button
              onClick={() => setExpanded((v) => !v)}
              class="mt-2.5 text-xs text-primary-600 hover:text-primary-800 font-medium transition-colors"
            >
              {expanded()
                ? "Show less"
                : `Show ${attributes().length - 4} more`}
            </button>
          </Show>
        </div>
      </section>
    </Show>
  );
};
