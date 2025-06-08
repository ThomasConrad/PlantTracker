export * from './api';

export interface AppState {
  isAuthenticated: boolean;
  user: User | null;
  plants: Plant[];
  loading: boolean;
  error: string | null;
}

export interface PlantFormData {
  name: string;
  genus: string;
  wateringIntervalDays: number;
  fertilizingIntervalDays: number;
  customMetrics: {
    name: string;
    unit: string;
    dataType: 'number' | 'text' | 'boolean';
  }[];
}

export interface TrackingFormData {
  type: EntryType;
  timestamp: Date;
  value?: number | string | boolean;
  notes?: string;
  metricId?: string;
}

export interface PlantStats {
  daysUntilWatering: number;
  daysUntilFertilizing: number;
  totalEntries: number;
  lastActivity: string | null;
}

import type { User, Plant, EntryType } from './api';