import { Component, createSignal, onMount, Show } from 'solid-js';
import { authStore } from '@/stores/auth';

export const UserSettingsPage: Component = () => {
  const [loading, setLoading] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [success, setSuccess] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // Form state
  const [name, setName] = createSignal('');
  const [email, setEmail] = createSignal('');
  const [currentPassword, setCurrentPassword] = createSignal('');
  const [newPassword, setNewPassword] = createSignal('');
  const [confirmPassword, setConfirmPassword] = createSignal('');

  // Notification preferences
  const [emailNotifications, setEmailNotifications] = createSignal(true);
  const [careReminders, setCareReminders] = createSignal(true);
  const [weeklyDigest, setWeeklyDigest] = createSignal(false);

  // Privacy settings
  const [dataExportLoading, setDataExportLoading] = createSignal(false);
  const [deleteAccountLoading, setDeleteAccountLoading] = createSignal(false);

  onMount(() => {
    // Initialize form with current user data
    if (authStore.user) {
      setName(authStore.user.name || '');
      setEmail(authStore.user.email || '');
    }
  });

  const handleProfileSave = async (e: Event) => {
    e.preventDefault();
    
    try {
      setSaving(true);
      setError(null);
      setSuccess(false);
      
      // TODO: Implement profile update API call
      // const response = await apiClient.updateProfile({
      //   name: name(),
      //   email: email(),
      // });
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      console.error('Error saving profile:', err);
      setError(err instanceof Error ? err.message : 'Failed to save profile');
    } finally {
      setSaving(false);
    }
  };

  const handlePasswordChange = async (e: Event) => {
    e.preventDefault();
    
    if (newPassword() !== confirmPassword()) {
      setError('New passwords do not match');
      return;
    }
    
    if (newPassword().length < 8) {
      setError('Password must be at least 8 characters long');
      return;
    }
    
    try {
      setSaving(true);
      setError(null);
      setSuccess(false);
      
      // TODO: Implement password change API call
      // const response = await apiClient.changePassword({
      //   current_password: currentPassword(),
      //   new_password: newPassword(),
      // });
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      console.error('Error changing password:', err);
      setError(err instanceof Error ? err.message : 'Failed to change password');
    } finally {
      setSaving(false);
    }
  };

  const handleDataExport = async () => {
    try {
      setDataExportLoading(true);
      
      // TODO: Implement data export API call
      // const response = await apiClient.exportUserData();
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      // For now, just show success message
      alert('Data export functionality will be implemented soon. You will receive an email with your data.');
    } catch (err) {
      console.error('Error exporting data:', err);
      setError('Failed to export data');
    } finally {
      setDataExportLoading(false);
    }
  };

  const handleDeleteAccount = async () => {
    const confirmed = confirm(
      'Are you sure you want to delete your account? This action cannot be undone and will permanently delete all your plant data.'
    );
    
    if (!confirmed) return;
    
    const doubleConfirm = confirm(
      'This is your final warning. Deleting your account will permanently remove all your plants, photos, and tracking data. Type YES to confirm.'
    );
    
    if (!doubleConfirm) return;
    
    try {
      setDeleteAccountLoading(true);
      
      // TODO: Implement account deletion API call
      // const response = await apiClient.deleteAccount();
      
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 2000));
      
      alert('Account deletion functionality will be implemented soon.');
    } catch (err) {
      console.error('Error deleting account:', err);
      setError('Failed to delete account');
    } finally {
      setDeleteAccountLoading(false);
    }
  };

  return (
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div class="mb-8">
        <h1 class="text-3xl font-bold text-gray-900">Account Settings</h1>
        <p class="mt-2 text-gray-600">Manage your account preferences and privacy settings</p>
      </div>

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

        {/* Error Message */}
        <Show when={error()}>
          <div class="bg-red-50 border border-red-200 rounded-md p-4">
            <div class="flex">
              <div class="flex-shrink-0">
                <svg class="h-5 w-5 text-red-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd" />
                </svg>
              </div>
              <div class="ml-3">
                <p class="text-sm text-red-800">{error()}</p>
              </div>
            </div>
          </div>
        </Show>

        {/* Profile Information */}
        <form onSubmit={handleProfileSave} class="bg-white shadow rounded-lg">
          <div class="px-4 py-5 sm:p-6">
            <h2 class="text-lg font-medium text-gray-900 mb-6">Profile Information</h2>
            
            <div class="space-y-6">
              <div>
                <label for="name" class="block text-sm font-medium text-gray-700 mb-2">
                  Full Name
                </label>
                <input
                  type="text"
                  id="name"
                  value={name()}
                  onInput={(e) => setName(e.currentTarget.value)}
                  class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500"
                />
              </div>

              <div>
                <label for="email" class="block text-sm font-medium text-gray-700 mb-2">
                  Email Address
                </label>
                <input
                  type="email"
                  id="email"
                  value={email()}
                  onInput={(e) => setEmail(e.currentTarget.value)}
                  class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500"
                />
              </div>

              <div class="text-sm text-gray-500">
                <p>Role: <span class="font-medium">{authStore.user?.role || 'User'}</span></p>
                <Show when={authStore.user?.canCreateInvites}>
                  <p>Invite permissions: <span class="font-medium">Can create invites</span></p>
                </Show>
              </div>
            </div>
          </div>

          <div class="px-4 py-3 bg-gray-50 text-right sm:px-6 rounded-b-lg">
            <button
              type="submit"
              disabled={saving()}
              class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50"
            >
              {saving() ? 'Saving...' : 'Save Profile'}
            </button>
          </div>
        </form>

        {/* Change Password */}
        <form onSubmit={handlePasswordChange} class="bg-white shadow rounded-lg">
          <div class="px-4 py-5 sm:p-6">
            <h2 class="text-lg font-medium text-gray-900 mb-6">Change Password</h2>
            
            <div class="space-y-6">
              <div>
                <label for="current-password" class="block text-sm font-medium text-gray-700 mb-2">
                  Current Password
                </label>
                <input
                  type="password"
                  id="current-password"
                  value={currentPassword()}
                  onInput={(e) => setCurrentPassword(e.currentTarget.value)}
                  class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500"
                />
              </div>

              <div>
                <label for="new-password" class="block text-sm font-medium text-gray-700 mb-2">
                  New Password
                </label>
                <input
                  type="password"
                  id="new-password"
                  value={newPassword()}
                  onInput={(e) => setNewPassword(e.currentTarget.value)}
                  class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500"
                />
              </div>

              <div>
                <label for="confirm-password" class="block text-sm font-medium text-gray-700 mb-2">
                  Confirm New Password
                </label>
                <input
                  type="password"
                  id="confirm-password"
                  value={confirmPassword()}
                  onInput={(e) => setConfirmPassword(e.currentTarget.value)}
                  class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500"
                />
              </div>
            </div>
          </div>

          <div class="px-4 py-3 bg-gray-50 text-right sm:px-6 rounded-b-lg">
            <button
              type="submit"
              disabled={saving() || !currentPassword() || !newPassword() || !confirmPassword()}
              class="inline-flex justify-center py-2 px-4 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50"
            >
              {saving() ? 'Changing...' : 'Change Password'}
            </button>
          </div>
        </form>

        {/* Notification Preferences */}
        <div class="bg-white shadow rounded-lg">
          <div class="px-4 py-5 sm:p-6">
            <h2 class="text-lg font-medium text-gray-900 mb-6">Notification Preferences</h2>
            
            <div class="space-y-4">
              <div class="flex items-center">
                <input
                  type="checkbox"
                  id="email-notifications"
                  checked={emailNotifications()}
                  onChange={(e) => setEmailNotifications(e.currentTarget.checked)}
                  class="h-4 w-4 text-green-600 focus:ring-green-500 border-gray-300 rounded"
                />
                <label for="email-notifications" class="ml-2 block text-sm text-gray-900">
                  Email notifications
                </label>
              </div>

              <div class="flex items-center">
                <input
                  type="checkbox"
                  id="care-reminders"
                  checked={careReminders()}
                  onChange={(e) => setCareReminders(e.currentTarget.checked)}
                  class="h-4 w-4 text-green-600 focus:ring-green-500 border-gray-300 rounded"
                />
                <label for="care-reminders" class="ml-2 block text-sm text-gray-900">
                  Plant care reminders
                </label>
              </div>

              <div class="flex items-center">
                <input
                  type="checkbox"
                  id="weekly-digest"
                  checked={weeklyDigest()}
                  onChange={(e) => setWeeklyDigest(e.currentTarget.checked)}
                  class="h-4 w-4 text-green-600 focus:ring-green-500 border-gray-300 rounded"
                />
                <label for="weekly-digest" class="ml-2 block text-sm text-gray-900">
                  Weekly progress digest
                </label>
              </div>
            </div>

            <p class="mt-4 text-sm text-gray-500">
              Note: Notification preferences are currently for display only and will be implemented in a future update.
            </p>
          </div>
        </div>

        {/* Privacy & Data */}
        <div class="bg-white shadow rounded-lg">
          <div class="px-4 py-5 sm:p-6">
            <h2 class="text-lg font-medium text-gray-900 mb-6">Privacy & Data</h2>
            
            <div class="space-y-6">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="text-sm font-medium text-gray-900">Export your data</h3>
                  <p class="text-sm text-gray-500">Download a copy of all your plant data</p>
                </div>
                <button
                  onClick={handleDataExport}
                  disabled={dataExportLoading()}
                  class="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
                >
                  {dataExportLoading() ? 'Exporting...' : 'Export Data'}
                </button>
              </div>

              <div class="border-t border-gray-200 pt-6">
                <div class="flex items-center justify-between">
                  <div>
                    <h3 class="text-sm font-medium text-red-900">Delete your account</h3>
                    <p class="text-sm text-red-600">Permanently delete your account and all data</p>
                  </div>
                  <button
                    onClick={handleDeleteAccount}
                    disabled={deleteAccountLoading()}
                    class="inline-flex items-center px-4 py-2 border border-red-300 shadow-sm text-sm font-medium rounded-md text-red-700 bg-white hover:bg-red-50 disabled:opacity-50"
                  >
                    {deleteAccountLoading() ? 'Deleting...' : 'Delete Account'}
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};