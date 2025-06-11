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

interface GoogleCalendarStatus {
  connected: boolean;
  connected_at?: string;
  scopes?: string[];
  expires_at?: string;
}

export const CalendarSettingsPage: Component = () => {
  const [subscriptionInfo, setSubscriptionInfo] = createSignal<CalendarSubscriptionInfo | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [copied, setCopied] = createSignal(false);
  const [regenerating, setRegenerating] = createSignal(false);
  
  // Google Calendar state
  const [googleStatus, setGoogleStatus] = createSignal<GoogleCalendarStatus | null>(null);
  const [googleLoading, setGoogleLoading] = createSignal(false);
  const [googleError, setGoogleError] = createSignal<string | null>(null);
  const [syncing, setSyncing] = createSignal(false);

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

  // Google Calendar functions
  const loadGoogleStatus = async () => {
    try {
      setGoogleLoading(true);
      setGoogleError(null);
      
      const response = await apiClient.request<GoogleCalendarStatus>('/google-calendar/status');
      setGoogleStatus(response);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load Google Calendar status';
      setGoogleError(errorMessage);
    } finally {
      setGoogleLoading(false);
    }
  };

  const connectGoogleCalendar = async () => {
    try {
      setGoogleLoading(true);
      setGoogleError(null);
      
      const response = await apiClient.request<{ auth_url: string; state: string }>('/google-calendar/auth-url');
      
      // Redirect to Google OAuth
      window.location.href = response.auth_url;
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to get Google authorization URL';
      setGoogleError(errorMessage);
      setGoogleLoading(false);
    }
  };

  const disconnectGoogleCalendar = async () => {
    try {
      setGoogleLoading(true);
      setGoogleError(null);
      
      await apiClient.request('/google-calendar/disconnect', {
        method: 'POST'
      });
      
      setGoogleStatus({ connected: false });
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to disconnect Google Calendar';
      setGoogleError(errorMessage);
    } finally {
      setGoogleLoading(false);
    }
  };

  const syncPlantReminders = async () => {
    try {
      setSyncing(true);
      setGoogleError(null);
      
      const response = await apiClient.request<{ success: boolean; message: string; events_created: number }>('/google-calendar/sync-reminders', {
        method: 'POST',
        body: JSON.stringify({
          days_ahead: 365,
          replace_existing: false
        })
      });
      
      // Show success message
      alert(`Success! ${response.message}`);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to sync plant reminders';
      setGoogleError(errorMessage);
    } finally {
      setSyncing(false);
    }
  };

  onMount(() => {
    loadSubscriptionInfo();
    loadGoogleStatus();
  });

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

        {/* Google Calendar Integration */}
        <div class="bg-white shadow rounded-lg p-6">
          <div class="flex items-center justify-between mb-4">
            <div>
              <h2 class="text-xl font-semibold text-gray-900">Google Calendar Integration</h2>
              <p class="text-sm text-gray-600 mt-1">
                Automatically add plant care reminders directly to your Google Calendar
              </p>
            </div>
            <div class="flex items-center">
              <Show when={googleStatus()?.connected} fallback={
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                  Not Connected
                </span>
              }>
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800">
                  ✓ Connected
                </span>
              </Show>
            </div>
          </div>

          <Show when={googleError()}>
            <div class="bg-red-50 border border-red-200 rounded-md p-4 mb-4">
              <div class="text-sm text-red-700">{googleError()}</div>
            </div>
          </Show>

          <Show when={googleLoading()}>
            <div class="flex justify-center py-4">
              <LoadingSpinner size="sm" />
            </div>
          </Show>

          <Show when={!googleLoading()}>
            <Show when={googleStatus()?.connected} fallback={
              <div class="space-y-4">
                <div class="flex items-start space-x-3">
                  <svg class="h-8 w-8 text-blue-500 mt-1" fill="currentColor" viewBox="0 0 24 24">
                    <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
                    <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
                    <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
                    <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
                  </svg>
                  <div class="flex-1">
                    <h3 class="text-lg font-medium text-gray-900">Connect with Google Calendar</h3>
                    <p class="text-sm text-gray-600 mt-1">
                      Connect your Google Calendar to automatically create plant care reminders as events.
                      This provides better integration than iCalendar subscriptions, allowing you to:
                    </p>
                    <ul class="mt-2 text-sm text-gray-600 space-y-1">
                      <li>• Get notifications on your devices</li>
                      <li>• Mark tasks as complete</li>
                      <li>• Have events appear immediately</li>
                      <li>• Customize event details</li>
                    </ul>
                  </div>
                </div>
                <Button
                  onClick={connectGoogleCalendar}
                  disabled={googleLoading()}
                  class="w-full sm:w-auto"
                >
                  <Show when={googleLoading()} fallback="Connect Google Calendar">
                    <LoadingSpinner size="sm" class="mr-2" />
                    Connecting...
                  </Show>
                </Button>
              </div>
            }>
              <div class="space-y-4">
                <div class="flex items-center justify-between">
                  <div>
                    <p class="text-sm text-gray-600">
                      Connected on {googleStatus()?.connected_at ? new Date(googleStatus()!.connected_at!).toLocaleDateString() : 'Unknown'}
                    </p>
                    <Show when={googleStatus()?.expires_at}>
                      <p class="text-xs text-gray-500">
                        Access expires: {new Date(googleStatus()!.expires_at!).toLocaleDateString()}
                      </p>
                    </Show>
                  </div>
                  <Button
                    onClick={disconnectGoogleCalendar}
                    variant="outline"
                    disabled={googleLoading()}
                    class="text-red-600 border-red-300 hover:bg-red-50"
                  >
                    Disconnect
                  </Button>
                </div>

                <div class="flex space-x-4">
                  <Button
                    onClick={syncPlantReminders}
                    disabled={syncing()}
                    class="flex-1 sm:flex-none"
                  >
                    <Show when={syncing()} fallback="Sync Plant Reminders">
                      <LoadingSpinner size="sm" class="mr-2" />
                      Syncing...
                    </Show>
                  </Button>
                </div>

                <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
                  <h4 class="text-sm font-medium text-blue-900 mb-2">How it works:</h4>
                  <ul class="text-sm text-blue-800 space-y-1">
                    <li>• Click "Sync Plant Reminders" to create events for the next year</li>
                    <li>• Events will appear in your Google Calendar immediately</li>
                    <li>• Re-sync anytime to update your schedule with new plants</li>
                    <li>• Events include plant details and care instructions</li>
                  </ul>
                </div>
              </div>
            </Show>
          </Show>
        </div>

        <Show when={subscriptionInfo()}>
          {(info) => (
            <div class="space-y-8">
              {/* iCalendar Subscription */}
              <div class="bg-white shadow rounded-lg p-6">
                <h2 class="text-xl font-semibold text-gray-900 mb-2">iCalendar Subscription</h2>
                <p class="text-sm text-gray-600 mb-4">
                  Alternative method: Subscribe to an iCalendar feed in any calendar application
                </p>
                
                <h3 class="text-lg font-medium text-gray-900 mb-4">Your Calendar Feed</h3>
                
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
                <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 mt-6">
                  <h4 class="text-sm font-medium text-blue-900 mb-2">📋 Quick Setup</h4>
                  <ol class="list-decimal list-inside space-y-1 text-sm text-blue-800">
                    <li>Copy the calendar subscription URL above</li>
                    <li>Open your calendar application</li>
                    <li>Look for "Add Calendar", "Subscribe to Calendar", or "Import Calendar"</li>
                    <li>Paste the URL when prompted</li>
                    <li>Your plant care reminders will now appear in your calendar!</li>
                  </ol>
                  
                  <div class="mt-3 pt-3 border-t border-blue-200">
                    <p class="text-xs text-blue-700">
                      <strong>Note:</strong> Events will be created for the next 365 days and will update automatically when you modify your plant care schedules.
                    </p>
                  </div>
                </div>
              </div>
            </div>
          )}
        </Show>
      </div>
    </AppLayout>
  );
};