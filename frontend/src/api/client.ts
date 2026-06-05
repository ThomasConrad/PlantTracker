import type { Plant, Photo, components } from "@/types/api";
import { logApiSuccess, logApiError, logWarn } from "@/utils/logger";

// Generated API types
type AuthResponse = components["schemas"]["AuthResponse"];
type LoginRequest = components["schemas"]["LoginRequest"];
type CreateUserRequest = components["schemas"]["CreateUserRequest"];
type ValidateInviteRequest = components["schemas"]["ValidateInviteRequest"];
type UserResponse = components["schemas"]["UserResponse"];
type User = UserResponse;
type CreatePlantRequest = components["schemas"]["CreatePlantRequest"];
type UpdatePlantRequest = components["schemas"]["UpdatePlantRequest"];
type TrackingEntry = components["schemas"]["TrackingEntry"];
type TrackingEntriesResponse = components["schemas"]["TrackingEntriesResponse"];
type CreateTrackingEntryRequest =
  components["schemas"]["CreateTrackingEntryRequest"];
type PhotosResponse = components["schemas"]["PhotosResponse"];
type PlantsResponse = components["schemas"]["PlantsResponse"];
interface UpdateProfileRequest {
  name: string;
  email: string;
  first_day_of_week?: "sunday" | "monday";
  preferred_units?: "metric" | "imperial";
}
interface ChangePasswordRequest {
  current_password: string;
  new_password: string;
}
interface DeleteAccountRequest {
  current_password: string;
}

interface UpdateTrackingEntryRequest {
  timestamp?: string;
  value?: unknown;
  notes?: string;
}

interface WaitlistSignupRequest {
  email: string;
  name?: string;
  message?: string;
}

interface WaitlistResponse {
  id: string;
  email: string;
  name?: string;
  status: string;
  created_at: string;
}

interface ReminderPreferences {
  enabled: boolean;
  reminderTime: string;
  timezone: string;
  browserNotificationsEnabled: boolean;
  pushHealthAlerts: boolean;
  pushDailySummary: boolean;
  pushCoachSuggestions: boolean;
  pushReminders: boolean;
}

interface DueReminder {
  plantId: string;
  plantName: string;
  reminderType: string;
  dueAt: string;
  dueDate: string;
  daysOverdue: number;
  alreadySent: boolean;
}

interface DueRemindersResponse {
  reminders: DueReminder[];
  totalDue: number;
  unsentCount: number;
}

interface DispatchRemindersResponse {
  reminders: DueReminder[];
  sentCount: number;
}

// ─── Plant Identification Types ─────────────────────────────────────────────

export interface PlantCandidate {
  scientific_name: string;
  common_name: string | null;
  genus: string;
  confidence: number;
  reasoning: string;
  reference_images: string[];
  suggested_care: SuggestedCare;
  trefle_slug: string | null;
}

export interface SuggestedCare {
  watering_interval_days: number | null;
  fertilizing_interval_days: number | null;
  light_requirement: string | null;
  humidity_notes: string | null;
  temperature_notes: string | null;
  additional_notes: string | null;
}

export interface IdentifyPlantResponse {
  candidates: PlantCandidate[];
  auto_select: boolean;
  analysis_notes: string;
}

export interface SpeciesSearchResult {
  scientific_name: string;
  common_name: string | null;
  genus: string | null;
  family: string | null;
  image_url: string | null;
  slug: string;
}

export interface SearchSpeciesResponse {
  results: SpeciesSearchResult[];
}

import {
  enqueueRequest,
  getCachedResponse,
  cacheResponse,
} from "@/utils/offlineQueue";

// Frontend is always served by the backend — relative URL always works.
const API_BASE_URL = "/api/v1";

class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public data?: unknown,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  getBaseUrl(): string {
    return this.baseUrl;
  }

  async request<T>(
    endpoint: string,
    options: Partial<{
      method: string;
      headers: Record<string, string>;
      body: string;
    }> = {},
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const method = options.method || "GET";
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      ...(options.headers || {}),
    };

    const startTime = performance.now();
    let parsedBody: unknown;
    try {
      parsedBody = options.body ? JSON.parse(options.body) : undefined;
    } catch {
      parsedBody = options.body;
    }

    try {
      const response = await fetch(url, {
        method,
        body: options.body,
        headers,
        credentials: "include",
      });

      const durationMs = Math.round(performance.now() - startTime);

      if (!response.ok) {
        const errorData = await response.json().catch(() => {
          logWarn("Failed to parse error response as JSON", { url, method, status: response.status });
          return {};
        });
        logApiError({
          method,
          url,
          status: response.status,
          durationMs,
          requestBody: parsedBody,
          responseBody: errorData,
        });
        throw new ApiError(
          errorData.message || `HTTP ${response.status}`,
          response.status,
          errorData,
        );
      }

      // Handle responses with no content (204 No Content, or empty response)
      if (
        response.status === 204 ||
        response.headers.get("content-length") === "0"
      ) {
        logApiSuccess({ method, url, status: response.status, durationMs });
        // Cache successful GET responses
        if (method === "GET") {
          cacheResponse(url, "", response.status);
        }
        return undefined as T;
      }

      // Check if response has content to parse
      const contentType = response.headers.get("content-type");
      if (contentType && contentType.includes("application/json")) {
        const data = await response.json();
        logApiSuccess({ method, url, status: response.status, durationMs, responseBody: data });
        // Cache successful GET responses
        if (method === "GET") {
          cacheResponse(url, JSON.stringify(data), response.status);
        }
        return data;
      }

      logApiSuccess({ method, url, status: response.status, durationMs });
      return undefined as T;
    } catch (error) {
      // If it's an ApiError (server responded with an error), don't queue
      if (error instanceof ApiError) {
        throw error;
      }

      const durationMs = Math.round(performance.now() - startTime);
      logApiError({
        method,
        url,
        durationMs,
        requestBody: parsedBody,
        error,
        extra: { offline: !navigator.onLine },
      });

      // Network error — handle offline
      if (method === "GET") {
        // For reads, try to return cached data
        const cached = await getCachedResponse(url);
        if (cached) {
          if (cached.body) {
            return JSON.parse(cached.body) as T;
          }
          return undefined as T;
        }
        throw new ApiError("You appear to be offline", 0, {});
      }

      // For mutations, queue for later replay
      await enqueueRequest(method, url, options.body || null, headers);
      throw new ApiError(
        "You're offline. This action will sync when you reconnect.",
        0,
        { queued: true },
      );
    }
  }

  async login(credentials: LoginRequest): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>("/auth/login", {
      method: "POST",
      body: JSON.stringify(credentials),
    });
    return response;
  }

  async register(userData: CreateUserRequest): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>("/auth/register", {
      method: "POST",
      body: JSON.stringify(userData),
    });
    return response;
  }

  async validateInvite(
    request: ValidateInviteRequest,
  ): Promise<{ valid: boolean; uses_remaining: number }> {
    return this.request<{ valid: boolean; uses_remaining: number }>(
      "/invites/validate",
      {
        method: "POST",
        body: JSON.stringify(request),
      },
    );
  }

  async joinWaitlist(
    request: WaitlistSignupRequest,
  ): Promise<WaitlistResponse> {
    return this.request<WaitlistResponse>("/invites/waitlist", {
      method: "POST",
      body: JSON.stringify(request),
    });
  }

  async getCurrentUser(): Promise<User> {
    return this.request<User>("/auth/me");
  }

  async updateProfile(request: UpdateProfileRequest): Promise<User> {
    return this.request<User>("/auth/profile", {
      method: "PUT",
      body: JSON.stringify(request),
    });
  }

  async changePassword(
    request: ChangePasswordRequest,
  ): Promise<{ success: boolean; message: string }> {
    return this.request<{ success: boolean; message: string }>(
      "/auth/change-password",
      {
        method: "POST",
        body: JSON.stringify(request),
      },
    );
  }

  async updateLlmSettings(settings: {
    baseUrl: string | null;
    apiKey: string | null;
    model: string | null;
  }): Promise<unknown> {
    return this.request("/auth/llm-settings", {
      method: "PUT",
      body: JSON.stringify(settings),
    });
  }

  async exportUserData(): Promise<unknown> {
    return this.request<unknown>("/auth/export");
  }

  async deleteAccount(
    request: DeleteAccountRequest,
  ): Promise<{ success: boolean; message: string }> {
    return this.request<{ success: boolean; message: string }>(
      "/auth/account",
      {
        method: "DELETE",
        body: JSON.stringify(request),
      },
    );
  }

  async uploadProfilePicture(
    file: File,
  ): Promise<{ success: boolean; message: string }> {
    const formData = new FormData();
    formData.append("file", file);

    const response = await fetch(`${this.baseUrl}/auth/profile-picture`, {
      method: "POST",
      body: formData,
      credentials: "include",
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ApiError(
        errorData.message || `HTTP ${response.status}`,
        response.status,
        errorData,
      );
    }

    return response.json();
  }

  async deleteProfilePicture(): Promise<void> {
    await this.request("/auth/profile-picture", {
      method: "DELETE",
    });
  }

  getProfilePictureUrl(cacheBust?: string): string {
    const query = cacheBust ? `?v=${encodeURIComponent(cacheBust)}` : "";
    return `${this.baseUrl}/auth/profile-picture${query}`;
  }

  async logout(): Promise<void> {
    await this.request("/auth/logout", {
      method: "POST",
    });
  }

  async getPlants(params?: {
    limit?: number;
    offset?: number;
    search?: string;
    sort?: string;
    includeArchived?: boolean;
  }): Promise<PlantsResponse> {
    const searchParams = new URLSearchParams();
    if (params?.limit) searchParams.set("limit", params.limit.toString());
    if (params?.offset) searchParams.set("offset", params.offset.toString());
    if (params?.search) searchParams.set("search", params.search);
    if (params?.sort) searchParams.set("sort", params.sort);
    if (params?.includeArchived) searchParams.set("includeArchived", "true");

    const query = searchParams.toString();
    return this.request<PlantsResponse>(`/plants${query ? `?${query}` : ""}`);
  }

  async getPlant(plantId: string): Promise<Plant> {
    return this.request<Plant>(`/plants/${plantId}`);
  }

  async createPlant(plantData: CreatePlantRequest): Promise<Plant> {
    return this.request<Plant>("/plants", {
      method: "POST",
      body: JSON.stringify(plantData),
    });
  }

  async updatePlant(
    plantId: string,
    plantData: UpdatePlantRequest,
  ): Promise<Plant> {
    return this.request<Plant>(`/plants/${plantId}`, {
      method: "PUT",
      body: JSON.stringify(plantData),
    });
  }

  async deletePlant(plantId: string): Promise<void> {
    await this.request(`/plants/${plantId}`, {
      method: "DELETE",
    });
  }

  async archivePlant(plantId: string): Promise<Plant> {
    return this.request<Plant>(`/plants/${plantId}/archive`, {
      method: "POST",
    });
  }

  async unarchivePlant(plantId: string): Promise<Plant> {
    return this.request<Plant>(`/plants/${plantId}/unarchive`, {
      method: "POST",
    });
  }

  async getPlantPhotos(
    plantId: string,
    params?: { limit?: number; offset?: number },
  ): Promise<PhotosResponse> {
    const searchParams = new URLSearchParams();
    if (params?.limit) searchParams.set("limit", params.limit.toString());
    if (params?.offset) searchParams.set("offset", params.offset.toString());

    const query = searchParams.toString();
    return this.request<PhotosResponse>(
      `/plants/${plantId}/photos${query ? `?${query}` : ""}`,
    );
  }

  async uploadPlantPhoto(
    plantId: string,
    file: File,
    caption?: string,
  ): Promise<Photo> {
    const formData = new FormData();
    formData.append("file", file);
    if (caption) formData.append("caption", caption);

    const response = await fetch(`${this.baseUrl}/plants/${plantId}/photos`, {
      method: "POST",
      body: formData,
      credentials: "include", // Include cookies for session auth
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ApiError(
        errorData.message || `HTTP ${response.status}`,
        response.status,
        errorData,
      );
    }

    return response.json();
  }

  async clearPlantPreview(plantId: string): Promise<Plant> {
    const response = await fetch(`${this.baseUrl}/plants/${plantId}/preview`, {
      method: "DELETE",
      credentials: "include",
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ApiError(
        errorData.message || `HTTP ${response.status}`,
        response.status,
        errorData,
      );
    }

    return response.json();
  }

  async setPlantPreview(plantId: string, photoId: string): Promise<Plant> {
    const response = await fetch(
      `${this.baseUrl}/plants/${plantId}/preview/${photoId}`,
      {
        method: "PUT",
        credentials: "include",
      },
    );

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ApiError(
        errorData.message || `HTTP ${response.status}`,
        response.status,
        errorData,
      );
    }

    return response.json();
  }

  async deletePlantPhoto(plantId: string, photoId: string): Promise<void> {
    await this.request(`/plants/${plantId}/photos/${photoId}`, {
      method: "DELETE",
    });
  }

  getPhotoUrl(plantId: string, photoId: string): string {
    return `${this.baseUrl}/plants/${plantId}/photos/${photoId}`;
  }

  async getTrackingEntries(
    plantId: string,
    params?: {
      type?: string;
      from?: string;
      to?: string;
      limit?: number;
      offset?: number;
    },
  ): Promise<TrackingEntriesResponse> {
    const searchParams = new URLSearchParams();
    if (params?.type) searchParams.set("type", params.type);
    if (params?.from) searchParams.set("from", params.from);
    if (params?.to) searchParams.set("to", params.to);
    if (params?.limit) searchParams.set("limit", params.limit.toString());
    if (params?.offset) searchParams.set("offset", params.offset.toString());

    const query = searchParams.toString();
    return this.request<TrackingEntriesResponse>(
      `/plants/${plantId}/entries${query ? `?${query}` : ""}`,
    );
  }

  async createTrackingEntry(
    plantId: string,
    entryData: CreateTrackingEntryRequest,
  ): Promise<TrackingEntry> {
    return this.request<TrackingEntry>(`/plants/${plantId}/entries`, {
      method: "POST",
      body: JSON.stringify(entryData),
    });
  }

  async updateTrackingEntry(
    plantId: string,
    entryId: string,
    entryData: UpdateTrackingEntryRequest,
  ): Promise<TrackingEntry> {
    return this.request<TrackingEntry>(
      `/plants/${plantId}/entries/${entryId}`,
      {
        method: "PUT",
        body: JSON.stringify(entryData),
      },
    );
  }

  async deleteTrackingEntry(plantId: string, entryId: string): Promise<void> {
    await this.request(`/plants/${plantId}/entries/${entryId}`, {
      method: "DELETE",
    });
  }

  async getReminderPreferences(): Promise<ReminderPreferences> {
    return this.request<ReminderPreferences>("/reminders/preferences");
  }

  async updateReminderPreferences(
    payload: ReminderPreferences,
  ): Promise<ReminderPreferences> {
    return this.request<ReminderPreferences>("/reminders/preferences", {
      method: "PUT",
      body: JSON.stringify(payload),
    });
  }

  async getDueReminders(): Promise<DueRemindersResponse> {
    return this.request<DueRemindersResponse>("/reminders/due");
  }

  async dispatchDueReminders(): Promise<DispatchRemindersResponse> {
    return this.request<DispatchRemindersResponse>("/reminders/dispatch", {
      method: "POST",
    });
  }

  // ─── Plant Identification ───────────────────────────────────────────────────

  async identifyPlant(
    imageUrl: string,
    context?: string,
  ): Promise<IdentifyPlantResponse> {
    return this.request<IdentifyPlantResponse>("/plants/identify", {
      method: "POST",
      body: JSON.stringify({ image_url: imageUrl, context }),
    });
  }

  async searchSpecies(query: string): Promise<SearchSpeciesResponse> {
    const encoded = encodeURIComponent(query);
    return this.request<SearchSpeciesResponse>(
      `/plants/species-search?q=${encoded}`,
    );
  }

  // ─── Push Notifications ───────────────────────────────────────────────────

  async getVapidPublicKey(): Promise<{ publicKey: string }> {
    return this.request<{ publicKey: string }>("/push/vapid-key");
  }

  async subscribePush(subscription: PushSubscription): Promise<void> {
    const key = subscription.getKey("p256dh");
    const auth = subscription.getKey("auth");
    if (!key || !auth) throw new Error("Invalid push subscription keys");

    await this.request("/push/subscribe", {
      method: "POST",
      body: JSON.stringify({
        endpoint: subscription.endpoint,
        keys: {
          p256dh: arrayBufferToBase64Url(key),
          auth: arrayBufferToBase64Url(auth),
        },
      }),
    });
  }

  async unsubscribePush(endpoint: string): Promise<void> {
    await this.request("/push/unsubscribe", {
      method: "DELETE",
      body: JSON.stringify({ endpoint }),
    });
  }

  async testPush(): Promise<{ sent: number }> {
    return this.request<{ sent: number }>("/push/test", {
      method: "POST",
    });
  }
}

/** Convert ArrayBuffer to URL-safe base64 (no padding) */
function arrayBufferToBase64Url(buffer: ArrayBuffer): string {
  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

export const apiClient = new ApiClient();
export { ApiError };
