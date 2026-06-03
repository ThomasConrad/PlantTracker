import { Component, createSignal, onCleanup, onMount, Show } from "solid-js";
import { onQueueChange } from "@/utils/offlineQueue";

/**
 * Shows a banner when the app is offline or has pending sync items.
 * Renders as a small fixed bar at the top of the viewport.
 */
export const OfflineIndicator: Component = () => {
  const [offline, setOffline] = createSignal(!navigator.onLine);
  const [pendingCount, setPendingCount] = createSignal(0);

  onMount(() => {
    const handleOnline = () => setOffline(false);
    const handleOffline = () => setOffline(true);

    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);

    const unsubscribe = onQueueChange((count) => setPendingCount(count));

    onCleanup(() => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
      unsubscribe();
    });
  });

  return (
    <Show when={offline() || pendingCount() > 0}>
      <div
        class={`fixed top-0 left-0 right-0 z-[9999] px-4 py-1.5 text-center text-xs font-medium transition-colors ${
          offline()
            ? "bg-amber-500 text-amber-950"
            : "bg-blue-500 text-white"
        }`}
      >
        <Show
          when={offline()}
          fallback={
            <span>
              {pendingCount()} pending change{pendingCount() > 1 ? "s" : ""} — will sync when online
            </span>
          }
        >
          <span>You're offline — changes will sync when you reconnect</span>
        </Show>
      </div>
    </Show>
  );
};
