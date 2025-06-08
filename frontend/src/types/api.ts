export interface User {
  id: string;
  email: string;
  name: string;
  createdAt: string;
  updatedAt: string;
}

export interface CustomMetric {
  id: string;
  name: string;
  unit: string;
  dataType: 'number' | 'text' | 'boolean';
}

export interface Plant {
  id: string;
  name: string;
  genus: string;
  wateringIntervalDays: number;
  fertilizingIntervalDays: number;
  customMetrics: CustomMetric[];
  lastWatered: string | null;
  lastFertilized: string | null;
  createdAt: string;
  updatedAt: string;
  userId: string;
}

export interface Photo {
  id: string;
  url: string;
  thumbnailUrl: string;
  caption: string | null;
  createdAt: string;
  plantId: string;
}

export type EntryType = 'watering' | 'fertilizing' | 'custom_metric';

export interface TrackingEntry {
  id: string;
  type: EntryType;
  timestamp: string;
  value: number | string | boolean;
  notes: string | null;
  metricId: string | null;
  plantId: string;
  createdAt: string;
  updatedAt: string;
}

export interface CreatePlantRequest {
  name: string;
  genus: string;
  wateringIntervalDays: number;
  fertilizingIntervalDays: number;
  customMetrics?: {
    name: string;
    unit: string;
    dataType: 'number' | 'text' | 'boolean';
  }[];
}

export interface UpdatePlantRequest {
  name?: string;
  genus?: string;
  wateringIntervalDays?: number;
  fertilizingIntervalDays?: number;
  customMetrics?: {
    id?: string;
    name: string;
    unit: string;
    dataType: 'number' | 'text' | 'boolean';
  }[];
}

export interface CreateTrackingEntryRequest {
  type: EntryType;
  timestamp: string;
  value?: number | string | boolean;
  notes?: string;
  metricId?: string;
}

export interface UpdateTrackingEntryRequest {
  timestamp?: string;
  value?: number | string | boolean;
  notes?: string;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
  name: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface PaginatedResponse<T> {
  total: number;
  limit: number;
  offset: number;
  data: T[];
}

export interface PlantsResponse extends PaginatedResponse<Plant> {
  plants: Plant[];
}

export interface PhotosResponse extends PaginatedResponse<Photo> {
  photos: Photo[];
}

export interface TrackingEntriesResponse extends PaginatedResponse<TrackingEntry> {
  entries: TrackingEntry[];
}