export * from './api';
import type { Plant, components } from './api';

type User = components['schemas']['UserResponse'];
type EntryType = components['schemas']['EntryType'];

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
    dataType: 'Number' | 'Text' | 'Boolean';
  }[];
}

export interface TrackingFormData {
  entryType: EntryType;
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

export type { User, EntryType };