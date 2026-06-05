import { Component, createSignal, onMount, Show } from "solid-js";
import { apiClient } from "@/api/client";

interface HealthHearts {
  hearts: number;
  score: number;
  scoredAt?: string;
  reasoning?: string;
}

interface Props {
  plantId: string;
  class?: string;
}

export const PlantHearts: Component<Props> = (props) => {
  const [hearts, setHearts] = createSignal<HealthHearts | null>(null);

  onMount(async () => {
    try {
      const data = await apiClient.request<HealthHearts>(
        `/coach/plants/${props.plantId}/health`,
      );
      setHearts(data);
    } catch {
      // Silently fail — hearts are supplementary
    }
  });

  const renderHearts = () => {
    const h = hearts();
    if (!h) return null;

    const full = Math.floor(h.hearts);
    const hasHalf = h.hearts % 1 >= 0.5;
    const empty = 5 - full - (hasHalf ? 1 : 0);

    const tooltip = h.reasoning
      ? `${h.reasoning} (${h.hearts}/5)`
      : `Health: ${h.hearts}/5`;

    return (
      <span
        class={`inline-flex items-center gap-px ${props.class || ""}`}
        title={tooltip}
      >
        {"❤️".repeat(full)}
        {hasHalf ? "💛" : ""}
        {"🤍".repeat(empty)}
      </span>
    );
  };

  return <Show when={hearts()}>{renderHearts()}</Show>;
};
