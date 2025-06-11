import { Component, createSignal, onMount, Show } from 'solid-js';
import { AppLayout } from '@/components/layouts/AppLayout';
import { Button } from '@/components/ui/Button';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { apiClient } from '@/api/client';

interface CalendarSubscriptionInfo {
  feedUrl: string;
  instructions: {
    general: string;
    iOS: string;
    android: string;
    outlook: string;
    apple: string;
  };
  features: string[];
}

export const CalendarSettingsPage: Component = () => {
  const [subscriptionInfo, setSubscriptionInfo] = createSignal<CalendarSubscriptionInfo | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [copied, setCopied] = createSignal(false);
  const [regenerating, setRegenerating] = createSignal(false);

  const loadSubscriptionInfo = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await apiClient.request<CalendarSubscriptionInfo>('/calendar/subscription');
      setSubscriptionInfo(response);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load calendar subscription info';
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
      console.error('Failed to copy to clipboard:', err);
    }
  };

  const regenerateToken = async () => {
    try {
      setRegenerating(true);
      setError(null);
      
      const response = await apiClient.request<{ feedUrl: string; message: string }>('/calendar/regenerate-token', {
        method: 'POST'
      });
      
      // Update the subscription info with the new URL
      const currentInfo = subscriptionInfo();
      if (currentInfo) {
        setSubscriptionInfo({
          ...currentInfo,
          feedUrl: response.feedUrl
        });
      }
      
      // Show success message
      setCopied(false); // Reset copy state
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to regenerate token';
      setError(errorMessage);
    } finally {
      setRegenerating(false);
    }
  };

  onMount(loadSubscriptionInfo);

  return (
    <AppLayout>
      <div class="max-w-4xl mx-auto px-4 py-8">
        <div class="mb-8">
          <h1 class="text-3xl font-bold text-gray-900 mb-4">Calendar Subscription</h1>
          <p class="text-lg text-gray-600">
            Subscribe to your plant care schedule in any calendar application to get automatic watering and fertilizing reminders.
          </p>
        </div>

        <Show when={loading()}>
          <div class="flex justify-center py-8">
            <LoadingSpinner />
          </div>
        </Show>

        <Show when={error()}>
          <div class="bg-red-50 border border-red-200 rounded-md p-4 mb-6">
            <div class="flex">
              <div class="ml-3">
                <h3 class="text-sm font-medium text-red-800">Error</h3>
                <div class="mt-2 text-sm text-red-700">{error()}</div>
              </div>
            </div>
          </div>
        </Show>

        <Show when={subscriptionInfo()}>
          {(info) => (
            <div class="space-y-8">
              {/* Calendar Feed URL */}
              <div class="bg-white shadow rounded-lg p-6">
                <h2 class="text-xl font-semibold text-gray-900 mb-4">Your Calendar Feed</h2>
                
                <div class="space-y-4">
                  <div>
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                      Calendar Subscription URL
                    </label>
                    <div class="flex space-x-2">
                      <input
                        type="text"
                        value={info().feedUrl}
                        readonly
                        class="flex-1 min-w-0 block w-full px-3 py-2 border border-gray-300 rounded-md text-sm bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                      />
                      <Button
                        onClick={copyToClipboard}
                        variant="outline"
                        class="px-4 py-2"
                      >
                        <Show when={copied()} fallback="Copy">
                          ✓ Copied!
                        </Show>
                      </Button>
                    </div>
                  </div>

                  <div class="flex space-x-4">
                    <Button
                      onClick={regenerateToken}
                      variant="outline"
                      disabled={regenerating()}
                      class="px-4 py-2"
                    >
                      <Show when={regenerating()} fallback="Regenerate URL">
                        <LoadingSpinner size="sm" class="mr-2" />
                        Regenerating...
                      </Show>
                    </Button>
                  </div>

                  <p class="text-sm text-gray-600">
                    Keep this URL private. If you think it has been compromised, regenerate it above.
                  </p>
                </div>
              </div>

              {/* Features */}
              <div class="bg-white shadow rounded-lg p-6">
                <h2 class="text-xl font-semibold text-gray-900 mb-4">What You'll Get</h2>
                <ul class="space-y-3">
                  {info().features.map((feature) => (
                    <li class="flex items-start">
                      <svg class="h-5 w-5 text-green-500 mt-0.5 mr-3 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                      </svg>
                      <span class="text-gray-700">{feature}</span>
                    </li>
                  ))}
                </ul>
              </div>

              {/* Instructions */}
              <div class="bg-white shadow rounded-lg p-6">
                <h2 class="text-xl font-semibold text-gray-900 mb-4">How to Subscribe</h2>
                
                <div class="mb-6">
                  <h3 class="text-lg font-medium text-gray-900 mb-2">General Instructions</h3>
                  <p class="text-gray-700">{info().instructions.general}</p>
                </div>

                <div class="grid md:grid-cols-2 gap-6">
                  <div>
                    <h3 class="text-lg font-medium text-gray-900 mb-2">📱 iOS</h3>
                    <p class="text-sm text-gray-700">{info().instructions.iOS}</p>
                  </div>

                  <div>
                    <h3 class="text-lg font-medium text-gray-900 mb-2">🤖 Android</h3>
                    <p class="text-sm text-gray-700">{info().instructions.android}</p>
                  </div>

                  <div>
                    <h3 class="text-lg font-medium text-gray-900 mb-2">🖥️ Outlook</h3>
                    <p class="text-sm text-gray-700">{info().instructions.outlook}</p>
                  </div>

                  <div>
                    <h3 class="text-lg font-medium text-gray-900 mb-2">🍎 Apple Calendar</h3>
                    <p class="text-sm text-gray-700">{info().instructions.apple}</p>
                  </div>
                </div>
              </div>

              {/* Help */}
              <div class="bg-blue-50 border border-blue-200 rounded-lg p-6">
                <h3 class="text-lg font-medium text-blue-900 mb-2">📋 Quick Setup</h3>
                <ol class="list-decimal list-inside space-y-2 text-sm text-blue-800">
                  <li>Copy the calendar subscription URL above</li>
                  <li>Open your calendar application</li>
                  <li>Look for "Add Calendar", "Subscribe to Calendar", or "Import Calendar"</li>
                  <li>Paste the URL when prompted</li>
                  <li>Your plant care reminders will now appear in your calendar!</li>
                </ol>
                
                <div class="mt-4 pt-4 border-t border-blue-200">
                  <p class="text-sm text-blue-700">
                    <strong>Note:</strong> Events will be created for the next 365 days and will update automatically when you modify your plant care schedules.
                  </p>
                </div>
              </div>
            </div>
          )}
        </Show>
      </div>
    </AppLayout>
  );
};