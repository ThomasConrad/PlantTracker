import { Component, For, Show, createSignal, onMount } from "solid-js";
import { apiClient } from "@/api/client";
import {
  remindersStore,
  type DueReminder,
  type ReminderPreferences,
} from "@/stores/reminders";

export const RemindersPage: Component = () => {
  const [prefs, setPrefs] = createSignal<ReminderPreferences>({
    enabled: false,
    reminderTime: "09:00",
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC",
    browserNotificationsEnabled: false,
    pushHealthAlerts: true,
    pushDailySummary: true,
    pushCoachSuggestions: true,
    pushReminders: true,
  });
  const [dueReminders, setDueReminders] = createSignal<DueReminder[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [success, setSuccess] = createSignal<string | null>(null);

  const loadData = async () => {
    try {
      setLoading(true);
      setError(null);
      const [preferences, due] = await Promise.all([
        apiClient.getReminderPreferences(),
        apiClient.getDueReminders(),
      ]);
      setPrefs(preferences);
      setDueReminders(due.reminders);
      await remindersStore.loadDueCount();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load reminders");
    } finally {
      setLoading(false);
    }
  };

  const savePreferences = async (e: Event) => {
    e.preventDefault();
    try {
      setSaving(true);
      setError(null);
      const updated = await apiClient.updateReminderPreferences(prefs());
      setPrefs(updated);
      setSuccess("Reminder preferences saved");
      setTimeout(() => setSuccess(null), 2500);
      await remindersStore.refreshAndNotify();
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : "Failed to save reminder preferences",
      );
    } finally {
      setSaving(false);
    }
  };

  const requestPermission = async () => {
    const result = await remindersStore.requestNotificationPermission();
    if (result === "unsupported") {
      setError("Browser notifications are not supported in this browser.");
      return;
    }
    if (result === "granted") {
      setSuccess("Browser notifications enabled.");
      setPrefs((prev) => ({ ...prev, browserNotificationsEnabled: true }));
      return;
    }
    setError("Notification permission was not granted.");
  };

  const sendNow = async () => {
    try {
      setError(null);
      const dispatched = await apiClient.dispatchDueReminders();
      setSuccess(
        `Dispatched ${dispatched.sentCount} reminder notification(s).`,
      );
      setTimeout(() => setSuccess(null), 2500);
      await loadData();
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to dispatch reminders",
      );
    }
  };

  onMount(() => {
    void loadData();
  });

  return (
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-6">
      <div>
        <h1 class="text-3xl font-bold text-gray-900">Reminders</h1>
        <p class="mt-2 text-gray-600">
          Configure daily reminders and review what is due now.
        </p>
      </div>

      <Show when={error()}>
        <div class="bg-red-50 border border-red-200 rounded-md p-3 text-sm text-red-800">
          {error()}
        </div>
      </Show>
      <Show when={success()}>
        <div class="bg-green-50 border border-green-200 rounded-md p-3 text-sm text-green-800">
          {success()}
        </div>
      </Show>

      <form
        class="bg-white shadow rounded-lg p-6 space-y-4"
        onSubmit={savePreferences}
      >
        <div class="flex items-center justify-between">
          <label class="text-sm font-medium text-gray-800">
            Enable reminders
          </label>
          <input
            type="checkbox"
            checked={prefs().enabled}
            onChange={(e) =>
              setPrefs((prev) => ({
                ...prev,
                enabled: e.currentTarget.checked,
              }))
            }
          />
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              Daily reminder time
            </label>
            <input
              type="time"
              value={prefs().reminderTime}
              onInput={(e) =>
                setPrefs((prev) => ({
                  ...prev,
                  reminderTime: e.currentTarget.value,
                }))
              }
              class="w-full px-3 py-2 border border-gray-300 rounded-md"
              required
            />
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 mb-1">
              Timezone
            </label>
            <input
              type="text"
              value={prefs().timezone}
              onInput={(e) =>
                setPrefs((prev) => ({
                  ...prev,
                  timezone: e.currentTarget.value,
                }))
              }
              class="w-full px-3 py-2 border border-gray-300 rounded-md"
              required
            />
          </div>
        </div>

        <div class="flex items-center justify-between">
          <label class="text-sm font-medium text-gray-800">
            Use browser notifications
          </label>
          <input
            type="checkbox"
            checked={prefs().browserNotificationsEnabled}
            onChange={(e) =>
              setPrefs((prev) => ({
                ...prev,
                browserNotificationsEnabled: e.currentTarget.checked,
              }))
            }
          />
        </div>

        <div class="flex flex-wrap gap-3">
          <button
            type="submit"
            disabled={saving()}
            class="px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 disabled:opacity-50"
          >
            {saving() ? "Saving..." : "Save Preferences"}
          </button>
          <button
            type="button"
            onClick={requestPermission}
            class="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50"
          >
            Request Browser Permission
          </button>
          <button
            type="button"
            onClick={sendNow}
            class="px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50"
          >
            Send Due Notifications Now
          </button>
        </div>
      </form>

      <div class="bg-white shadow rounded-lg p-6">
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-lg font-semibold text-gray-900">Due Now</h2>
          <button
            class="text-sm text-blue-600 hover:text-blue-700"
            onClick={loadData}
            disabled={loading()}
          >
            Refresh
          </button>
        </div>

        <Show
          when={!loading()}
          fallback={<p class="text-gray-500">Loading reminders...</p>}
        >
          <Show
            when={dueReminders().length > 0}
            fallback={<p class="text-gray-500">No due reminders.</p>}
          >
            <div class="space-y-3">
              <For each={dueReminders()}>
                {(reminder) => (
                  <div class="border border-gray-200 rounded-md p-3">
                    <div class="flex items-center justify-between">
                      <div>
                        <p class="font-medium text-gray-900">
                          {reminder.plantName}
                        </p>
                        <p class="text-sm text-gray-600">
                          {reminder.reminderType === "watering"
                            ? "Watering"
                            : "Fertilizing"}{" "}
                          due on {new Date(reminder.dueAt).toLocaleDateString()}
                        </p>
                      </div>
                      <div class="text-right">
                        <p class="text-sm text-red-600">
                          {reminder.daysOverdue > 0
                            ? `${reminder.daysOverdue} day(s) overdue`
                            : "Due today"}
                        </p>
                        <Show when={reminder.alreadySent}>
                          <p class="text-xs text-gray-500">Notified</p>
                        </Show>
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>
    </div>
  );
};
