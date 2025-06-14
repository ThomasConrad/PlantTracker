import { Component, createSignal, Show, For } from 'solid-js';
import { plantsStore } from '@/stores/plants';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
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
    <div class="fixed top-0 sm:top-16 left-0 right-0 bottom-0 bg-black bg-opacity-50 flex items-end sm:items-center justify-center p-0 sm:p-4 z-50">
      <div class="bg-white w-full max-w-md rounded-t-2xl sm:rounded-lg shadow-xl transform transition-all duration-200 ease-out h-auto max-h-[90vh] overflow-y-auto">
        {/* Header */}
        <div class="sticky top-0 bg-white border-b border-gray-200 px-4 py-3 sm:px-6 sm:py-4 rounded-t-2xl sm:rounded-none">
          <div class="flex items-center justify-between">
            <h2 class="text-lg font-semibold text-gray-900">
              Care for {props.plant.name}
            </h2>
            <button
              onClick={props.onClose}
              class="text-gray-400 hover:text-gray-600 transition-colors"
            >
              <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>

        <form onSubmit={handleSubmit} class="p-4 sm:p-6 space-y-6">
          {/* Care Type Selection */}
          <div class="space-y-3">
            <label class="block text-sm font-medium text-gray-700">Care Type</label>
            <div class="grid grid-cols-2 gap-3">
              <button
                type="button"
                onClick={() => handleTypeChange('watering')}
                class={`p-3 rounded-lg border text-center transition-all ${
                  entryType() === 'watering'
                    ? 'bg-blue-50 border-blue-200 text-blue-700'
                    : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50'
                }`}
              >
                <div class="flex flex-col items-center space-y-1">
                  <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707" />
                  </svg>
                  <span class="text-sm font-medium">Watering</span>
                </div>
              </button>
              
              <button
                type="button"
                onClick={() => handleTypeChange('fertilizing')}
                class={`p-3 rounded-lg border text-center transition-all ${
                  entryType() === 'fertilizing'
                    ? 'bg-green-50 border-green-200 text-green-700'
                    : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50'
                }`}
              >
                <div class="flex flex-col items-center space-y-1">
                  <svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
                  </svg>
                  <span class="text-sm font-medium">Fertilizing</span>
                </div>
              </button>
            </div>
          </div>

          {/* Watering Specific Fields */}
          <Show when={entryType() === 'watering'}>
            <div class="space-y-4 p-4 bg-blue-50 rounded-lg">
              <h3 class="text-sm font-medium text-blue-900">Watering Details</h3>
              <div class="grid grid-cols-2 gap-3">
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
                  <label class="block text-sm font-medium text-gray-700">Unit</label>
                  <select
                    class="input w-full"
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
            <div class="space-y-4 p-4 bg-green-50 rounded-lg">
              <h3 class="text-sm font-medium text-green-900">Fertilizer Details</h3>
              
              <div class="grid grid-cols-1 gap-4">
                <div class="space-y-1">
                  <label class="block text-sm font-medium text-gray-700">Type</label>
                  <select
                    class="input w-full"
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

                <div class="grid grid-cols-2 gap-3">
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
                    <label class="block text-sm font-medium text-gray-700">Unit</label>
                    <select
                      class="input w-full"
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
            <label class="block text-sm font-medium text-gray-700">
              Notes (optional)
            </label>
            <textarea
              class="input w-full min-h-[80px] resize-none"
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
          <div class="flex space-x-3 pt-4">
            <Button
              type="button"
              variant="outline"
              onClick={props.onClose}
              class="flex-1"
            >
              Cancel
            </Button>
            <Button
              type="submit"
              variant="primary"
              loading={submitting()}
              class="flex-1"
            >
              {entryType() === 'watering' ? 'Record Watering' : 'Record Fertilizing'}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}; 