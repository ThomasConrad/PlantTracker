import { Component, createEffect, createSignal, Show } from 'solid-js';
import { useNavigate, useParams } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { PlantForm } from '@/components/plants/PlantForm';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import type { PlantFormData } from '@/types';

export const EditPlantPage: Component = () => {
  const navigate = useNavigate();
  const params = useParams();
  const [loading, setLoading] = createSignal(false);

  createEffect(() => {
    if (params.id) {
      plantsStore.loadPlant(params.id);
    }
  });

  const handleSubmit = async (data: PlantFormData & { thumbnailFile?: File }) => {
    try {
      setLoading(true);
      
      // Update the plant first
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
    } catch (error) {
      console.error('Failed to update plant:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="space-y-6">
      <div>
        <h1 class="text-2xl font-bold text-gray-900">Edit Plant</h1>
        <p class="text-gray-600">Update your plant's information and care settings.</p>
      </div>

      <Show
        when={plantsStore.selectedPlant && !plantsStore.loading}
        fallback={
          <div class="flex justify-center py-12">
            <LoadingSpinner size="lg" />
          </div>
        }
      >
        <div class="bg-white shadow-sm rounded-lg border border-gray-200">
          <div class="p-6">
            <PlantForm
              initialData={plantsStore.selectedPlant || undefined}
              onSubmit={handleSubmit}
              submitText="Update Plant"
              loading={loading()}
            />
          </div>
        </div>
      </Show>

      {plantsStore.error && (
        <div class="bg-red-50 border border-red-200 rounded-md p-4">
          <p class="text-sm text-red-600">{plantsStore.error}</p>
        </div>
      )}
    </div>
  );
};