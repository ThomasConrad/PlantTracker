// This file contains only local types not covered by the generated API types
// The main API types are now generated from the OpenAPI spec in api-generated.ts

export interface CustomMetric {
  id: string;
  name: string;
  unit: string;
  dataType: 'Number' | 'Text' | 'Boolean';
}

// Import and re-export the generated Plant type
import type { components } from './api-generated';
export type Plant = components['schemas']['PlantResponse'];

export interface Photo {
  id: string;
  plantId: string;
  filename: string;
  originalFilename: string;
  size: number;
  contentType: string;
  thumbnailWidth?: number | null;
  thumbnailHeight?: number | null;
  createdAt: string;
}

export interface PhotoWithThumbnail extends Photo {
  fullUrl: string;
  thumbnailUrl?: string | null;
}

// Re-export commonly used generated types for convenience
export type { components } from './api-generated';