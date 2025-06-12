import { createSignal } from 'solid-js';
import { apiClient } from '@/api/client';
import type { Plant, Photo, components } from '@/types';

type TrackingEntry = components['schemas']['TrackingEntry'];
type CreateTrackingEntryRequest = components['schemas']['CreateTrackingEntryRequest'];
type TrackingEntriesResponse = components['schemas']['TrackingEntriesResponse'];
type CreatePlantRequest = components['schemas']['CreatePlantRequest'];

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

  async loadPlants(params?: { search?: string; sort?: string }): Promise<void> {
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

  async createPlant(plantData: CreatePlantRequest & { thumbnailFile?: File }): Promise<Plant> {
    try {
      setLoading(true);
      setError(null);
      
      // First create the plant with basic data
      const { thumbnailFile, ...createData } = plantData;
      const newPlant = await apiClient.createPlant(createData);
      
      let finalPlant = newPlant;
      
      // Handle thumbnail upload if a file was provided
      if (thumbnailFile instanceof File) {
        try {
          // Upload the photo
          const uploadedPhoto = await apiClient.uploadPlantPhoto(newPlant.id, thumbnailFile);
          
          // Set the uploaded photo as the plant's thumbnail
          finalPlant = await apiClient.setPlantThumbnail(newPlant.id, uploadedPhoto.id);
        } catch (photoError) {
          console.error('Failed to upload/set thumbnail for new plant:', photoError);
          // Don't throw here - the plant creation was successful, just the thumbnail failed
        }
      }
      
      setPlants(prev => [...prev, finalPlant]);
      return finalPlant;
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create plant';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  },

  async updatePlant(plantId: string, plantData: Partial<CreatePlantRequest> & { thumbnailFile?: File }): Promise<Plant> {
    try {
      setLoading(true);
      setError(null);
      
      // First update the basic plant data
      const { thumbnailFile, ...updateData } = plantData;
      const updatedPlant = await apiClient.updatePlant(plantId, updateData);
      
      let finalPlant = updatedPlant;
      
      // Handle thumbnail upload if a file was provided
      if (thumbnailFile instanceof File) {
        try {
          // Uploading thumbnail file
          
          // Upload the photo
          const uploadedPhoto = await apiClient.uploadPlantPhoto(plantId, thumbnailFile);
          // Photo uploaded successfully
          
          // Set the uploaded photo as the plant's thumbnail
          finalPlant = await apiClient.setPlantThumbnail(plantId, uploadedPhoto.id);
          // Thumbnail set successfully
          
          // Photo processing completed
        } catch (photoError) {
          console.error('Failed to upload/set thumbnail:', photoError);
          // Don't throw here - the plant update was successful, just the thumbnail failed
        }
      }
      
      setPlants(prev =>
        prev.map(plant => plant.id === plantId ? finalPlant : plant)
      );
      
      if (selectedPlant()?.id === plantId) {
        setSelectedPlant(finalPlant);
      }
      
      return finalPlant;
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

  async setPlantThumbnail(plantId: string, photoId: string): Promise<Plant> {
    try {
      setError(null);
      const updatedPlant = await apiClient.setPlantThumbnail(plantId, photoId);
      
      // Update plants in store
      setPlants(plants => 
        plants.map(plant => 
          plant.id === plantId ? updatedPlant : plant
        )
      );
      
      // Update selectedPlant if it's the one being updated
      if (selectedPlant()?.id === plantId) {
        setSelectedPlant(updatedPlant);
      }
      
      return updatedPlant;
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to set thumbnail';
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