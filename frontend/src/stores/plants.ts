import { createSignal } from 'solid-js';
import { apiClient } from '@/api/client';
import type { Plant, Photo, components } from '@/types';

type CreatePlantRequest = components['schemas']['CreatePlantRequest'];
type TrackingEntry = components['schemas']['TrackingEntry'];
type CreateTrackingEntryRequest = components['schemas']['CreateTrackingEntryRequest'];
type TrackingEntriesResponse = components['schemas']['TrackingEntriesResponse'];

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

const [plants, setPlants] = createSignal<Plant[]>([]);
const [selectedPlant, setSelectedPlant] = createSignal<Plant | null>(null);
const [loading, setLoading] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);

const plantsStore = {
  get plants() {
    return plants();
  },
  get selectedPlant() {
    return selectedPlant();
  },
  get loading() {
    return loading();
  },
  get error() {
    return error();
  },

  async loadPlants(params?: { search?: string }): Promise<void> {
    try {
      setLoading(true);
      setError(null);
      const response = await apiClient.getPlants(params);
      setPlants(response.plants);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load plants';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  },

  async loadPlant(plantId: string): Promise<void> {
    try {
      setLoading(true);
      setError(null);
      const plant = await apiClient.getPlant(plantId);
      setSelectedPlant(plant);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load plant';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  },

  async createPlant(plantData: CreatePlantRequest): Promise<Plant> {
    try {
      setLoading(true);
      setError(null);
      const newPlant = await apiClient.createPlant(plantData);
      setPlants(prev => [...prev, newPlant]);
      return newPlant;
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create plant';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  },

  async updatePlant(plantId: string, plantData: UpdatePlantRequest): Promise<Plant> {
    try {
      setLoading(true);
      setError(null);
      const updatedPlant = await apiClient.updatePlant(plantId, plantData);
      
      setPlants(prev =>
        prev.map(plant => plant.id === plantId ? updatedPlant : plant)
      );
      
      if (selectedPlant()?.id === plantId) {
        setSelectedPlant(updatedPlant);
      }
      
      return updatedPlant;
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update plant';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  },

  async deletePlant(plantId: string): Promise<void> {
    try {
      setLoading(true);
      setError(null);
      await apiClient.deletePlant(plantId);
      setPlants(prev => prev.filter(plant => plant.id !== plantId));
      
      if (selectedPlant()?.id === plantId) {
        setSelectedPlant(null);
      }
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete plant';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  },

  async uploadPhoto(plantId: string, file: File, caption?: string): Promise<Photo> {
    try {
      setError(null);
      return await apiClient.uploadPlantPhoto(plantId, file, caption);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to upload photo';
      setError(errorMessage);
      throw err;
    }
  },

  async createTrackingEntry(
    plantId: string,
    entryData: CreateTrackingEntryRequest
  ): Promise<TrackingEntry> {
    try {
      setError(null);
      const entry = await apiClient.createTrackingEntry(plantId, entryData);
      
      if (entryData.entryType === 'watering' || entryData.entryType === 'fertilizing') {
        const plantToUpdate = plants().find(p => p.id === plantId);
        if (plantToUpdate) {
          const updatedPlant = {
            ...plantToUpdate,
            lastWatered: entryData.entryType === 'watering' ? entryData.timestamp : plantToUpdate.lastWatered,
            lastFertilized: entryData.entryType === 'fertilizing' ? entryData.timestamp : plantToUpdate.lastFertilized,
          };
          
          setPlants(prev =>
            prev.map(plant => plant.id === plantId ? updatedPlant : plant)
          );
          
          if (selectedPlant()?.id === plantId) {
            setSelectedPlant(updatedPlant);
          }
        }
      }
      
      return entry;
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create tracking entry';
      setError(errorMessage);
      throw err;
    }
  },

  async updateTrackingEntry(
    plantId: string,
    entryId: string,
    entryData: UpdateTrackingEntryRequest
  ): Promise<TrackingEntry> {
    try {
      setError(null);
      return await apiClient.updateTrackingEntry(plantId, entryId, entryData);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update tracking entry';
      setError(errorMessage);
      throw err;
    }
  },

  async deleteTrackingEntry(plantId: string, entryId: string): Promise<void> {
    try {
      setError(null);
      await apiClient.deleteTrackingEntry(plantId, entryId);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete tracking entry';
      setError(errorMessage);
      throw err;
    }
  },

  async getTrackingEntries(plantId: string): Promise<TrackingEntriesResponse> {
    try {
      setError(null);
      return await apiClient.getTrackingEntries(plantId);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to get tracking entries';
      setError(errorMessage);
      throw err;
    }
  },

  clearError(): void {
    setError(null);
  },

  clearSelectedPlant(): void {
    setSelectedPlant(null);
  },
};

export { plantsStore };