import { Component, createSignal, Show, For } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { EnhancedCareForm } from './EnhancedCareForm';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';

type CreateTrackingEntryRequest = components['schemas']['CreateTrackingEntryRequest'];
type EntryType = components['schemas']['EntryType'];

interface TrackingSectionProps {
  plant: Plant;
}

export const TrackingSection: Component<TrackingSectionProps> = (props) => {
  const [showTrackingForm, setShowTrackingForm] = createSignal(false);
  const [trackingType, setTrackingType] = createSignal<EntryType>('watering');
  const [selectedMetricId, setSelectedMetricId] = createSignal('');
  const [value, setValue] = createSignal('');
  const [notes, setNotes] = createSignal('');
  const [submitting, setSubmitting] = createSignal(false);
  const [showEnhancedForm, setShowEnhancedForm] = createSignal(false);
  const [enhancedFormType, setEnhancedFormType] = createSignal<EntryType>('watering');

  const handleQuickAction = async (type: EntryType) => {
    try {
      setSubmitting(true);
      const payload: CreateTrackingEntryRequest = {
        entryType: type,
        timestamp: new Date().toISOString(),
      };
      await plantsStore.createTrackingEntry(props.plant.id, payload);
    } catch (error) {
      console.error('Failed to create tracking entry:', error);
    } finally {
      setSubmitting(false);
    }
  };

  const handleTrackingSubmit = async (e: Event) => {
    e.preventDefault();
    
    const entryData: CreateTrackingEntryRequest = {
      entryType: trackingType(),
      timestamp: new Date().toISOString(),
      notes: notes() || undefined,
    };

    if (trackingType() === 'customMetric') {
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

    try {
      setSubmitting(true);
      await plantsStore.createTrackingEntry(props.plant.id, entryData);
      setShowTrackingForm(false);
      setValue('');
      setNotes('');
      setSelectedMetricId('');
    } catch (error) {
      console.error('Failed to create tracking entry:', error);
    } finally {
      setSubmitting(false);
    }
  };

  const openEnhancedForm = (type: EntryType) => {
    setEnhancedFormType(type);
    setShowEnhancedForm(true);
  };

  const closeEnhancedForm = () => {
    setShowEnhancedForm(false);
  };

  return (
    <div class="card">
      <div class="card-header">
        <h3 class="text-lg font-medium text-gray-900">Track Activity</h3>
      </div>
      <div class="card-body space-y-4">
        <div class="space-y-4">
          {/* Quick Actions */}
          <div class="flex flex-wrap gap-3">
            <Button
              variant="primary"
              size="sm"
              onClick={() => handleQuickAction('watering')}
              loading={submitting()}
            >
              <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707"
                />
              </svg>
              Quick Water
            </Button>

            <Button
              variant="primary"
              size="sm"
              onClick={() => handleQuickAction('fertilizing')}
              loading={submitting()}
            >
              <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z"
                />
              </svg>
              Quick Fertilize
            </Button>
          </div>

          {/* Enhanced Actions */}
          <div class="border-t border-gray-200 pt-4">
            <h4 class="text-sm font-medium text-gray-700 mb-3">Add with Details</h4>
            <div class="flex flex-wrap gap-3">
              <Button
                variant="outline"
                size="sm"
                onClick={() => openEnhancedForm('watering')}
              >
                <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
                </svg>
                Water with Amount
              </Button>

              <Button
                variant="outline"
                size="sm"
                onClick={() => openEnhancedForm('fertilizing')}
              >
                <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                </svg>
                Fertilize with Type
              </Button>

              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowTrackingForm(!showTrackingForm())}
              >
                <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 4v16m8-8H4" />
                </svg>
                Custom Entry
              </Button>
            </div>
          </div>
        </div>

        <Show when={showTrackingForm()}>
          <form onSubmit={handleTrackingSubmit} class="space-y-4 p-4 bg-gray-50 rounded-lg">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-1">
                <label class="label">Type</label>
                <select
                  class="input"
                  value={trackingType()}
                  onChange={(e) => setTrackingType(e.currentTarget.value as EntryType)}
                >
                  <option value="watering">Watering</option>
                  <option value="fertilizing">Fertilizing</option>
                  <option value="note">Note</option>
                  {props.plant.customMetrics.length > 0 && (
                    <option value="customMetric">Custom Metric</option>
                  )}
                </select>
              </div>

              <Show when={trackingType() === 'customMetric'}>
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

            <Show when={trackingType() === 'customMetric' && selectedMetricId()}>
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
              placeholder="Any additional observations..."
            />

            <div class="flex justify-end space-x-3">
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setShowTrackingForm(false)}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                size="sm"
                loading={submitting()}
                disabled={
                  trackingType() === 'customMetric' && 
                  (!selectedMetricId() || !value())
                }
              >
                Add Entry
              </Button>
            </div>
          </form>
        </Show>

        {/* Enhanced Care Form Modal */}
        <Show when={showEnhancedForm()}>
          <EnhancedCareForm
            plant={props.plant}
            initialType={enhancedFormType()}
            onClose={closeEnhancedForm}
          />
        </Show>
      </div>
    </div>
  );
};