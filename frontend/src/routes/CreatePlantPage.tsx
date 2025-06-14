import { Component, createSignal } from 'solid-js';
import { useNavigate } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { PlantForm } from '@/components/plants/PlantForm';
import type { PlantFormData } from '@/types';

export const CreatePlantPage: Component = () => {
  const navigate = useNavigate();
  const [loading, setLoading] = createSignal(false);

  const handleSubmit = async (data: PlantFormData) => {
    try {
      setLoading(true);
      
      const plant = await plantsStore.createPlant(data);
      navigate(`/plants/${plant.id}`);
    } catch (error) {
      console.error('Failed to create plant:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="space-y-6">
      <div>
        <h1 class="text-2xl font-bold text-gray-900">Add New Plant</h1>
        <p class="text-gray-600">Track your plant's care and growth over time.</p>
      </div>

      <div class="bg-white shadow-sm rounded-lg border border-gray-200">
        <div class="p-6">
          <PlantForm
            onSubmit={handleSubmit}
            submitText="Create Plant"
            loading={loading()}
          />
        </div>
      </div>

      {plantsStore.error && (
        <div class="bg-red-50 border border-red-200 rounded-md p-4">
          <p class="text-sm text-red-600">{plantsStore.error}</p>
        </div>
      )}
    </div>
  );
};