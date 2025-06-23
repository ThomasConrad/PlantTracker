import { Component, createSignal, onMount, Show } from 'solid-js';
import { A } from '@solidjs/router';

interface AdminSettings {
  max_total_users: number;
  default_user_invite_limit: number;
  registration_enabled: boolean;
}

export const AdminSettingsPage: Component = () => {
  const [settings, setSettings] = createSignal<AdminSettings | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [success, setSuccess] = createSignal(false);

  // Form state
  const [maxUsers, setMaxUsers] = createSignal(1000);
  const [defaultInviteLimit, setDefaultInviteLimit] = createSignal(5);
  const [registrationEnabled, setRegistrationEnabled] = createSignal(true);

  const loadSettings = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await fetch('/api/v1/admin/settings', {
        credentials: 'include'
      });
      
      if (!response.ok) {
        throw new Error('Failed to load settings');
      }
      
      const data = await response.json();
      setSettings(data);
      
      // Update form state
      setMaxUsers(data.max_total_users);
      setDefaultInviteLimit(data.default_user_invite_limit);
      setRegistrationEnabled(data.registration_enabled);
    } catch (err) {
      console.error('Error loading settings:', err);
      setError(err instanceof Error ? err.message : 'Failed to load settings');
    } finally {
      setLoading(false);
    }
  };

  const saveSettings = async (e: Event) => {
    e.preventDefault();
    
    try {
      setSaving(true);
      setError(null);
      setSuccess(false);
      
      const response = await fetch('/api/v1/admin/settings', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
        },
        credentials: 'include',
        body: JSON.stringify({
          max_total_users: maxUsers(),
          default_user_invite_limit: defaultInviteLimit(),
          registration_enabled: registrationEnabled(),
        }),
      });
      
      if (!response.ok) {
        throw new Error('Failed to save settings');
      }
      
      const data = await response.json();
      setSettings(data);
      setSuccess(true);
      
      // Hide success message after 3 seconds
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      console.error('Error saving settings:', err);
      setError(err instanceof Error ? err.message : 'Failed to save settings');
    } finally {
      setSaving(false);
    }
  };

  onMount(() => {
    loadSettings();
  });

  return (
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div class="mb-8">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-3xl font-bold text-gray-900">System Settings</h1>
            <p class="mt-2 text-gray-600">Configure global system settings and limitations</p>
          </div>
          <A
            href="/admin/dashboard"
            class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
          >
            ← Back to Dashboard
          </A>
        </div>
      </div>

      <Show 
        when={!loading() && !error()} 
        fallback={
          <Show 
            when={loading()} 
            fallback={
              <div class="text-center py-12">
                <div class="text-red-600">{error()}</div>
                <button 
                  onClick={loadSettings}
                  class="mt-4 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                >
                  Retry
                </button>
              </div>
            }
          >
            <div class="flex justify-center py-12">
              <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-green-600"></div>
            </div>
          </Show>
        }
      >
        <div class="space-y-8">
          {/* Success Message */}
          <Show when={success()}>
            <div class="bg-green-50 border border-green-200 rounded-md p-4">
              <div class="flex">
                <div class="flex-shrink-0">
                  <svg class="h-5 w-5 text-green-400" fill="currentColor" viewBox="0 0 20 20">
                    <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd" />
                  </svg>
                </div>
                <div class="ml-3">
                  <p class="text-sm text-green-800">Settings saved successfully!</p>
                </div>
              </div>
            </div>
          </Show>

          {/* Settings Form */}
          <form onSubmit={saveSettings} class="bg-white shadow rounded-lg">
            <div class="px-4 py-5 sm:p-6">
              <h2 class="text-lg font-medium text-gray-900 mb-6">System Configuration</h2>
              
              <div class="space-y-6">
                {/* Max Total Users */}
                <div>
                  <label for="max-users" class="block text-sm font-medium text-gray-700 mb-2">
                    Maximum Total Users
                  </label>
                  <input
                    type="number"
                    id="max-users"
                    value={maxUsers()}
                    onInput={(e) => setMaxUsers(parseInt(e.currentTarget.value) || 0)}
                    min="1"
                    max="100000"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500"
                  />
                  <p class="mt-1 text-sm text-gray-500">
                    Maximum number of user accounts that can be created in the system.
                  </p>
                </div>

                {/* Default Invite Limit */}
                <div>
                  <label for="invite-limit" class="block text-sm font-medium text-gray-700 mb-2">
                    Default User Invite Limit
                  </label>
                  <input
                    type="number"
                    id="invite-limit"
                    value={defaultInviteLimit()}
                    onInput={(e) => setDefaultInviteLimit(parseInt(e.currentTarget.value) || 0)}
                    min="0"
                    max="1000"
                    class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500"
                  />
                  <p class="mt-1 text-sm text-gray-500">
                    Default number of invites new users can create. Set to 0 to disable invite creation for regular users.
                  </p>
                </div>

                {/* Registration Enabled */}
                <div>
                  <div class="flex items-center">
                    <input
                      type="checkbox"
                      id="registration-enabled"
                      checked={registrationEnabled()}
                      onChange={(e) => setRegistrationEnabled(e.currentTarget.checked)}
                      class="h-4 w-4 text-green-600 focus:ring-green-500 border-gray-300 rounded"
                    />
                    <label for="registration-enabled" class="ml-2 block text-sm font-medium text-gray-700">
                      Enable Registration
                    </label>
                  </div>
                  <p class="mt-1 text-sm text-gray-500">
                    Allow new users to register with valid invite codes. If disabled, only existing admins can create accounts.
                  </p>
                </div>
              </div>
            </div>

            <div class="px-4 py-3 bg-gray-50 text-right sm:px-6 rounded-b-lg">
              <button
                type="submit"
                disabled={saving()}
                class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <Show when={saving()}>
                  <svg class="animate-spin -ml-1 mr-3 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                  </svg>
                </Show>
                {saving() ? 'Saving...' : 'Save Settings'}
              </button>
            </div>
          </form>

          {/* System Information */}
          <div class="bg-white shadow rounded-lg">
            <div class="px-4 py-5 sm:p-6">
              <h2 class="text-lg font-medium text-gray-900 mb-6">Current System Status</h2>
              
              <Show when={settings()}>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
                  <div class="bg-blue-50 p-4 rounded-lg">
                    <div class="text-2xl font-bold text-blue-600">{settings()?.max_total_users}</div>
                    <div class="text-sm text-blue-800">Max Users Allowed</div>
                  </div>
                  
                  <div class="bg-green-50 p-4 rounded-lg">
                    <div class="text-2xl font-bold text-green-600">{settings()?.default_user_invite_limit}</div>
                    <div class="text-sm text-green-800">Default Invite Limit</div>
                  </div>
                  
                  <div class="bg-purple-50 p-4 rounded-lg">
                    <div class="text-2xl font-bold text-purple-600">
                      {settings()?.registration_enabled ? 'Enabled' : 'Disabled'}
                    </div>
                    <div class="text-sm text-purple-800">Registration Status</div>
                  </div>
                </div>
              </Show>
            </div>
          </div>

          {/* Warning Notice */}
          <div class="bg-yellow-50 border border-yellow-200 rounded-md p-4">
            <div class="flex">
              <div class="flex-shrink-0">
                <svg class="h-5 w-5 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                </svg>
              </div>
              <div class="ml-3">
                <h3 class="text-sm font-medium text-yellow-800">Important Notice</h3>
                <div class="mt-2 text-sm text-yellow-700">
                  <p>
                    Changes to these settings affect the entire system. Be careful when modifying user limits 
                    and registration settings as they may impact existing users and new registrations.
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};