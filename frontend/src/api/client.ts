import type { Plant, Photo, components } from '@/types/api';

// Generated API types
type AuthResponse = components['schemas']['AuthResponse'];
type LoginRequest = components['schemas']['LoginRequest'];
type UserResponse = components['schemas']['UserResponse'];
type User = UserResponse;
type CreatePlantRequest = components['schemas']['CreatePlantRequest'];
type TrackingEntry = components['schemas']['TrackingEntry'];
type TrackingEntriesResponse = components['schemas']['TrackingEntriesResponse'];
type CreateTrackingEntryRequest = components['schemas']['CreateTrackingEntryRequest'];
type PhotosResponse = components['schemas']['PhotosResponse'];

// Local types for compatibility
interface RegisterRequest {
  email: string;
  password: string;
  name: string;
}

interface UpdatePlantRequest {
  name?: string;
  genus?: string;
  wateringIntervalDays?: number;
  fertilizingIntervalDays?: number;
}

interface UpdateTrackingEntryRequest {
  timestamp?: string;
  value?: unknown;
  notes?: string;
}

interface PlantsResponse {
  plants: Plant[];
  total: number;
}

const API_BASE_URL = (import.meta as { env?: { VITE_API_URL?: string } }).env?.VITE_API_URL || '/api/v1';

class ApiError extends Error {
  constructor(
    message: string,
    public status: number,
    public data?: unknown
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string,
    options: Partial<{
      method: string;
      headers: Record<string, string>;
      body: string;
    }> = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...(options.headers || {}),
    };

    const response = await fetch(url, {
      method: options.method || 'GET',
      body: options.body,
      headers,
      credentials: 'include', // Include cookies for session auth
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ApiError(
        errorData.message || `HTTP ${response.status}`,
        response.status,
        errorData
      );
    }

    return response.json();
  }

  async login(credentials: LoginRequest): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>('/auth/login', {
      method: 'POST',
      body: JSON.stringify(credentials),
    });
    return response;
  }

  async register(userData: RegisterRequest): Promise<AuthResponse> {
    const response = await this.request<AuthResponse>('/auth/register', {
      method: 'POST',
      body: JSON.stringify(userData),
    });
    return response;
  }

  async getCurrentUser(): Promise<User> {
    return this.request<User>('/auth/me');
  }

  async logout(): Promise<void> {
    await this.request('/auth/logout', {
      method: 'POST',
    });
  }

  async getPlants(params?: {
    limit?: number;
    offset?: number;
    search?: string;
  }): Promise<PlantsResponse> {
    const searchParams = new URLSearchParams();
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    if (params?.offset) searchParams.set('offset', params.offset.toString());
    if (params?.search) searchParams.set('search', params.search);

    const query = searchParams.toString();
    return this.request<PlantsResponse>(`/plants${query ? `?${query}` : ''}`);
  }

  async getPlant(plantId: string): Promise<Plant> {
    return this.request<Plant>(`/plants/${plantId}`);
  }

  async createPlant(plantData: CreatePlantRequest): Promise<Plant> {
    return this.request<Plant>('/plants', {
      method: 'POST',
      body: JSON.stringify(plantData),
    });
  }

  async updatePlant(plantId: string, plantData: UpdatePlantRequest): Promise<Plant> {
    return this.request<Plant>(`/plants/${plantId}`, {
      method: 'PUT',
      body: JSON.stringify(plantData),
    });
  }

  async deletePlant(plantId: string): Promise<void> {
    await this.request(`/plants/${plantId}`, {
      method: 'DELETE',
    });
  }

  async getPlantPhotos(
    plantId: string,
    params?: { limit?: number; offset?: number }
  ): Promise<PhotosResponse> {
    const searchParams = new URLSearchParams();
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    if (params?.offset) searchParams.set('offset', params.offset.toString());

    const query = searchParams.toString();
    return this.request<PhotosResponse>(
      `/plants/${plantId}/photos${query ? `?${query}` : ''}`
    );
  }

  async uploadPlantPhoto(
    plantId: string,
    file: File,
    caption?: string
  ): Promise<Photo> {
    const formData = new FormData();
    formData.append('file', file);
    if (caption) formData.append('caption', caption);

    const response = await fetch(`${this.baseUrl}/plants/${plantId}/photos`, {
      method: 'POST',
      body: formData,
      credentials: 'include', // Include cookies for session auth
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new ApiError(
        errorData.message || `HTTP ${response.status}`,
        response.status,
        errorData
      );
    }

    return response.json();
  }

  async deletePlantPhoto(plantId: string, photoId: string): Promise<void> {
    await this.request(`/plants/${plantId}/photos/${photoId}`, {
      method: 'DELETE',
    });
  }

  getPhotoUrl(plantId: string, photoId: string): string {
    return `${this.baseUrl}/plants/${plantId}/photos/${photoId}`;
  }

  getThumbnailUrl(plantId: string, photoId: string): string {
    return `${this.baseUrl}/plants/${plantId}/photos/${photoId}/thumbnail`;
  }

  async setPlantThumbnail(plantId: string, photoId: string): Promise<Plant> {
    return this.request<Plant>(`/plants/${plantId}/thumbnail/${photoId}`, {
      method: 'PUT',
    });
  }

  async getTrackingEntries(
    plantId: string,
    params?: {
      type?: string;
      from?: string;
      to?: string;
      limit?: number;
      offset?: number;
    }
  ): Promise<TrackingEntriesResponse> {
    const searchParams = new URLSearchParams();
    if (params?.type) searchParams.set('type', params.type);
    if (params?.from) searchParams.set('from', params.from);
    if (params?.to) searchParams.set('to', params.to);
    if (params?.limit) searchParams.set('limit', params.limit.toString());
    if (params?.offset) searchParams.set('offset', params.offset.toString());

    const query = searchParams.toString();
    return this.request<TrackingEntriesResponse>(
      `/plants/${plantId}/entries${query ? `?${query}` : ''}`
    );
  }

  async createTrackingEntry(
    plantId: string,
    entryData: CreateTrackingEntryRequest
  ): Promise<TrackingEntry> {
    return this.request<TrackingEntry>(`/plants/${plantId}/entries`, {
      method: 'POST',
      body: JSON.stringify(entryData),
    });
  }

  async updateTrackingEntry(
    plantId: string,
    entryId: string,
    entryData: UpdateTrackingEntryRequest
  ): Promise<TrackingEntry> {
    return this.request<TrackingEntry>(`/plants/${plantId}/entries/${entryId}`, {
      method: 'PUT',
      body: JSON.stringify(entryData),
    });
  }

  async deleteTrackingEntry(plantId: string, entryId: string): Promise<void> {
    await this.request(`/plants/${plantId}/entries/${entryId}`, {
      method: 'DELETE',
    });
  }
}

export const apiClient = new ApiClient();
export { ApiError };