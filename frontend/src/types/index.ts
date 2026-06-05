export * from "./api";
import type { Plant, components } from "./api";
export type { Plant };

type User = components["schemas"]["UserResponse"];
type CareTaskWithStatus = components["schemas"]["CareTaskWithStatus"];
type Measurement = components["schemas"]["Measurement"];
type TrackingEntry = components["schemas"]["TrackingEntry"];

export interface AppState {
  isAuthenticated: boolean;
  user: User | null;
  plants: Plant[];
  loading: boolean;
  error: string | null;
}

export interface CareTaskFormData {
  name: string;
  icon?: string;
  color?: string;
  intervalDays?: number;
  amount?: number;
  unit?: string;
  notes?: string;
}

export interface PlantAttributeFormData {
  key: string;
  label: string;
  value: string;
  icon?: string | null;
  category?: string | null;
  source?: "identification" | "coach" | "user";
}

export interface PlantFormData {
  name: string;
  genus: string;
  careTasks: CareTaskFormData[];
  customMetrics: {
    name: string;
    unit: string;
    dataType: "Number" | "Text" | "Boolean";
  }[];
  attributes?: PlantAttributeFormData[];
}

export interface TrackingFormData {
  careTaskIds?: string[];
  measurements?: Measurement[];
  timestamp: Date;
  notes?: string;
  photoIds?: string[];
}

export type { User, CareTaskWithStatus, Measurement, TrackingEntry };
