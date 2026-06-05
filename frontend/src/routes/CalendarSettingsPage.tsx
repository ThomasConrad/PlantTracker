import { Component, createSignal, onMount, Show } from "solid-js";
import { Button } from "@/components/ui/Button";
import { LoadingSpinner } from "@/components/ui/LoadingSpinner";
import { apiClient } from "@/api/client";

interface CalendarSubscriptionInfo {
  feedUrl: string;
}

interface GoogleTasksStatus {
  connected: boolean;
  connected_at?: string;
  scopes?: string[];
  expires_at?: string;
}

export const CalendarSettingsPage: Component = () => {
  const [subscriptionInfo, setSubscriptionInfo] =
    createSignal<CalendarSubscriptionInfo | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [copied, setCopied] = createSignal(false);
  const [regenerating, setRegenerating] = createSignal(false);

  // Google Tasks state
  const [googleStatus, setGoogleStatus] =
    createSignal<GoogleTasksStatus | null>(null);
  const [googleLoading, setGoogleLoading] = createSignal(false);
  const [googleError, setGoogleError] = createSignal<string | null>(null);
  const [syncing, setSyncing] = createSignal(false);
  const [polling, setPolling] = createSignal(false);

  // Toast notification
  const [toast, setToast] = createSignal<{
    message: string;
    type: "success" | "error";
  } | null>(null);

  const showToast = (message: string, type: "success" | "error" = "success") => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 4000);
  };

  const loadSubscriptionInfo = async () => {
    try {
      setLoading(true);
      setError(null);

      const response = await apiClient.request<CalendarSubscriptionInfo>(
        "/calendar/subscription",
      );
      setSubscriptionInfo(response);
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error
          ? err.message
          : "Failed to load calendar subscription info";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const copyToClipboard = async () => {
    const info = subscriptionInfo();
    if (!info) return;

    try {
      await navigator.clipboard.writeText(info.feedUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy to clipboard:", err);
    }
  };

  const regenerateToken = async () => {
    try {
      setRegenerating(true);
      setError(null);

      const response = await apiClient.request<{
        feedUrl: string;
        message: string;
      }>("/calendar/regenerate-token", {
        method: "POST",
      });

      const currentInfo = subscriptionInfo();
      if (currentInfo) {
        setSubscriptionInfo({
          ...currentInfo,
          feedUrl: response.feedUrl,
        });
      }

      setCopied(false);
      showToast("Calendar URL regenerated successfully");
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to regenerate token";
      setError(errorMessage);
    } finally {
      setRegenerating(false);
    }
  };

  // Google Tasks functions
  const loadGoogleStatus = async () => {
    try {
      setGoogleLoading(true);
      setGoogleError(null);

      const response = await apiClient.request<GoogleTasksStatus>(
        "/google-tasks/status",
      );
      setGoogleStatus(response);
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error
          ? err.message
          : "Failed to load Google Tasks status";
      setGoogleError(errorMessage);
    } finally {
      setGoogleLoading(false);
    }
  };

  const connectGoogleTasks = async () => {
    try {
      setGoogleLoading(true);
      setGoogleError(null);

      const response = await apiClient.request<{
        auth_url: string;
        state: string;
      }>("/google-tasks/auth-url");

      window.location.href = response.auth_url;
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error
          ? err.message
          : "Failed to get Google authorization URL";
      setGoogleError(errorMessage);
      setGoogleLoading(false);
    }
  };

  const disconnectGoogleTasks = async () => {
    try {
      setGoogleLoading(true);
      setGoogleError(null);

      await apiClient.request("/google-tasks/disconnect", {
        method: "POST",
      });

      setGoogleStatus({ connected: false });
      showToast("Google Tasks disconnected");
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error
          ? err.message
          : "Failed to disconnect Google Tasks";
      setGoogleError(errorMessage);
    } finally {
      setGoogleLoading(false);
    }
  };

  const syncPlantTasks = async () => {
    try {
      setSyncing(true);
      setGoogleError(null);

      const response = await apiClient.request<{
        success: boolean;
        message: string;
        tasks_created: number;
        tasks_skipped: number;
      }>("/google-tasks/sync-tasks", {
        method: "POST",
        body: JSON.stringify({ days_ahead: 365 }),
      });

      showToast(response.message);
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error ? err.message : "Failed to sync plant tasks";
      setGoogleError(errorMessage);
    } finally {
      setSyncing(false);
    }
  };

  const pollCompletions = async () => {
    try {
      setPolling(true);
      setGoogleError(null);

      const response = await apiClient.request<{
        success: boolean;
        message: string;
        completed: number;
        checked: number;
      }>("/google-tasks/poll-completions", {
        method: "POST",
      });

      showToast(response.message);
    } catch (err: unknown) {
      const errorMessage =
        err instanceof Error
          ? err.message
          : "Failed to poll task completions";
      setGoogleError(errorMessage);
    } finally {
      setPolling(false);
    }
  };

  onMount(() => {
    loadSubscriptionInfo();
    loadGoogleStatus();
  });

  return (
    <div class="max-w-4xl mx-auto px-4 py-8">
      {/* Toast notification */}
      <Show when={toast()}>
        {(t) => (
          <div
            class={`fixed top-4 right-4 z-50 px-4 py-3 rounded-lg shadow-lg transition-all duration-300 ${
              t().type === "success"
                ? "bg-green-600 text-white"
                : "bg-red-600 text-white"
            }`}
          >
            {t().message}
          </div>
        )}
      </Show>

      <div class="mb-8">
        <h1 class="text-3xl font-bold text-gray-900 dark:text-gray-100">
          Calendar & Task Integration
        </h1>
      </div>

      <Show when={loading()}>
        <div class="flex justify-center py-8">
          <LoadingSpinner />
        </div>
      </Show>

      <Show when={error()}>
        <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4 mb-6">
          <div class="flex">
            <div class="ml-3">
              <h3 class="text-sm font-medium text-red-800 dark:text-red-300">Error</h3>
              <div class="mt-2 text-sm text-red-700 dark:text-red-400">{error()}</div>
            </div>
          </div>
        </div>
      </Show>

      {/* Google Tasks Integration */}
      <div class="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
        <div class="flex items-center justify-between mb-4">
          <div>
            <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100">
              Google Tasks
            </h2>
            <p class="text-sm text-gray-600 dark:text-gray-400 mt-1">
              Create plant care tasks in Google Tasks and sync completions back
            </p>
          </div>
          <div class="flex items-center">
            <Show
              when={googleStatus()?.connected}
              fallback={
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 dark:bg-gray-700 text-gray-800 dark:text-gray-300">
                  Not Connected
                </span>
              }
            >
              <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300">
                Connected
              </span>
            </Show>
          </div>
        </div>

        <Show when={googleError()}>
          <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-4 mb-4">
            <div class="text-sm text-red-700 dark:text-red-400">{googleError()}</div>
          </div>
        </Show>

        <Show when={googleLoading()}>
          <div class="flex justify-center py-4">
            <LoadingSpinner size="sm" />
          </div>
        </Show>

        <Show when={!googleLoading()}>
          <Show
            when={googleStatus()?.connected}
            fallback={
              <Button
                onClick={connectGoogleTasks}
                disabled={googleLoading()}
                class="w-full sm:w-auto"
              >
                <Show when={googleLoading()} fallback="Connect Google Tasks">
                  <LoadingSpinner size="sm" class="mr-2" />
                  Connecting...
                </Show>
              </Button>
            }
          >
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm text-gray-600 dark:text-gray-400">
                    Connected on{" "}
                    {googleStatus()?.connected_at
                      ? new Date(
                          googleStatus()!.connected_at!,
                        ).toLocaleDateString("en-GB")
                      : "Unknown"}
                  </p>
                  <Show when={googleStatus()?.expires_at}>
                    <p class="text-xs text-gray-500 dark:text-gray-500">
                      Access expires:{" "}
                      {new Date(googleStatus()!.expires_at!).toLocaleDateString(
                        "en-GB",
                      )}
                    </p>
                  </Show>
                </div>
                <Button
                  onClick={disconnectGoogleTasks}
                  variant="outline"
                  disabled={googleLoading()}
                  class="text-red-600 dark:text-red-400 border-red-300 dark:border-red-700 hover:bg-red-50 dark:hover:bg-red-900/20"
                >
                  Disconnect
                </Button>
              </div>

              <div class="flex flex-wrap gap-3">
                <Button
                  onClick={syncPlantTasks}
                  disabled={syncing()}
                >
                  <Show when={syncing()} fallback="Sync Plant Tasks">
                    <LoadingSpinner size="sm" class="mr-2" />
                    Syncing...
                  </Show>
                </Button>
                <Button
                  onClick={pollCompletions}
                  disabled={polling()}
                  variant="outline"
                >
                  <Show when={polling()} fallback="Check Completions">
                    <LoadingSpinner size="sm" class="mr-2" />
                    Checking...
                  </Show>
                </Button>
              </div>
            </div>
          </Show>
        </Show>
      </div>

      <Show when={subscriptionInfo()}>
        {(info) => (
          <div class="mt-8">
            {/* iCalendar Subscription */}
            <div class="bg-white dark:bg-gray-800 shadow rounded-lg p-6">
              <h2 class="text-xl font-semibold text-gray-900 dark:text-gray-100 mb-2">
                iCalendar Feed
              </h2>
              <p class="text-sm text-gray-600 dark:text-gray-400 mb-4">
                Subscribe to this URL in any calendar app to see plant care events
              </p>

              <div class="space-y-4">
                <div>
                  <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    Subscription URL
                  </label>
                  <div class="flex space-x-2">
                    <input
                      type="text"
                      value={info().feedUrl}
                      readonly
                      class="flex-1 min-w-0 block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md text-sm bg-gray-50 dark:bg-gray-700 dark:text-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                    />
                    <Button
                      onClick={copyToClipboard}
                      variant="outline"
                      class="px-4 py-2"
                    >
                      <Show when={copied()} fallback="Copy">
                        Copied!
                      </Show>
                    </Button>
                  </div>
                </div>

                <div class="flex items-center justify-between">
                  <p class="text-sm text-gray-500 dark:text-gray-400">
                    Keep this URL private. Regenerate if compromised.
                  </p>
                  <Button
                    onClick={regenerateToken}
                    variant="outline"
                    disabled={regenerating()}
                    class="px-4 py-2"
                  >
                    <Show when={regenerating()} fallback="Regenerate">
                      <LoadingSpinner size="sm" class="mr-2" />
                      Regenerating...
                    </Show>
                  </Button>
                </div>
              </div>
            </div>
          </div>
        )}
      </Show>
    </div>
  );
};
