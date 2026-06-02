import { Component, createSignal, Show, For } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { Button } from '@/components/ui/Button';
import { TrackingEntryForm } from './TrackingEntryForm';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';

type CreateTrackingEntryRequest = components['schemas']['CreateTrackingEntryRequest'];
type CareTaskWithStatus = components['schemas']['CareTaskWithStatus'];

interface TrackingSectionProps {
  plant: Plant;
}

export const TrackingSection: Component<TrackingSectionProps> = (props) => {
  const [submitting, setSubmitting] = createSignal<string | null>(null);
  const [showDetailedForm, setShowDetailedForm] = createSignal(false);

  const careTasks = () => (props.plant.careTasks ?? []).filter(t => !t.archivedAt);

  const handleQuickCareTask = async (task: CareTaskWithStatus) => {
    try {
      setSubmitting(task.id);
      const payload: CreateTrackingEntryRequest = {
        careTaskIds: [task.id],
        timestamp: new Date().toISOString(),
      };
      await plantsStore.createTrackingEntry(props.plant.id, payload);
    } catch (error) {
      console.error('Failed to log care task:', error);
    } finally {
      setSubmitting(null);
    }
  };

  const openDetailedForm = () => {
    setShowDetailedForm(true);
  };

  const closeDetailedForm = () => {
    setShowDetailedForm(false);
  };

  const handleDetailedSubmit = async (data: CreateTrackingEntryRequest) => {
    await plantsStore.createTrackingEntry(props.plant.id, data);
  };

  return (
    <div class="card">
      <div class="card-header">
        <h3 class="text-lg font-medium text-gray-900">Track Activity</h3>
      </div>
      <div class="card-body space-y-4">
        <div class="space-y-4">
          {/* Quick Care Task Actions */}
          <Show when={careTasks().length > 0}>
            <div class="flex flex-wrap gap-2">
              <For each={careTasks()}>
                {(task) => (
                  <Button
                    variant="primary"
                    size="sm"
                    onClick={() => handleQuickCareTask(task)}
                    loading={submitting() === task.id}
                  >
                    <span class="mr-1">{task.icon ?? '🌱'}</span>
                    {task.name}
                  </Button>
                )}
              </For>
            </div>
          </Show>

          {/* Create Detailed Entry */}
          <div class="border-t border-gray-200 pt-4">
            <Button
              variant="outline"
              size="sm"
              onClick={openDetailedForm}
              class="w-full flex items-center justify-center"
            >
              <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 4v16m8-8H4" />
              </svg>
              Create Detailed Entry
            </Button>
          </div>
        </div>

        {/* Detailed Tracking Form Modal */}
        <Show when={showDetailedForm()}>
          <TrackingEntryForm
            plant={props.plant}
            onClose={closeDetailedForm}
            onSuccess={closeDetailedForm}
            onSubmit={handleDetailedSubmit}
          />
        </Show>
      </div>
    </div>
  );
};
