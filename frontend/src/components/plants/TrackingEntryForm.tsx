import { Component, createSignal, Show, For } from 'solid-js';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';

type CreateTrackingEntryRequest = components['schemas']['CreateTrackingEntryRequest'];
type EntryType = components['schemas']['EntryType'];

interface TrackingEntryFormProps {
  plant: Plant;
  onClose: () => void;
  onSuccess: () => void;
  onSubmit: (data: CreateTrackingEntryRequest) => Promise<void>;
}

export const TrackingEntryForm: Component<TrackingEntryFormProps> = (props) => {
  const [entryType, setEntryType] = createSignal<EntryType>('watering');
  const [notes, setNotes] = createSignal('');
  const [submitting, setSubmitting] = createSignal(false);
  
  // Watering fields
  const [waterAmount, setWaterAmount] = createSignal('');
  const [waterUnit, setWaterUnit] = createSignal('ml');
  
  // Fertilizing fields
  const [fertilizerType, setFertilizerType] = createSignal('');
  const [fertilizerBrand, setFertilizerBrand] = createSignal('');
  const [fertilizerAmount, setFertilizerAmount] = createSignal('');
  const [fertilizerUnit, setFertilizerUnit] = createSignal('ml');
  const [dilutionRatio, setDilutionRatio] = createSignal('');
  
  // Custom metric fields
  const [selectedMetricId, setSelectedMetricId] = createSignal('');
  const [customValue, setCustomValue] = createSignal('');

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    
    const baseData: CreateTrackingEntryRequest = {
      entryType: entryType(),
      timestamp: new Date().toISOString(),
      notes: notes() || undefined,
    };

    // Add type-specific data
    if (entryType() === 'watering' && waterAmount()) {
      baseData.value = {
        amount: parseFloat(waterAmount()),
        unit: waterUnit(),
      };
    } else if (entryType() === 'fertilizing') {
      const fertValue: any = {};
      if (fertilizerType()) fertValue.type = fertilizerType();
      if (fertilizerBrand()) fertValue.brand = fertilizerBrand();
      if (fertilizerAmount()) {
        fertValue.amount = parseFloat(fertilizerAmount());
        fertValue.unit = fertilizerUnit();
      }
      if (dilutionRatio()) fertValue.dilution = dilutionRatio();
      if (Object.keys(fertValue).length > 0) {
        baseData.value = fertValue;
      }
    } else if (entryType() === 'customMetric') {
      baseData.metricId = selectedMetricId();
      const metric = props.plant.customMetrics.find(m => m.id === selectedMetricId());
      if (metric) {
        if (metric.dataType === 'Number') {
          baseData.value = parseFloat(customValue()) || 0;
        } else if (metric.dataType === 'Boolean') {
          baseData.value = customValue() === 'true';
        } else {
          baseData.value = customValue();
        }
      }
    }

    try {
      setSubmitting(true);
      await props.onSubmit(baseData);
      props.onSuccess();
      props.onClose();
    } catch (error) {
      console.error('Failed to create tracking entry:', error);
    } finally {
      setSubmitting(false);
    }
  };

  const resetForm = () => {
    setNotes('');
    setWaterAmount('');
    setWaterUnit('ml');
    setFertilizerType('');
    setFertilizerBrand('');
    setFertilizerAmount('');
    setFertilizerUnit('ml');
    setDilutionRatio('');
    setSelectedMetricId('');
    setCustomValue('');
  };

  const handleClose = () => {
    resetForm();
    props.onClose();
  };

  const handleOverlayTouchMove = (e: TouchEvent) => {
    e.preventDefault();
  };

  return (
    <div 
      class="modal-overlay"
      onTouchMove={handleOverlayTouchMove}
    >
      <div 
        class="modal-container"
        onTouchMove={(e) => e.stopPropagation()}
      >
        <div class="modal-header">
          <h2 class="modal-title">Create Tracking Entry</h2>
          <button
            type="button"
            onClick={handleClose}
            class="modal-close-button"
          >
            <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <form onSubmit={handleSubmit} class="modal-body">
          <div class="space-y-6">
            {/* Entry Type Selection */}
            <div class="space-y-2">
              <label class="label">Activity Type</label>
              <div class="grid grid-cols-2 gap-3">
                <button
                  type="button"
                  onClick={() => setEntryType('watering')}
                  class={`care-type-button ${entryType() === 'watering' ? 'care-type-button-active' : 'care-type-button-inactive'}`}
                >
                  <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
                  </svg>
                  Watering
                </button>
                <button
                  type="button"
                  onClick={() => setEntryType('fertilizing')}
                  class={`care-type-button ${entryType() === 'fertilizing' ? 'care-type-button-active' : 'care-type-button-inactive'}`}
                >
                  <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                  </svg>
                  Fertilizing
                </button>
                <button
                  type="button"
                  onClick={() => setEntryType('note')}
                  class={`care-type-button ${entryType() === 'note' ? 'care-type-button-active' : 'care-type-button-inactive'}`}
                >
                  <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                  Note
                </button>
                <Show when={props.plant.customMetrics.length > 0}>
                  <button
                    type="button"
                    onClick={() => setEntryType('customMetric')}
                    class={`care-type-button ${entryType() === 'customMetric' ? 'care-type-button-active' : 'care-type-button-inactive'}`}
                  >
                    <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v10" />
                    </svg>
                    Custom
                  </button>
                </Show>
              </div>
            </div>

            {/* Watering Details */}
            <Show when={entryType() === 'watering'}>
              <div class="space-y-4">
                <h3 class="text-sm font-medium text-gray-900">Watering Details</h3>
                <div class="grid grid-cols-2 gap-4">
                  <Input
                    label="Amount"
                    type="number"
                    value={waterAmount()}
                    onInput={(e) => setWaterAmount(e.currentTarget.value)}
                    placeholder="e.g., 250"
                    step="0.1"
                  />
                  <div class="space-y-1">
                    <label class="label">Unit</label>
                    <select
                      class="input"
                      value={waterUnit()}
                      onChange={(e) => setWaterUnit(e.currentTarget.value)}
                    >
                      <option value="ml">ml</option>
                      <option value="l">l</option>
                      <option value="cups">cups</option>
                      <option value="fl oz">fl oz</option>
                    </select>
                  </div>
                </div>
              </div>
            </Show>

            {/* Fertilizing Details */}
            <Show when={entryType() === 'fertilizing'}>
              <div class="space-y-4">
                <h3 class="text-sm font-medium text-gray-900">Fertilizing Details</h3>
                <div class="grid grid-cols-1 gap-4">
                  <Input
                    label="Fertilizer Type"
                    value={fertilizerType()}
                    onInput={(e) => setFertilizerType(e.currentTarget.value)}
                    placeholder="e.g., Liquid All-Purpose"
                  />
                  <Input
                    label="Brand (optional)"
                    value={fertilizerBrand()}
                    onInput={(e) => setFertilizerBrand(e.currentTarget.value)}
                    placeholder="e.g., Miracle-Gro"
                  />
                  <div class="grid grid-cols-2 gap-4">
                    <Input
                      label="Amount (optional)"
                      type="number"
                      value={fertilizerAmount()}
                      onInput={(e) => setFertilizerAmount(e.currentTarget.value)}
                      placeholder="e.g., 10"
                      step="0.1"
                    />
                    <div class="space-y-1">
                      <label class="label">Unit</label>
                      <select
                        class="input"
                        value={fertilizerUnit()}
                        onChange={(e) => setFertilizerUnit(e.currentTarget.value)}
                      >
                        <option value="ml">ml</option>
                        <option value="l">l</option>
                        <option value="tsp">tsp</option>
                        <option value="tbsp">tbsp</option>
                        <option value="g">g</option>
                      </select>
                    </div>
                  </div>
                  <Input
                    label="Dilution Ratio (optional)"
                    value={dilutionRatio()}
                    onInput={(e) => setDilutionRatio(e.currentTarget.value)}
                    placeholder="e.g., 1:10 or 1 tsp per 500ml"
                  />
                </div>
              </div>
            </Show>

            {/* Custom Metric Details */}
            <Show when={entryType() === 'customMetric'}>
              <div class="space-y-4">
                <h3 class="text-sm font-medium text-gray-900">Custom Metric</h3>
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
                <Show when={selectedMetricId()}>
                  {(() => {
                    const metric = props.plant.customMetrics.find(m => m.id === selectedMetricId());
                    if (!metric) return null;

                    if (metric.dataType === 'Boolean') {
                      return (
                        <div class="space-y-1">
                          <label class="label">Value</label>
                          <select
                            class="input"
                            value={customValue()}
                            onChange={(e) => setCustomValue(e.currentTarget.value)}
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
                          value={customValue()}
                          onInput={(e) => setCustomValue(e.currentTarget.value)}
                          required
                        />
                      );
                    }
                  })()}
                </Show>
              </div>
            </Show>

            {/* Notes */}
            <Input
              label="Notes (optional)"
              value={notes()}
              onInput={(e) => setNotes(e.currentTarget.value)}
              placeholder="Any additional observations or details..."
            />
          </div>

          <div class="modal-footer">
            <Button
              type="button"
              variant="outline"
              onClick={handleClose}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              loading={submitting()}
              disabled={
                (entryType() === 'customMetric' && (!selectedMetricId() || !customValue())) ||
                submitting()
              }
            >
              Create Entry
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};