import { Component, createSignal, Show, For, onMount } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';
import { formatDate } from '@/utils/date';

type TrackingEntry = components['schemas']['TrackingEntry'];
type CreateTrackingEntryRequest = components['schemas']['CreateTrackingEntryRequest'];
type EntryType = components['schemas']['EntryType'];

interface ActivityLogProps {
  plant: Plant;
}

export const ActivityLog: Component<ActivityLogProps> = (props) => {
  const [entries, setEntries] = createSignal<TrackingEntry[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [showAddForm, setShowAddForm] = createSignal(false);
  const [entryType, setEntryType] = createSignal<EntryType>('note');
  const [notes, setNotes] = createSignal('');
  const [selectedMetricId, setSelectedMetricId] = createSignal('');
  const [value, setValue] = createSignal('');
  const [submitting, setSubmitting] = createSignal(false);
  const [selectedPhotos, setSelectedPhotos] = createSignal<File[]>([]);
  const [uploadingPhotos, setUploadingPhotos] = createSignal(false);

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

  const handleQuickAction = async (type: EntryType) => {
    try {
      setSubmitting(true);
      const payload: CreateTrackingEntryRequest = {
        entryType: type,
        timestamp: new Date().toISOString(),
      };
      await plantsStore.createTrackingEntry(props.plant.id, payload);
      await loadEntries(); // Refresh entries
    } catch (error) {
      console.error('Failed to create tracking entry:', error);
    } finally {
      setSubmitting(false);
    }
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
        entryType: entryType(),
        timestamp: new Date().toISOString(),
        notes: notes() || undefined,
        photoIds: photoIds.length > 0 ? photoIds : undefined,
      };

      if (entryType() === 'customMetric') {
        entryData.metricId = selectedMetricId();
        const metric = props.plant.customMetrics.find(m => m.id === selectedMetricId());
        if (metric) {
          if (metric.dataType === 'Number') {
            entryData.value = parseFloat(value()) || 0;
          } else if (metric.dataType === 'Boolean') {
            entryData.value = value() === 'true';
          } else {
            entryData.value = value();
          }
        }
      }

      await plantsStore.createTrackingEntry(props.plant.id, entryData);
      setShowAddForm(false);
      setValue('');
      setNotes('');
      setSelectedMetricId('');
      setSelectedPhotos([]);
      await loadEntries(); // Refresh entries
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

  const getEntryIcon = (type: EntryType) => {
    switch (type) {
      case 'watering':
        return (
          <svg class="h-5 w-5 text-blue-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
          </svg>
        );
      case 'fertilizing':
        return (
          <svg class="h-5 w-5 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
          </svg>
        );
      case 'customMetric':
        return (
          <svg class="h-5 w-5 text-purple-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v10" />
          </svg>
        );
      case 'note':
        return (
          <svg class="h-5 w-5 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
          </svg>
        );
      default:
        return null;
    }
  };

  const getEntryTypeLabel = (type: EntryType) => {
    switch (type) {
      case 'watering': return 'Watered';
      case 'fertilizing': return 'Fertilized';
      case 'customMetric': return 'Measurement';
      case 'note': return 'Note';
      default: return type;
    }
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
        {/* Quick Actions */}
        <div class="flex flex-wrap gap-3 mb-6">
          <Button
            variant="primary"
            size="sm"
            onClick={() => handleQuickAction('watering')}
            loading={submitting()}
          >
            <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
            </svg>
            Water Now
          </Button>

          <Button
            variant="primary"
            size="sm"
            onClick={() => handleQuickAction('fertilizing')}
            loading={submitting()}
          >
            <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
            </svg>
            Fertilize Now
          </Button>
        </div>

        {/* Add Entry Form */}
        <Show when={showAddForm()}>
          <form onSubmit={handleAddEntry} class="space-y-4 p-4 bg-gray-50 rounded-lg mb-6">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-1">
                <label class="label">Type</label>
                <select
                  class="input"
                  value={entryType()}
                  onChange={(e) => setEntryType(e.currentTarget.value as EntryType)}
                >
                  <option value="note">Note</option>
                  <option value="watering">Watering</option>
                  <option value="fertilizing">Fertilizing</option>
                  {props.plant.customMetrics.length > 0 && (
                    <option value="customMetric">Custom Metric</option>
                  )}
                </select>
              </div>

              <Show when={entryType() === 'customMetric'}>
                <div class="space-y-1">
                  <label class="label">Metric</label>
                  <select
                    class="input"
                    value={selectedMetricId()}
                    onChange={(e) => setSelectedMetricId(e.currentTarget.value)}
                    required
                  >
                    <option value="">Select metric...</option>
                    <For each={props.plant.customMetrics}>
                      {(metric) => (
                        <option value={metric.id}>{metric.name} ({metric.unit})</option>
                      )}
                    </For>
                  </select>
                </div>
              </Show>
            </div>

            <Show when={entryType() === 'customMetric' && selectedMetricId()}>
              {(() => {
                const metric = props.plant.customMetrics.find(m => m.id === selectedMetricId());
                if (!metric) return null;

                if (metric.dataType === 'Boolean') {
                  return (
                    <div class="space-y-1">
                      <label class="label">Value</label>
                      <select
                        class="input"
                        value={value()}
                        onChange={(e) => setValue(e.currentTarget.value)}
                        required
                      >
                        <option value="">Select...</option>
                        <option value="true">Yes</option>
                        <option value="false">No</option>
                      </select>
                    </div>
                  );
                } else {
                  return (
                    <Input
                      label={`Value (${metric.unit})`}
                      type={metric.dataType === 'Number' ? 'number' : 'text'}
                      value={value()}
                      onInput={(e) => setValue(e.currentTarget.value)}
                      required
                    />
                  );
                }
              })()}
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
                <div class="mt-2 grid grid-cols-2 gap-2">
                  <For each={selectedPhotos()}>
                    {(photo, index) => (
                      <div class="relative">
                        <img
                          src={URL.createObjectURL(photo)}
                          alt="Preview"
                          class="w-full h-20 object-cover rounded border"
                        />
                        <button
                          type="button"
                          onClick={() => removePhoto(index())}
                          class="absolute -top-1 -right-1 bg-red-500 text-white rounded-full w-5 h-5 flex items-center justify-center text-xs hover:bg-red-600"
                        >
                          ×
                        </button>
                      </div>
                    )}
                  </For>
                </div>
              </Show>
            </div>

            <div class="flex justify-end space-x-3">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setShowAddForm(false)}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                size="sm"
                loading={submitting() || uploadingPhotos()}
                disabled={
                  (entryType() === 'customMetric' && 
                  (!selectedMetricId() || !value())) ||
                  uploadingPhotos()
                }
              >
                {uploadingPhotos() ? 'Uploading Photos...' : 'Add Entry'}
              </Button>
            </div>
          </form>
        </Show>

        {/* Activity List */}
        <Show when={loading()}>
          <div class="text-center py-8">
            <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
            <p class="mt-2 text-sm text-gray-600">Loading activities...</p>
          </div>
        </Show>

        <Show when={!loading() && entries().length === 0}>
          <div class="text-center py-8">
            <svg class="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p class="mt-2 text-sm text-gray-600">No activities recorded yet</p>
            <p class="text-xs text-gray-500">Track watering, fertilizing, or add custom notes</p>
          </div>
        </Show>

        <Show when={!loading() && entries().length > 0}>
          <div class="space-y-4">
            <For each={entries()}>
              {(entry) => (
                <div class="flex items-start space-x-3 p-3 bg-white border border-gray-200 rounded-lg">
                  <div class="flex-shrink-0 mt-1">
                    {getEntryIcon(entry.entryType)}
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center justify-between">
                      <p class="text-sm font-medium text-gray-900">
                        {getEntryTypeLabel(entry.entryType)}
                        <Show when={entry.entryType === 'customMetric' && entry.value !== null}>
                          <span class="ml-2 text-gray-600">
                            {typeof entry.value === 'string' ? entry.value : JSON.stringify(entry.value)}
                          </span>
                        </Show>
                      </p>
                      <p class="text-xs text-gray-500">
                        {formatDate(entry.timestamp)}
                      </p>
                    </div>
                    <Show when={entry.notes}>
                      <p class="mt-1 text-sm text-gray-600">{entry.notes}</p>
                    </Show>
                    <Show when={entry.photoIds && Array.isArray(entry.photoIds) && entry.photoIds.length > 0}>
                      <div class="mt-2 grid grid-cols-2 gap-2">
                        <For each={entry.photoIds as string[]}>
                          {(photoId) => (
                            <img
                              src={`/api/v1/plants/${props.plant.id}/photos/${photoId}`}
                              alt="Activity photo"
                              class="w-full h-20 object-cover rounded border cursor-pointer hover:opacity-80"
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