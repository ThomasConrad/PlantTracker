import { Component, createSignal, onMount, Show } from "solid-js";
import { apiClient } from "@/api/client";
import { authStore } from "@/stores/auth";
import { Select } from "@/components/ui";

export const UserSettingsPage: Component = () => {
  const [profileSaving, setProfileSaving] = createSignal(false);
  const [passwordSaving, setPasswordSaving] = createSignal(false);
  const [exportLoading, setExportLoading] = createSignal(false);
  const [deleteLoading, setDeleteLoading] = createSignal(false);
  const [pictureSaving, setPictureSaving] = createSignal(false);

  const [success, setSuccess] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);

  const [name, setName] = createSignal("");
  const [email, setEmail] = createSignal("");
  const [firstDayOfWeek, setFirstDayOfWeek] = createSignal<"sunday" | "monday">(
    "monday",
  );
  const [preferredUnits, setPreferredUnits] = createSignal<
    "metric" | "imperial"
  >("metric");
  const [currentPassword, setCurrentPassword] = createSignal("");
  const [newPassword, setNewPassword] = createSignal("");
  const [confirmPassword, setConfirmPassword] = createSignal("");
  const [deletePassword, setDeletePassword] = createSignal("");
  const [avatarLoadError, setAvatarLoadError] = createSignal(false);
  const [pictureVersion, setPictureVersion] = createSignal<string>(
    Date.now().toString(),
  );

  // LLM settings
  const [llmSaving, setLlmSaving] = createSignal(false);
  const [llmBaseUrl, setLlmBaseUrl] = createSignal("");
  const [llmApiKey, setLlmApiKey] = createSignal("");
  const [llmApiKeySet, setLlmApiKeySet] = createSignal(false);
  const [llmModel, setLlmModel] = createSignal("");

  onMount(() => {
    if (authStore.user) {
      setName(authStore.user.name || "");
      setEmail(authStore.user.email || "");
      setFirstDayOfWeek(authStore.user.firstDayOfWeek || "monday");
      setPreferredUnits(authStore.user.preferredUnits || "metric");
      setLlmBaseUrl(authStore.user.llmBaseUrl || "");
      setLlmApiKeySet(authStore.user.llmApiKeySet || false);
      setLlmModel(authStore.user.llmModel || "");
    }
  });

  const showSuccess = (message: string) => {
    setSuccess(message);
    setTimeout(() => setSuccess(null), 4000);
  };

  const avatarUrl = () => apiClient.getProfilePictureUrl(pictureVersion());

  const handleProfilePictureUpload = async (e: Event) => {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    setError(null);
    try {
      setPictureSaving(true);
      await apiClient.uploadProfilePicture(file);
      await authStore.initializeAuth();
      setPictureVersion(Date.now().toString());
      setAvatarLoadError(false);
      showSuccess("Profile picture updated.");
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to upload profile picture",
      );
    } finally {
      setPictureSaving(false);
      input.value = "";
    }
  };

  const handleRemoveProfilePicture = async () => {
    setError(null);
    try {
      setPictureSaving(true);
      await apiClient.deleteProfilePicture();
      await authStore.initializeAuth();
      setPictureVersion(Date.now().toString());
      setAvatarLoadError(true);
      showSuccess("Profile picture removed.");
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to remove profile picture",
      );
    } finally {
      setPictureSaving(false);
    }
  };

  const handleProfileSave = async (e: Event) => {
    e.preventDefault();
    setError(null);

    try {
      setProfileSaving(true);
      await apiClient.updateProfile({
        name: name().trim(),
        email: email().trim(),
        first_day_of_week: firstDayOfWeek(),
        preferred_units: preferredUnits(),
      });
      await authStore.initializeAuth();
      showSuccess("Profile updated successfully.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update profile");
    } finally {
      setProfileSaving(false);
    }
  };

  const handlePasswordChange = async (e: Event) => {
    e.preventDefault();
    setError(null);

    if (newPassword() !== confirmPassword()) {
      setError("New passwords do not match");
      return;
    }

    try {
      setPasswordSaving(true);
      await apiClient.changePassword({
        current_password: currentPassword(),
        new_password: newPassword(),
      });

      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
      showSuccess("Password changed successfully.");
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to change password",
      );
    } finally {
      setPasswordSaving(false);
    }
  };

  const handleLlmSettingsSave = async (e: Event) => {
    e.preventDefault();
    setError(null);

    try {
      setLlmSaving(true);
      await apiClient.updateLlmSettings({
        baseUrl: llmBaseUrl().trim() || null,
        apiKey: llmApiKey().trim() || null,
        model: llmModel().trim() || null,
      });
      await authStore.initializeAuth();
      setLlmApiKey(""); // Clear from memory after save
      setLlmApiKeySet(!!llmBaseUrl().trim());
      showSuccess("AI coach settings saved.");
    } catch (err) {
      setError(
        err instanceof Error ? err.message : "Failed to save AI settings",
      );
    } finally {
      setLlmSaving(false);
    }
  };

  const handleDataExport = async () => {
    setError(null);

    try {
      setExportLoading(true);
      const payload = await apiClient.exportUserData();
      const blob = new Blob([JSON.stringify(payload, null, 2)], {
        type: "application/json",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `planty-export-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
      showSuccess("Data export downloaded.");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to export data");
    } finally {
      setExportLoading(false);
    }
  };

  const handleDeleteAccount = async () => {
    setError(null);

    if (!deletePassword()) {
      setError("Enter your current password to delete your account");
      return;
    }

    const confirmed = confirm(
      "Delete your account permanently? This removes all plants, photos, and tracking data.",
    );
    if (!confirmed) return;

    try {
      setDeleteLoading(true);
      await apiClient.deleteAccount({ current_password: deletePassword() });
      await authStore.logout();
      window.location.href = "/";
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete account");
    } finally {
      setDeleteLoading(false);
    }
  };

  return (
    <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-8 space-y-8">
      <div>
        <h1 class="text-3xl font-bold text-gray-900">Account Settings</h1>
        <p class="mt-2 text-gray-600">
          Manage your profile, password, and data privacy options.
        </p>
      </div>

      <Show when={success()}>
        <div class="bg-green-50 border border-green-200 rounded-md p-4 text-sm text-green-800">
          {success()}
        </div>
      </Show>

      <Show when={error()}>
        <div class="bg-red-50 border border-red-200 rounded-md p-4 text-sm text-red-800">
          {error()}
        </div>
      </Show>

      <form onSubmit={handleProfileSave} class="bg-white shadow rounded-lg">
        <div class="px-4 py-5 sm:p-6">
          <h2 class="text-lg font-medium text-gray-900 mb-6">
            Profile Information
          </h2>
          <div class="space-y-4">
            <div>
              <label class="block text-sm font-medium text-gray-700 mb-2">
                Profile Picture
              </label>
              <div class="flex items-center gap-4">
                <div class="h-16 w-16 rounded-full bg-gray-200 overflow-hidden flex items-center justify-center">
                  <Show
                    when={!avatarLoadError()}
                    fallback={
                      <span class="text-lg font-semibold text-gray-700">
                        {authStore.user?.name?.[0]?.toUpperCase() || "U"}
                      </span>
                    }
                  >
                    <img
                      src={avatarUrl()}
                      alt="Profile"
                      class="h-full w-full object-cover"
                      onError={() => setAvatarLoadError(true)}
                      onLoad={() => setAvatarLoadError(false)}
                    />
                  </Show>
                </div>
                <div class="flex flex-wrap gap-2">
                  <label class="inline-flex cursor-pointer justify-center py-2 px-4 text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 disabled:opacity-50">
                    {pictureSaving() ? "Uploading..." : "Upload"}
                    <input
                      type="file"
                      class="hidden"
                      accept="image/*"
                      onChange={handleProfilePictureUpload}
                      disabled={pictureSaving()}
                    />
                  </label>
                  <button
                    type="button"
                    onClick={handleRemoveProfilePicture}
                    disabled={pictureSaving() || avatarLoadError()}
                    class="inline-flex justify-center py-2 px-4 text-sm font-medium rounded-md text-gray-700 bg-gray-200 hover:bg-gray-300 disabled:opacity-50"
                  >
                    Remove
                  </button>
                </div>
              </div>
            </div>
            <div>
              <label
                for="name"
                class="block text-sm font-medium text-gray-700 mb-2"
              >
                Full Name
              </label>
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
              <label
                for="email"
                class="block text-sm font-medium text-gray-700 mb-2"
              >
                Email Address
              </label>
              <input
                id="email"
                type="email"
                value={email()}
                onInput={(e) => setEmail(e.currentTarget.value)}
                class="w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500"
                required
              />
            </div>
            <div>
              <Select
                id="first-day-of-week"
                label="First Day of Week"
                value={firstDayOfWeek()}
                onValueChange={(value) =>
                  setFirstDayOfWeek(value as "sunday" | "monday")
                }
                options={[
                  { value: "sunday", label: "Sunday" },
                  { value: "monday", label: "Monday" },
                ]}
              />
            </div>
            <div>
              <Select
                id="preferred-units"
                label="Preferred Units"
                value={preferredUnits()}
                onValueChange={(value) =>
                  setPreferredUnits(value as "metric" | "imperial")
                }
                options={[
                  { value: "metric", label: "Metric (cm, ml, g)" },
                  { value: "imperial", label: "Imperial (in, fl oz, oz)" },
                ]}
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
            {profileSaving() ? "Saving..." : "Save Profile"}
          </button>
        </div>
      </form>

      <form onSubmit={handlePasswordChange} class="bg-white shadow rounded-lg">
        <div class="px-4 py-5 sm:p-6">
          <h2 class="text-lg font-medium text-gray-900 mb-6">
            Change Password
          </h2>
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
            {passwordSaving() ? "Updating..." : "Change Password"}
          </button>
        </div>
      </form>

      <form
        onSubmit={handleLlmSettingsSave}
        class="bg-white dark:bg-gray-800 shadow rounded-lg"
      >
        <div class="px-4 py-5 sm:p-6">
          <h2 class="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
            AI Coach
          </h2>
          <p class="text-sm text-gray-500 dark:text-gray-400 mb-6">
            Connect any OpenAI-compatible API. Works with OpenAI, OpenRouter,
            Ollama, LiteLLM, and more.
          </p>
          <div class="space-y-4">
            <div>
              <label
                for="llm-base-url"
                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              >
                API Base URL
              </label>
              <input
                id="llm-base-url"
                type="url"
                value={llmBaseUrl()}
                onInput={(e) => setLlmBaseUrl(e.currentTarget.value)}
                placeholder="https://openrouter.ai/api/v1"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400"
              />
              <p class="mt-1 text-xs text-gray-400">
                OpenAI: https://api.openai.com/v1 &middot; OpenRouter:
                https://openrouter.ai/api/v1 &middot; Ollama:
                http://localhost:11434/v1
              </p>
            </div>
            <div>
              <label
                for="llm-api-key"
                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              >
                API Key
              </label>
              <input
                id="llm-api-key"
                type="password"
                value={llmApiKey()}
                onInput={(e) => setLlmApiKey(e.currentTarget.value)}
                placeholder={llmApiKeySet() ? "••••••••••••••••" : "sk-..."}
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400"
              />
              <Show when={llmApiKeySet()}>
                <p class="mt-1 text-xs text-green-600 dark:text-green-400">
                  Key is saved. Leave blank to keep current key.
                </p>
              </Show>
            </div>
            <div>
              <label
                for="llm-model"
                class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1"
              >
                Model
              </label>
              <input
                id="llm-model"
                type="text"
                value={llmModel()}
                onInput={(e) => setLlmModel(e.currentTarget.value)}
                placeholder="gpt-4o"
                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm focus:ring-green-500 focus:border-green-500 bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 placeholder:text-gray-400"
              />
              <p class="mt-1 text-xs text-gray-400">
                e.g. gpt-4o, anthropic/claude-sonnet-4-20250514, llama3.2-vision
              </p>
            </div>
          </div>
        </div>
        <div class="px-4 py-3 bg-gray-50 dark:bg-gray-700/50 text-right sm:px-6 rounded-b-lg">
          <button
            type="submit"
            disabled={llmSaving()}
            class="inline-flex justify-center py-2 px-4 text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700 disabled:opacity-50"
          >
            {llmSaving() ? "Saving..." : "Save AI Settings"}
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
            {exportLoading() ? "Exporting..." : "Export My Data"}
          </button>

          <div class="border-t pt-4">
            <p class="text-sm text-gray-600 mb-3">
              Delete account permanently (requires your current password).
            </p>
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
                {deleteLoading() ? "Deleting..." : "Delete Account"}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
