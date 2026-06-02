import { Component, createSignal, Show, For, onMount } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';
import { formatDate } from '@/utils/date';

type TrackingEntry = components['schemas']['TrackingEntry'];
type CreateTrackingEntryRequest = components['schemas']['CreateTrackingEntryRequest'];
type CareTaskWithStatus = components['schemas']['CareTaskWithStatus'];

interface ActivityLogProps {
  plant: Plant;
}

export const ActivityLog: Component<ActivityLogProps> = (props) => {
  const [entries, setEntries] = createSignal<TrackingEntry[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [showAddForm, setShowAddForm] = createSignal(false);
  const [selectedCareTaskIds, setSelectedCareTaskIds] = createSignal<string[]>([]);
  const [notes, setNotes] = createSignal('');
  const [selectedMetricId, setSelectedMetricId] = createSignal('');
  const [value, setValue] = createSignal('');
  const [submitting, setSubmitting] = createSignal(false);
  const [selectedPhotos, setSelectedPhotos] = createSignal<File[]>([]);
  const [uploadingPhotos, setUploadingPhotos] = createSignal(false);

  const careTasks = () => (props.plant.careTasks ?? []).filter(t => !t.archivedAt);

  const loadEntries = async () => {
    try {
      setLoading(true);
      const response = await plantsStore.getTrackingEntries(props.plant.id);
      setEntries(response.entries);
    } catch (error) {
      console.error('Failed to load tracking entries:', error);
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadEntries();
  });

  const handleQuickCareTask = async (task: CareTaskWithStatus) => {
    try {
      setSubmitting(true);
      const payload: CreateTrackingEntryRequest = {
        careTaskIds: [task.id],
        timestamp: new Date().toISOString(),
      };
      await plantsStore.createTrackingEntry(props.plant.id, payload);
      await loadEntries();
    } catch (error) {
      console.error('Failed to log care task:', error);
    } finally {
      setSubmitting(false);
    }
  };

  const toggleCareTask = (taskId: string) => {
    setSelectedCareTaskIds(prev => 
      prev.includes(taskId) ? prev.filter(id => id !== taskId) : [...prev, taskId]
    );
  };

  const handleAddEntry = async (e: Event) => {
    e.preventDefault();
    
    try {
      setSubmitting(true);
      
      // Upload photos first if any are selected
      let photoIds: string[] = [];
      if (selectedPhotos().length > 0) {
        setUploadingPhotos(true);
        for (const photo of selectedPhotos()) {
          try {
            const uploadedPhoto = await plantsStore.uploadPhoto(props.plant.id, photo);
            photoIds.push(uploadedPhoto.id);
          } catch (error) {
            console.error('Failed to upload photo:', error);
          }
        }
        setUploadingPhotos(false);
      }
      
      const entryData: CreateTrackingEntryRequest = {
        timestamp: new Date().toISOString(),
        careTaskIds: selectedCareTaskIds().length > 0 ? selectedCareTaskIds() : undefined,
        notes: notes() || undefined,
        photoIds: photoIds.length > 0 ? photoIds : undefined,
      };

      // Add measurement if selected
      if (selectedMetricId() && value()) {
        const metric = props.plant.customMetrics.find(m => m.id === selectedMetricId());
        if (metric) {
          let parsedValue: unknown;
          if (metric.dataType === 'Number') {
            parsedValue = parseFloat(value()) || 0;
          } else if (metric.dataType === 'Boolean') {
            parsedValue = value() === 'true';
          } else {
            parsedValue = value();
          }
          entryData.measurements = [{ metricId: selectedMetricId(), value: parsedValue }];
        }
      }

      await plantsStore.createTrackingEntry(props.plant.id, entryData);
      setShowAddForm(false);
      setValue('');
      setNotes('');
      setSelectedMetricId('');
      setSelectedCareTaskIds([]);
      setSelectedPhotos([]);
      await loadEntries();
    } catch (error) {
      console.error('Failed to create tracking entry:', error);
    } finally {
      setSubmitting(false);
      setUploadingPhotos(false);
    }
  };

  const handlePhotoSelect = (e: Event) => {
    const input = e.currentTarget as HTMLInputElement;
    const files = Array.from(input.files || []);
    setSelectedPhotos(files);
  };

  const removePhoto = (index: number) => {
    setSelectedPhotos(prev => prev.filter((_, i) => i !== index));
  };

  const getEntryLabel = (entry: TrackingEntry): string => {
    const parts: string[] = [];
    if (entry.careTaskIds && entry.careTaskIds.length > 0) {
      const taskNames = entry.careTaskIds
        .map(id => (props.plant.careTasks ?? []).find(t => t.id === id)?.name)
        .filter(Boolean);
      if (taskNames.length > 0) parts.push(taskNames.join(', '));
      else parts.push('Care');
    }
    if (entry.measurements && entry.measurements.length > 0) {
      parts.push('Measurement');
    }
    if (entry.photoIds && entry.photoIds.length > 0 && parts.length === 0) {
      parts.push('Photo');
    }
    if (parts.length === 0) parts.push('Note');
    return parts.join(' + ');
  };

  const getEntryIcon = (entry: TrackingEntry) => {
    if (entry.careTaskIds && entry.careTaskIds.length > 0) {
      // Show the icon of the first care task
      const task = (props.plant.careTasks ?? []).find(t => t.id === entry.careTaskIds![0]);
      if (task?.icon) return <span class="text-lg">{task.icon}</span>;
      return (
        <svg class="h-5 w-5 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      );
    }
    if (entry.measurements && entry.measurements.length > 0) {
      return (
        <svg class="h-5 w-5 text-purple-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
      );
    }
    if (entry.photoIds && entry.photoIds.length > 0) {
      return (
        <svg class="h-5 w-5 text-indigo-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
        </svg>
      );
    }
    return (
      <svg class="h-5 w-5 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
      </svg>
    );
  };

  return (
    <div class="card">
      <div class="card-header">
        <div class="flex justify-between items-center">
          <h3 class="text-lg font-medium text-gray-900">Activity Log</h3>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowAddForm(!showAddForm())}
          >
            <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 4v16m8-8H4" />
            </svg>
            Add Entry
          </Button>
        </div>
      </div>
      
      <div class="card-body">
        {/* Quick Care Task Actions */}
        <Show when={careTasks().length > 0}>
          <div class="flex flex-wrap gap-2 mb-6">
            <For each={careTasks()}>
              {(task) => (
                <Button
                  variant="primary"
                  size="sm"
                  onClick={() => handleQuickCareTask(task)}
                  loading={submitting()}
                >
                  <span class="mr-1">{task.icon ?? '🌱'}</span>
                  {task.name}
                </Button>
              )}
            </For>
          </div>
        </Show>

        {/* Add Entry Form */}
        <Show when={showAddForm()}>
          <form onSubmit={handleAddEntry} class="space-y-4 p-4 bg-gray-50 rounded-lg mb-6">
            {/* Care Tasks (multi-select) */}
            <Show when={careTasks().length > 0}>
              <div class="space-y-2">
                <label class="label">Care Tasks (optional, select multiple)</label>
                <div class="flex flex-wrap gap-2">
                  <For each={careTasks()}>
                    {(task) => (
                      <button
                        type="button"
                        onClick={() => toggleCareTask(task.id)}
                        class={`px-3 py-1.5 rounded-full text-sm border transition-colors ${
                          selectedCareTaskIds().includes(task.id)
                            ? 'bg-primary-100 border-primary-300 text-primary-800'
                            : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50'
                        }`}
                      >
                        <span class="mr-1">{task.icon ?? '🌱'}</span>
                        {task.name}
                      </button>
                    )}
                  </For>
                </div>
              </div>
            </Show>

            {/* Measurement */}
            <Show when={props.plant.customMetrics.length > 0}>
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div class="space-y-1">
                  <label class="label">Measurement (optional)</label>
                  <select
                    class="input"
                    value={selectedMetricId()}
                    onChange={(e) => setSelectedMetricId(e.currentTarget.value)}
                  >
                    <option value="">None</option>
                    <For each={props.plant.customMetrics}>
                      {(metric) => (
                        <option value={metric.id}>{metric.name} ({metric.unit})</option>
                      )}
                    </For>
                  </select>
                </div>
                <Show when={selectedMetricId()}>
                  {(() => {
                    const metric = props.plant.customMetrics.find(m => m.id === selectedMetricId());
                    if (!metric) return null;
                    if (metric.dataType === 'Boolean') {
                      return (
                        <div class="space-y-1">
                          <label class="label">Value</label>
                          <select class="input" value={value()} onChange={(e) => setValue(e.currentTarget.value)} required>
                            <option value="">Select...</option>
                            <option value="true">Yes</option>
                            <option value="false">No</option>
                          </select>
                        </div>
                      );
                    }
                    return (
                      <Input
                        label={`Value (${metric.unit})`}
                        type={metric.dataType === 'Number' ? 'number' : 'text'}
                        value={value()}
                        onInput={(e) => setValue(e.currentTarget.value)}
                        required
                      />
                    );
                  })()}
                </Show>
              </div>
            </Show>

            <Input
              label="Notes (optional)"
              type="text"
              value={notes()}
              onInput={(e) => setNotes(e.currentTarget.value)}
              placeholder="Any observations or details..."
            />

            {/* Photo Upload */}
            <div class="space-y-1">
              <label class="label">Photos (optional)</label>
              <input
                type="file"
                accept="image/*"
                multiple
                onChange={handlePhotoSelect}
                class="block w-full text-sm text-gray-500 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100"
              />
              <Show when={selectedPhotos().length > 0}>
                <div class="mt-2 grid grid-cols-4 gap-2">
                  <For each={selectedPhotos()}>
                    {(photo, index) => (
                      <div class="relative">
                        <img src={URL.createObjectURL(photo)} alt="Preview" class="w-full h-16 object-cover rounded border" />
                        <button
                          type="button"
                          onClick={() => removePhoto(index())}
                          class="absolute -top-1 -right-1 bg-red-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs hover:bg-red-600"
                        >×</button>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>

            <div class="flex justify-end space-x-3">
              <Button type="button" variant="outline" size="sm" onClick={() => setShowAddForm(false)}>Cancel</Button>
              <Button
                type="submit"
                size="sm"
                loading={submitting() || uploadingPhotos()}
                disabled={uploadingPhotos()}
              >
                {uploadingPhotos() ? 'Uploading...' : 'Add Entry'}
              </Button>
            </div>
          </form>
        </Show>

        {/* Activity List */}
        <Show when={loading()}>
          <div class="text-center py-8">
            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
          </div>
        </Show>

        <Show when={!loading() && entries().length === 0}>
          <div class="text-center py-8">
            <p class="text-sm text-gray-600">No activities recorded yet</p>
          </div>
        </Show>

        <Show when={!loading() && entries().length > 0}>
          <div class="space-y-3">
            <For each={entries()}>
              {(entry) => (
                <div class="flex items-start space-x-3 p-3 bg-white border border-gray-200 rounded-lg">
                  <div class="flex-shrink-0 mt-1">{getEntryIcon(entry)}</div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center justify-between">
                      <p class="text-sm font-medium text-gray-900">{getEntryLabel(entry)}</p>
                      <p class="text-xs text-gray-500">{formatDate(entry.timestamp)}</p>
                    </div>
                    <Show when={entry.measurements && entry.measurements.length > 0}>
                      <div class="mt-1 flex flex-wrap gap-1">
                        <For each={entry.measurements!}>
                          {(m) => {
                            const metric = props.plant.customMetrics.find(cm => cm.id === m.metricId);
                            return (
                              <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-purple-100 text-purple-800">
                                {metric?.name ?? 'Metric'}: {String(m.value)} {metric?.unit ?? ''}
                              </span>
                            );
                          }}
                        </For>
                      </div>
                    </Show>
                    <Show when={entry.notes}>
                      <p class="mt-1 text-sm text-gray-600">{entry.notes}</p>
                    </Show>
                    <Show when={entry.photoIds && entry.photoIds.length > 0}>
                      <div class="mt-2 grid grid-cols-4 gap-1">
                        <For each={entry.photoIds!}>
                          {(photoId) => (
                            <img
                              src={`/api/v1/plants/${props.plant.id}/photos/${photoId}`}
                              alt="Activity photo"
                              class="w-full h-12 object-cover rounded border cursor-pointer hover:opacity-80"
                              onClick={() => window.open(`/api/v1/plants/${props.plant.id}/photos/${photoId}`, '_blank')}
                            />
                          )}
                        </For>
                      </div>
                    </Show>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};
