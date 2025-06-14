import { Component, createSignal, Show, For } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { CareTypeButton } from '@/components/ui/CareTypeButton';
import type { Plant } from '@/types';
import type { components } from '@/types/api-generated';

type CreateTrackingEntryRequest = components['schemas']['CreateTrackingEntryRequest'];
type EntryType = components['schemas']['EntryType'];

interface EnhancedCareFormProps {
  plant: Plant;
  onClose: () => void;
  initialType?: EntryType;
}

export const EnhancedCareForm: Component<EnhancedCareFormProps> = (props) => {
  const [entryType, setEntryType] = createSignal<EntryType>(props.initialType || 'watering');
  const [notes, setNotes] = createSignal('');
  const [submitting, setSubmitting] = createSignal(false);
  
  // Watering specific fields
  const [waterAmount, setWaterAmount] = createSignal('');
  const [waterUnit, setWaterUnit] = createSignal('ml');
  
  // Fertilizing specific fields
  const [fertilizerType, setFertilizerType] = createSignal('');
  const [fertilizerAmount, setFertilizerAmount] = createSignal('');
  const [fertilizerUnit, setFertilizerUnit] = createSignal('ml');
  const [fertilizerBrand, setFertilizerBrand] = createSignal('');
  const [dilutionRatio, setDilutionRatio] = createSignal('');

  const waterUnits = ['ml', 'L', 'cup', 'tbsp', 'tsp', 'oz', 'gallon'];
  const fertilizerTypes = ['liquid', 'granular', 'powder', 'tablet', 'stick', 'organic'];
  const fertilizerUnits = ['ml', 'L', 'tbsp', 'tsp', 'g', 'oz', 'drops'];

  const resetForm = () => {
    setNotes('');
    setWaterAmount('');
    setWaterUnit('ml');
    setFertilizerType('');
    setFertilizerAmount('');
    setFertilizerUnit('ml');
    setFertilizerBrand('');
    setDilutionRatio('');
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    
    try {
      setSubmitting(true);
      
      let value: any = null;
      let finalNotes = notes();

      if (entryType() === 'watering' && waterAmount()) {
        value = {
          amount: parseFloat(waterAmount()),
          unit: waterUnit()
        };
      } else if (entryType() === 'fertilizing') {
        const fertilizerData: any = {};
        
        if (fertilizerType()) fertilizerData.type = fertilizerType();
        if (fertilizerAmount()) {
          fertilizerData.amount = parseFloat(fertilizerAmount());
          fertilizerData.unit = fertilizerUnit();
        }
        if (dilutionRatio()) fertilizerData.dilution = dilutionRatio();
        
        if (Object.keys(fertilizerData).length > 0) {
          value = fertilizerData;
        }
        
        // Add fertilizer brand to notes if specified
        if (fertilizerBrand()) {
          const brandNote = `Brand: ${fertilizerBrand()}`;
          finalNotes = finalNotes ? `${brandNote}. ${finalNotes}` : brandNote;
        }
      }

      const payload: CreateTrackingEntryRequest = {
        entryType: entryType(),
        timestamp: new Date().toISOString(),
        value: value,
        notes: finalNotes || undefined,
      };

      await plantsStore.createTrackingEntry(props.plant.id, payload);
      resetForm();
      props.onClose();
    } catch (error) {
      console.error('Failed to create tracking entry:', error);
    } finally {
      setSubmitting(false);
    }
  };

  const handleTypeChange = (newType: EntryType) => {
    setEntryType(newType);
    resetForm();
  };



  return (
    <div class="care-form-overlay">
      <div class="care-form-container">
        {/* Header */}
        <div class="care-form-header">
          <div class="flex items-center justify-between">
            <h2 class="care-form-title">
              Care for {props.plant.name}
            </h2>
            <button
              onClick={props.onClose}
              class="care-form-close-button"
            >
              <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <form onSubmit={handleSubmit} class="care-form-body">
          {/* Care Type Selection */}
          <div class="space-y-3">
            <label class="label">Care Type</label>
            <div class="care-type-grid">
              <CareTypeButton
                type="watering"
                isActive={entryType() === 'watering'}
                onClick={() => handleTypeChange('watering')}
              >
                <svg class="care-type-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
                </svg>
                <span class="care-type-label">Watering</span>
              </CareTypeButton>
              
              <CareTypeButton
                type="fertilizing"
                isActive={entryType() === 'fertilizing'}
                onClick={() => handleTypeChange('fertilizing')}
              >
                <svg class="care-type-icon" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                </svg>
                <span class="care-type-label">Fertilizing</span>
              </CareTypeButton>
            </div>
          </div>

          {/* Watering Specific Fields */}
          <Show when={entryType() === 'watering'}>
            <div class="care-details-watering">
              <h3 class="care-details-title-watering">Watering Details</h3>
              <div class="care-form-grid-2">
                <Input
                  label="Amount (optional)"
                  type="number"
                  step="0.1"
                  min="0"
                  value={waterAmount()}
                  onInput={(e) => setWaterAmount(e.currentTarget.value)}
                  placeholder="250"
                />
                <div class="space-y-1">
                  <label class="label">Unit</label>
                  <select
                    class="care-form-select"
                    value={waterUnit()}
                    onChange={(e) => setWaterUnit(e.currentTarget.value)}
                  >
                    <For each={waterUnits}>
                      {(unit) => <option value={unit}>{unit}</option>}
                    </For>
                  </select>
                </div>
              </div>
            </div>
          </Show>

          {/* Fertilizing Specific Fields */}
          <Show when={entryType() === 'fertilizing'}>
            <div class="care-details-fertilizing">
              <h3 class="care-details-title-fertilizing">Fertilizer Details</h3>
              
              <div class="care-form-grid-1">
                <div class="space-y-1">
                  <label class="label">Type</label>
                  <select
                    class="care-form-select"
                    value={fertilizerType()}
                    onChange={(e) => setFertilizerType(e.currentTarget.value)}
                  >
                    <option value="">Select type...</option>
                    <For each={fertilizerTypes}>
                      {(type) => <option value={type}>{type.charAt(0).toUpperCase() + type.slice(1)}</option>}
                    </For>
                  </select>
                </div>

                <Input
                  label="Brand (optional)"
                  type="text"
                  value={fertilizerBrand()}
                  onInput={(e) => setFertilizerBrand(e.currentTarget.value)}
                  placeholder="e.g., Miracle-Gro, Osmocote"
                />

                <div class="care-form-grid-2">
                  <Input
                    label="Amount (optional)"
                    type="number"
                    step="0.1"
                    min="0"
                    value={fertilizerAmount()}
                    onInput={(e) => setFertilizerAmount(e.currentTarget.value)}
                    placeholder="10"
                  />
                  <div class="space-y-1">
                    <label class="label">Unit</label>
                    <select
                      class="care-form-select"
                      value={fertilizerUnit()}
                      onChange={(e) => setFertilizerUnit(e.currentTarget.value)}
                    >
                      <For each={fertilizerUnits}>
                        {(unit) => <option value={unit}>{unit}</option>}
                      </For>
                    </select>
                  </div>
                </div>

                <Input
                  label="Dilution Ratio (optional)"
                  type="text"
                  value={dilutionRatio()}
                  onInput={(e) => setDilutionRatio(e.currentTarget.value)}
                  placeholder="e.g., 1:10, 1:20"
                />
              </div>
            </div>
          </Show>

          {/* Notes */}
          <div class="space-y-1">
            <label class="label">
              Notes (optional)
            </label>
            <textarea
              class="care-form-textarea"
              value={notes()}
              onInput={(e) => setNotes(e.currentTarget.value)}
              placeholder={
                entryType() === 'watering' 
                  ? "e.g., Soil was very dry, morning watering"
                  : "e.g., Weekly feeding, plant showing new growth"
              }
              rows={3}
            />
          </div>

          {/* Action Buttons */}
          <div class="care-form-actions">
            <Button
              type="button"
              variant="outline"
              onClick={props.onClose}
              class="care-form-button"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              loading={submitting()}
              class="care-form-button"
            >
              {entryType() === 'watering' ? 'Record Watering' : 'Record Fertilizing'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}; 