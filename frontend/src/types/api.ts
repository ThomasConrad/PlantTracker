// This file contains only local types not covered by the generated API types
// The main API types are now generated from the OpenAPI spec in api-generated.ts

export interface CustomMetric {
  id: string;
  name: string;
  unit: string;
  dataType: 'Number' | 'Text' | 'Boolean';
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
  plantId: string;
  filename: string;
  originalFilename: string;
  size: number;
  contentType: string;
  createdAt: string;
}

// Re-export commonly used generated types for convenience
export type { components } from './api-generated';