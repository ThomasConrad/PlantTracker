import { Component, createEffect, createSignal, Show } from 'solid-js';
import { A, useNavigate, useParams } from '@solidjs/router';
import { plantsStore } from '@/stores/plants';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { Button } from '@/components/ui/Button';
import { ActivityLog } from '@/components/plants/ActivityLog';
import { PhotoGallery } from '@/components/plants/PhotoGallery';
import { formatDate } from '@/utils/date';

export const PlantDetailPage: Component = () => {
  const navigate = useNavigate();
  const params = useParams();
  const [showDeleteConfirm, setShowDeleteConfirm] = createSignal(false);
  const [deleting, setDeleting] = createSignal(false);

  createEffect(() => {
    if (params.id) {
      plantsStore.loadPlant(params.id);
    }
  });

  const handleDelete = async () => {
    try {
      setDeleting(true);
      await plantsStore.deletePlant(params.id);
      navigate('/plants');
    } catch (error) {
      console.error('Failed to delete plant:', error);
    } finally {
      setDeleting(false);
      setShowDeleteConfirm(false);
    }
  };

  return (
    <Show
      when={plantsStore.selectedPlant && !plantsStore.loading}
      fallback={
        <div class="flex justify-center py-12">
          <LoadingSpinner size="lg" />
        </div>
      }
    >
      <div class="space-y-6">{(() => {
        const plant = plantsStore.selectedPlant!;
        return (
      <>
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <div class="flex items-center space-x-4">
              <A
                href="/plants"
                class="text-gray-400 hover:text-gray-600"
                aria-label="Back to plants"
              >
                <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
                </svg>
              </A>
              <div>
                <h1 class="text-2xl font-bold text-gray-900">{plant.name}</h1>
                <p class="text-gray-600 italic">{plant.genus}</p>
              </div>
            </div>
          </div>
          
          <div class="flex items-center space-x-4">
            <A
              href={`/plants/${plant.id}/edit`}
              class="btn btn-outline btn-sm"
            >
              <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
              </svg>
              Edit
            </A>
            
            <Button
              variant="danger"
              size="sm"
              onClick={() => setShowDeleteConfirm(true)}
            >
              <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
              Delete
            </Button>
          </div>
        </div>

        <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div class="lg:col-span-2 space-y-6">
            <ActivityLog plant={plant} />
          </div>
          
          <div class="space-y-6">
            <div class="card">
              <div class="card-header">
                <h3 class="text-lg font-medium text-gray-900">Plant Info</h3>
              </div>
              <div class="card-body space-y-4">
                <div>
                  <label class="label">Watering Schedule</label>
                  <p class="text-sm text-gray-600">Every {plant.wateringIntervalDays} days</p>
                  {plant.lastWatered && (
                    <p class="text-xs text-gray-500">Last watered: {formatDate(plant.lastWatered)}</p>
                  )}
                </div>
                
                <div>
                  <label class="label">Fertilizing Schedule</label>
                  <p class="text-sm text-gray-600">Every {plant.fertilizingIntervalDays} days</p>
                  {plant.lastFertilized && (
                    <p class="text-xs text-gray-500">Last fertilized: {formatDate(plant.lastFertilized)}</p>
                  )}
                </div>
                
                <div>
                  <label class="label">Added</label>
                  <p class="text-sm text-gray-600">{formatDate(plant.createdAt)}</p>
                </div>
              </div>
            </div>
            
            <PhotoGallery plantId={plant.id} />
          </div>
        </div>

        {plantsStore.error && (
          <div class="bg-red-50 border border-red-200 rounded-md p-4">
            <p class="text-sm text-red-600">{plantsStore.error}</p>
          </div>
        )}

        {/* Delete Confirmation Modal */}
        {showDeleteConfirm() && (
          <div class="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center p-4 z-50">
            <div class="bg-white rounded-lg max-w-md w-full p-6">
              <h3 class="text-lg font-medium text-gray-900 mb-4">Delete Plant</h3>
              <p class="text-sm text-gray-600 mb-6">
                Are you sure you want to delete "{plant.name}"? This action cannot be undone.
              </p>
              <div class="flex justify-end space-x-4">
                <Button
                  variant="outline"
                  onClick={() => setShowDeleteConfirm(false)}
                  disabled={deleting()}
                >
                  Cancel
                </Button>
                <Button
                  variant="danger"
                  onClick={handleDelete}
                  loading={deleting()}
                >
                  Delete
                </Button>
              </div>
            </div>
          </div>
        )}
      </>
        );
      })()}</div>
    </Show>
  );
};