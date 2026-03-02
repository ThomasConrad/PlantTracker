import { Component, createSignal, onMount, Show } from 'solid-js';
import { apiClient } from '@/api/client';
import { authStore } from '@/stores/auth';

export const UserSettingsPage: Component = () => {
  const [profileSaving, setProfileSaving] = createSignal(false);
  const [passwordSaving, setPasswordSaving] = createSignal(false);
  const [exportLoading, setExportLoading] = createSignal(false);
  const [deleteLoading, setDeleteLoading] = createSignal(false);

  const [success, setSuccess] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  const [name, setName] = createSignal('');
  const [email, setEmail] = createSignal('');
  const [currentPassword, setCurrentPassword] = createSignal('');
  const [newPassword, setNewPassword] = createSignal('');
  const [confirmPassword, setConfirmPassword] = createSignal('');
  const [deletePassword, setDeletePassword] = createSignal('');

  onMount(() => {
    if (authStore.user) {
      setName(authStore.user.name || '');
      setEmail(authStore.user.email || '');
    }
  });

  const showSuccess = (message: string) => {
    setSuccess(message);
    setTimeout(() => setSuccess(null), 4000);
  };

  const handleProfileSave = async (e: Event) => {
    e.preventDefault();
    setError(null);

    try {
      setProfileSaving(true);
      await apiClient.updateProfile({ name: name().trim(), email: email().trim() });
      await authStore.initializeAuth();
      showSuccess('Profile updated successfully.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update profile');
    } finally {
      setProfileSaving(false);
    }
  };

  const handlePasswordChange = async (e: Event) => {
    e.preventDefault();
    setError(null);

    if (newPassword() !== confirmPassword()) {
      setError('New passwords do not match');
      return;
    }

    try {
      setPasswordSaving(true);
      await apiClient.changePassword({
        current_password: currentPassword(),
        new_password: newPassword(),
      });

      setCurrentPassword('');
      setNewPassword('');
      setConfirmPassword('');
      showSuccess('Password changed successfully.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to change password');
    } finally {
      setPasswordSaving(false);
    }
  };

  const handleDataExport = async () => {
    setError(null);

    try {
      setExportLoading(true);
      const payload = await apiClient.exportUserData();
      const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `planty-export-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
      showSuccess('Data export downloaded.');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to export data');
    } finally {
      setExportLoading(false);
    }
  };

  const handleDeleteAccount = async () => {
    setError(null);

    if (!deletePassword()) {
      setError('Enter your current password to delete your account');
      return;
    }

    const confirmed = confirm(
      'Delete your account permanently? This removes all plants, photos, and tracking data.'
    );
    if (!confirmed) return;

    try {
      setDeleteLoading(true);
      await apiClient.deleteAccount({ current_password: deletePassword() });
      await authStore.logout();
      window.location.href = '/';
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete account');
    } finally {
      setDeleteLoading(false);
    }
  };

  return (
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
      <div>
        <h1 class="text-3xl font-bold text-gray-900">Account Settings</h1>
        <p class="mt-2 text-gray-600">Manage your profile, password, and data privacy options.</p>
      </div>

      <Show when={success()}>
        <div class="bg-green-50 border border-green-200 rounded-md p-4 text-sm text-green-800">{success()}</div>
      </Show>

      <Show when={error()}>
        <div class="bg-red-50 border border-red-200 rounded-md p-4 text-sm text-red-800">{error()}</div>
      </Show>

      <form onSubmit={handleProfileSave} class="bg-white shadow rounded-lg">
        <div class="px-4 py-5 sm:p-6">
          <h2 class="text-lg font-medium text-gray-900 mb-6">Profile Information</h2>
          <div class="space-y-4">
            <div>
              <label for="name" class="block text-sm font-medium text-gray-700 mb-2">Full Name</label>
              <input
                id="name"
                type="text"
                value={name()}
                onInput={(e) => setName(e.currentTarget.value)}
                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500"
                required
              />
            </div>
            <div>
              <label for="email" class="block text-sm font-medium text-gray-700 mb-2">Email Address</label>
              <input
                id="email"
                type="email"
                value={email()}
                onInput={(e) => setEmail(e.currentTarget.value)}
                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500"
                required
              />
            </div>
          </div>
        </div>
        <div class="px-4 py-3 bg-gray-50 text-right sm:px-6 rounded-b-lg">
          <button
            type="submit"
            disabled={profileSaving()}
            class="inline-flex justify-center py-2 px-4 text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 disabled:opacity-50"
          >
            {profileSaving() ? 'Saving...' : 'Save Profile'}
          </button>
        </div>
      </form>

      <form onSubmit={handlePasswordChange} class="bg-white shadow rounded-lg">
        <div class="px-4 py-5 sm:p-6">
          <h2 class="text-lg font-medium text-gray-900 mb-6">Change Password</h2>
          <div class="space-y-4">
            <input
              type="password"
              placeholder="Current password"
              value={currentPassword()}
              onInput={(e) => setCurrentPassword(e.currentTarget.value)}
              class="w-full px-3 py-2 border border-gray-300 rounded-md"
              required
            />
            <input
              type="password"
              placeholder="New password"
              value={newPassword()}
              onInput={(e) => setNewPassword(e.currentTarget.value)}
              class="w-full px-3 py-2 border border-gray-300 rounded-md"
              minlength="8"
              required
            />
            <input
              type="password"
              placeholder="Confirm new password"
              value={confirmPassword()}
              onInput={(e) => setConfirmPassword(e.currentTarget.value)}
              class="w-full px-3 py-2 border border-gray-300 rounded-md"
              minlength="8"
              required
            />
          </div>
        </div>
        <div class="px-4 py-3 bg-gray-50 text-right sm:px-6 rounded-b-lg">
          <button
            type="submit"
            disabled={passwordSaving()}
            class="inline-flex justify-center py-2 px-4 text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 disabled:opacity-50"
          >
            {passwordSaving() ? 'Updating...' : 'Change Password'}
          </button>
        </div>
      </form>

      <div class="bg-white shadow rounded-lg">
        <div class="px-4 py-5 sm:p-6 space-y-4">
          <h2 class="text-lg font-medium text-gray-900">Data & Privacy</h2>

          <button
            type="button"
            onClick={handleDataExport}
            disabled={exportLoading()}
            class="inline-flex justify-center py-2 px-4 text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 disabled:opacity-50"
          >
            {exportLoading() ? 'Exporting...' : 'Export My Data'}
          </button>

          <div class="border-t pt-4">
            <p class="text-sm text-gray-600 mb-3">Delete account permanently (requires your current password).</p>
            <input
              type="password"
              placeholder="Current password"
              value={deletePassword()}
              onInput={(e) => setDeletePassword(e.currentTarget.value)}
              class="w-full max-w-md px-3 py-2 border border-gray-300 rounded-md mb-3"
            />
            <div>
              <button
                type="button"
                onClick={handleDeleteAccount}
                disabled={deleteLoading()}
                class="inline-flex justify-center py-2 px-4 text-sm font-medium rounded-md text-white bg-red-600 hover:bg-red-700 disabled:opacity-50"
              >
                {deleteLoading() ? 'Deleting...' : 'Delete Account'}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
