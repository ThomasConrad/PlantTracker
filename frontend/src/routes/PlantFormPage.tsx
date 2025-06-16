import { Component, createEffect, createSignal, Show } from 'solid-js';
import { A, useNavigate, useParams } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { PlantForm } from '@/components/plants/PlantForm';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { apiClient } from '@/api/client';
import type { PlantFormData, Photo } from '@/types';

export const PlantFormPage: Component = () => {
  const navigate = useNavigate();
  const params = useParams();
  const [loading, setLoading] = createSignal(false);
  const [existingPhotos, setExistingPhotos] = createSignal<Photo[]>([]);

  const isEditing = () => !!params.id;

  // Load existing photos for edit mode
  const loadExistingPhotos = async (plantId: string) => {
    try {
      const photosResponse = await apiClient.getPlantPhotos(plantId);
      setExistingPhotos(photosResponse.photos || []);
    } catch (error) {
      console.error('Failed to load existing photos:', error);
      setExistingPhotos([]);
    }
  };

  createEffect(() => {
    if (params.id) {
      plantsStore.loadPlant(params.id);
      loadExistingPhotos(params.id);
    }
  });

  // Handle selecting an existing photo as thumbnail
  const handlePhotoSelect = async (photoId: string) => {
    if (!params.id) return;
    try {
      await plantsStore.setPlantThumbnail(params.id, photoId);
      // Reload plant data to get updated thumbnail
      await plantsStore.loadPlant(params.id);
    } catch (error) {
      console.error('Failed to set thumbnail:', error);
    }
  };

  // Handle clearing the current thumbnail
  const handleClearThumbnail = async () => {
    if (!params.id) return;
    try {
      // The API doesn't have a clear thumbnail endpoint, but we could implement it
      // For now, we'll just indicate that no thumbnail is selected
      console.log('Clear thumbnail - this would need an API endpoint');
    } catch (error) {
      console.error('Failed to clear thumbnail:', error);
    }
  };

  const handleSubmit = async (data: PlantFormData & { thumbnailFile?: File }) => {
    try {
      setLoading(true);
      
      if (isEditing()) {
        // Update existing plant
        await plantsStore.updatePlant(params.id, data);
        
        // If a thumbnail file was provided, upload it and set as thumbnail
        if (data.thumbnailFile) {
          try {
            const photo = await plantsStore.uploadPhoto(params.id, data.thumbnailFile);
            await plantsStore.setPlantThumbnail(params.id, photo.id);
          } catch (thumbnailError) {
            console.warn('Failed to upload thumbnail:', thumbnailError);
          }
        }
        
        navigate(`/plants/${params.id}`);
      } else {
        // Create new plant
        const plant = await plantsStore.createPlant(data);
        navigate(`/plants/${plant.id}`);
      }
    } catch (error) {
      console.error(`Failed to ${isEditing() ? 'update' : 'create'} plant:`, error);
    } finally {
      setLoading(false);
    }
  };

  const getBackUrl = () => {
    if (isEditing()) {
      return `/plants/${params.id}`;
    }
    return '/plants';
  };

  const getTitle = () => isEditing() ? 'Edit Plant' : 'Add New Plant';
  const getDescription = () => 
    isEditing() 
      ? "Update your plant's information and care settings."
      : "Track your plant's care and growth over time.";
  const getSubmitText = () => isEditing() ? 'Update Plant' : 'Create Plant';

  return (
    <div class="min-h-full pb-20 sm:pb-8">
      {/* Header Section with improved mobile margins */}
      <div class="px-4 sm:px-6 pt-4 sm:pt-6 pb-6">
        <div class="max-w-4xl mx-auto">
          <div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4">
            <div class="flex-1">
              <div class="flex items-center space-x-3 sm:space-x-4">
                <A
                  href={getBackUrl()}
                  class="flex-shrink-0 p-2 -ml-2 text-gray-400 hover:text-gray-600 hover:bg-gray-50 rounded-lg transition-colors duration-200"
                  aria-label="Back"
                >
                  <svg class="h-5 w-5 sm:h-6 sm:w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
                  </svg>
                </A>
                <div class="min-w-0 flex-1">
                  <h1 class="text-xl sm:text-2xl lg:text-3xl font-bold text-gray-900 truncate">
                    {getTitle()}
                  </h1>
                  <p class="mt-1 text-sm sm:text-base text-gray-600 line-clamp-2">
                    {getDescription()}
                  </p>
                </div>
              </div>
            </div>
            
            {/* Status indicator for edit mode */}
            <Show when={isEditing()}>
              <div class="flex-shrink-0">
                <div class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                  <svg class="mr-1.5 h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                  Editing
                </div>
              </div>
            </Show>
          </div>
        </div>
      </div>

      {/* Main Content */}
      <div class="px-4 sm:px-6">
        <div class="max-w-4xl mx-auto">
          <Show
            when={!isEditing() || (plantsStore.selectedPlant && !plantsStore.loading)}
            fallback={
              <div class="flex justify-center py-16 sm:py-20">
                <div class="flex flex-col items-center gap-4">
                  <LoadingSpinner size="lg" />
                  <div class="text-center">
                    <p class="text-sm sm:text-base text-gray-500 font-medium">
                      Loading plant details...
                    </p>
                    <p class="text-xs sm:text-sm text-gray-400 mt-1">
                      Please wait while we fetch your plant information
                    </p>
                  </div>
                </div>
              </div>
            }
          >
            {/* Form Card */}
            <div class="bg-white shadow-sm rounded-xl sm:rounded-2xl border border-gray-200 overflow-hidden">
              {/* Card Header */}
              <div class="px-4 sm:px-6 py-4 sm:py-5 border-b border-gray-100 bg-gray-50/50">
                <div class="flex items-center space-x-3">
                  <div class="flex-shrink-0">
                    <div class="w-8 h-8 sm:w-10 sm:h-10 bg-primary-100 rounded-lg flex items-center justify-center">
                      <svg class="h-4 w-4 sm:h-5 sm:w-5 text-primary-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <Show 
                          when={isEditing()}
                          fallback={
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
                          }
                        >
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                        </Show>
                      </svg>
                    </div>
                  </div>
                  <div>
                    <h2 class="text-base sm:text-lg font-semibold text-gray-900">
                      Plant Details
                    </h2>
                    <p class="text-xs sm:text-sm text-gray-500">
                      {isEditing() ? 'Update the information below' : 'Fill in the details for your new plant'}
                    </p>
                  </div>
                </div>
              </div>

              {/* Form Content */}
              <div class="p-4 sm:p-6 lg:p-8">
                <PlantForm
                  initialData={isEditing() && plantsStore.selectedPlant ? {
                    name: plantsStore.selectedPlant.name,
                    genus: plantsStore.selectedPlant.genus,
                    wateringSchedule: plantsStore.selectedPlant.wateringSchedule?.intervalDays ? {
                      intervalDays: plantsStore.selectedPlant.wateringSchedule.intervalDays,
                      amount: plantsStore.selectedPlant.wateringSchedule.amount || undefined,
                      unit: plantsStore.selectedPlant.wateringSchedule.unit || undefined,
                      notes: plantsStore.selectedPlant.wateringSchedule.notes || undefined,
                    } : undefined,
                    fertilizingSchedule: plantsStore.selectedPlant.fertilizingSchedule?.intervalDays ? {
                      intervalDays: plantsStore.selectedPlant.fertilizingSchedule.intervalDays,
                      amount: plantsStore.selectedPlant.fertilizingSchedule.amount || undefined,
                      unit: plantsStore.selectedPlant.fertilizingSchedule.unit || undefined,
                      notes: plantsStore.selectedPlant.fertilizingSchedule.notes || undefined,
                    } : undefined,
                    customMetrics: plantsStore.selectedPlant.customMetrics?.map(m => ({
                      name: m.name,
                      unit: m.unit,
                      dataType: m.dataType as 'Number' | 'Text' | 'Boolean'
                    })) || []
                  } : undefined}
                  existingThumbnailUrl={isEditing() && plantsStore.selectedPlant?.thumbnailUrl ? plantsStore.selectedPlant.thumbnailUrl : null}
                  existingPhotos={existingPhotos()}
                  onSubmit={handleSubmit}
                  onPhotoSelect={handlePhotoSelect}
                  onClearThumbnail={handleClearThumbnail}
                  submitText={getSubmitText()}
                  loading={loading()}
                />
              </div>
            </div>
          </Show>

          {/* Error Message */}
          <Show when={plantsStore.error}>
            <div class="mt-6 bg-red-50 border border-red-200 rounded-xl p-4">
              <div class="flex items-start gap-3">
                <div class="flex-shrink-0">
                  <svg class="h-5 w-5 text-red-500 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                </div>
                <div class="flex-1 min-w-0">
                  <h3 class="text-sm font-medium text-red-800">
                    Something went wrong
                  </h3>
                  <p class="text-sm text-red-700 mt-1">
                    {plantsStore.error}
                  </p>
                </div>
              </div>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
}; 