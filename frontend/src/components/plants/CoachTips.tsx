import { Component, createSignal, For, onMount, Show } from "solid-js";
import { A } from "@solidjs/router";
import {
  coachApi,
  type PendingSuggestion,
} from "@/api/coach";

const SUGGESTION_ICONS: Record<string, string> = {
  schedule_change: "📅",
  new_task: "✨",
  care_action: "💧",
  photo_request: "📷",
  species_correction: "🔬",
};

export const CoachTips: Component = () => {
  const [suggestions, setSuggestions] = createSignal<PendingSuggestion[]>([]);
  const [dismissing, setDismissing] = createSignal<Set<string>>(new Set());
  const [accepting, setAccepting] = createSignal<Set<string>>(new Set());

  onMount(async () => {
    try {
      const resp = await coachApi.getPendingSuggestions();
      setSuggestions(resp.suggestions);
    } catch {
      // Silently fail — tips are supplementary
    }
  });

  const handleAccept = async (suggestion: PendingSuggestion) => {
    setAccepting((prev) => new Set([...prev, suggestion.id]));
    try {
      await coachApi.acceptSuggestion(suggestion.id);
      setSuggestions((prev) => prev.filter((s) => s.id !== suggestion.id));
    } catch {
      // silently fail
    } finally {
      setAccepting((prev) => {
        const next = new Set(prev);
        next.delete(suggestion.id);
        return next;
      });
    }
  };

  const handleDismiss = async (suggestion: PendingSuggestion) => {
    setDismissing((prev) => new Set([...prev, suggestion.id]));
    try {
      await coachApi.dismissSuggestion(suggestion.id);
      setSuggestions((prev) => prev.filter((s) => s.id !== suggestion.id));
    } catch {
      // silently fail
    } finally {
      setDismissing((prev) => {
        const next = new Set(prev);
        next.delete(suggestion.id);
        return next;
      });
    }
  };

  return (
    <Show when={suggestions().length > 0}>
      <div class="px-4 sm:px-6 mb-6">
        <div class="max-w-7xl mx-auto">
          <div class="flex items-center gap-2 mb-3">
            <div class="w-2 h-2 rounded-full bg-amber-400" />
            <h2 class="text-sm font-semibold text-gray-700 uppercase tracking-wide">
              Coach Tips
            </h2>
            <span class="text-xs text-gray-400 font-medium">
              {suggestions().length}{" "}
              {suggestions().length === 1 ? "suggestion" : "suggestions"}
            </span>
          </div>

          <div class="space-y-2">
            <For each={suggestions().slice(0, 5)}>
              {(suggestion) => (
                <div class="flex items-center gap-3 bg-white border border-amber-100 rounded-xl px-4 py-3 shadow-sm">
                  {/* Icon */}
                  <div class="flex-shrink-0 w-10 h-10 rounded-full bg-amber-50 flex items-center justify-center">
                    <span class="text-lg">
                      {SUGGESTION_ICONS[suggestion.suggestionType] || "💡"}
                    </span>
                  </div>

                  {/* Info */}
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <A
                        href={`/plants/${suggestion.plantId}`}
                        class="text-sm font-medium text-gray-900 truncate hover:text-primary-600 transition-colors"
                      >
                        {suggestion.plantName}
                      </A>
                    </div>
                    <p class="text-xs text-gray-600 line-clamp-2">
                      {suggestion.description}
                    </p>
                  </div>

                  {/* Actions */}
                  <div class="flex-shrink-0 flex items-center gap-1">
                    {/* Accept */}
                    <button
                      onClick={() => handleAccept(suggestion)}
                      disabled={
                        accepting().has(suggestion.id) ||
                        dismissing().has(suggestion.id)
                      }
                      class="w-8 h-8 rounded-full bg-green-50 hover:bg-green-100 text-green-600 flex items-center justify-center transition-colors disabled:opacity-50"
                      title="Accept suggestion"
                    >
                      <svg
                        class="w-4 h-4"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2.5"
                          d="M5 13l4 4L19 7"
                        />
                      </svg>
                    </button>
                    {/* Dismiss */}
                    <button
                      onClick={() => handleDismiss(suggestion)}
                      disabled={
                        accepting().has(suggestion.id) ||
                        dismissing().has(suggestion.id)
                      }
                      class="w-8 h-8 rounded-full bg-gray-50 hover:bg-gray-100 text-gray-400 flex items-center justify-center transition-colors disabled:opacity-50"
                      title="Dismiss"
                    >
                      <svg
                        class="w-4 h-4"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width="2"
                          d="M6 18L18 6M6 6l12 12"
                        />
                      </svg>
                    </button>
                  </div>
                </div>
              )}
            </For>

            <Show when={suggestions().length > 5}>
              <p class="text-xs text-gray-400 text-center pt-1">
                +{suggestions().length - 5} more suggestions
              </p>
            </Show>
          </div>
        </div>
      </div>
    </Show>
  );
};
