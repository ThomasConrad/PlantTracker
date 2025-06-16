import { Component, createSignal, For } from 'solid-js';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { PreviewUpload } from '@/components/plants/PreviewUpload';
import type { PlantFormData, Photo } from '@/types';

interface PlantFormProps {
  initialData?: Partial<PlantFormData>;
  existingPreviewUrl?: string | null;
  existingPhotos?: Photo[];
  onSubmit: (data: PlantFormData & { previewFile?: File }) => Promise<void>;
  onPhotoSelect?: (photoId: string) => Promise<void>;
  onClearPreview?: () => Promise<void>;
  submitText?: string;
  loading?: boolean;
}

export const PlantForm: Component<PlantFormProps> = (props) => {
  const [formData, setFormData] = createSignal<PlantFormData>({
    name: props.initialData?.name || '',
    genus: props.initialData?.genus || '',
    wateringSchedule: props.initialData?.wateringSchedule || { intervalDays: 7 },
    fertilizingSchedule: props.initialData?.fertilizingSchedule || { intervalDays: 14 },
    customMetrics: props.initialData?.customMetrics || [],
  });

  const [errors, setErrors] = createSignal<Record<string, string>>({});
  const [previewFile, setThumbnailFile] = createSignal<File | null>(null);
  const [previewError, setThumbnailError] = createSignal<string>('');

  const addCustomMetric = () => {
    setFormData(prev => ({
      ...prev,
      customMetrics: [
        ...prev.customMetrics,
        { name: '', unit: '', dataType: 'Number' as const }
      ]
    }));
  };

  const removeCustomMetric = (index: number) => {
    setFormData(prev => ({
      ...prev,
      customMetrics: prev.customMetrics.filter((_, i) => i !== index)
    }));
  };

  const updateCustomMetric = (index: number, field: string, value: string) => {
    setFormData(prev => ({
      ...prev,
      customMetrics: prev.customMetrics.map((metric, i) =>
        i === index ? { ...metric, [field]: value } : metric
      )
    }));
  };


  const validateForm = () => {
    const newErrors: Record<string, string> = {};

    if (!formData().name.trim()) {
      newErrors.name = 'Plant name is required';
    }

    if (!formData().genus.trim()) {
      newErrors.genus = 'Genus is required';
    }

    // Validate watering schedule if enabled
    const wateringInterval = formData().wateringSchedule?.intervalDays;
    if (wateringInterval && wateringInterval < 1) {
      newErrors.wateringIntervalDays = 'Watering interval must be at least 1 day';
    }

    // Validate fertilizing schedule if enabled
    const fertilizingInterval = formData().fertilizingSchedule?.intervalDays;
    if (fertilizingInterval && fertilizingInterval < 1) {
      newErrors.fertilizingIntervalDays = 'Fertilizing interval must be at least 1 day';
    }

    formData().customMetrics.forEach((metric, index) => {
      if (metric.name && !metric.unit) {
        newErrors[`customMetric_${index}_unit`] = 'Unit is required';
      }
      if (metric.unit && !metric.name) {
        newErrors[`customMetric_${index}_name`] = 'Name is required';
      }
    });

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handlePreviewSelect = (file: File) => {
    setThumbnailFile(file);
    setThumbnailError('');
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    
    if (!validateForm()) return;

    const validCustomMetrics = formData().customMetrics.filter(
      metric => metric.name.trim() && metric.unit.trim()
    );

    await props.onSubmit({
      ...formData(),
      name: formData().name.trim(),
      genus: formData().genus.trim(),
      customMetrics: validCustomMetrics,
      previewFile: previewFile() || undefined,
    });
  };

  return (
    <form onSubmit={handleSubmit} class="space-y-6">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <Input
          label="Plant Name"
          type="text"
          value={formData().name}
          onInput={(e) => setFormData(prev => ({ ...prev, name: e.currentTarget.value }))}
          error={errors().name}
          placeholder="e.g., My Fiddle Leaf Fig"
          required
        />

        <Input
          label="Genus"
          type="text"
          value={formData().genus}
          onInput={(e) => setFormData(prev => ({ ...prev, genus: e.currentTarget.value }))}
          error={errors().genus}
          placeholder="e.g., Ficus lyrata"
          required
        />
      </div>

      {/* Watering Schedule */}
      <div class="space-y-4">
        <h3 class="text-lg font-medium text-gray-900">Watering Schedule</h3>
        <div class="space-y-4">
          <label class="flex items-center space-x-2">
            <input
              type="checkbox"
              class="rounded border-gray-300 text-green-600 focus:ring-green-500"
              checked={!!formData().wateringSchedule?.intervalDays}
              onInput={(e) => {
                const checked = e.currentTarget.checked;
                setFormData(prev => ({
                  ...prev,
                  wateringSchedule: checked 
                    ? { intervalDays: 7, amount: undefined, unit: 'ml', notes: '' }
                    : undefined
                }));
              }}
            />
            <span class="text-sm font-medium text-gray-700">Enable watering schedule</span>
          </label>
          
          {formData().wateringSchedule?.intervalDays && (
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 ml-6">
              <Input
                label="Interval (days)"
                type="number"
                min="1"
                max="365"
value={formData().wateringSchedule?.intervalDays || 7}
                onInput={(e) => setFormData(prev => ({
                  ...prev,
                  wateringSchedule: {
                    ...prev.wateringSchedule!,
                    intervalDays: parseInt(e.currentTarget.value) || 1
                  }
                }))}
                error={errors().wateringIntervalDays}
                required
              />
              
              <Input
                label="Amount (optional)"
                type="number"
                min="0.1"
                step="0.1"
value={formData().wateringSchedule?.amount || ''}
                onInput={(e) => setFormData(prev => ({
                  ...prev,
                  wateringSchedule: {
                    ...prev.wateringSchedule!,
                    amount: parseFloat(e.currentTarget.value) || undefined
                  }
                }))}
                placeholder="e.g., 250"
              />
              
              <Input
                label="Unit (optional)"
                type="text"
value={formData().wateringSchedule?.unit || ''}
                onInput={(e) => setFormData(prev => ({
                  ...prev,
                  wateringSchedule: {
                    ...prev.wateringSchedule!,
                    unit: e.currentTarget.value || undefined
                  }
                }))}
                placeholder="e.g., ml, cups"
              />
            </div>
          )}
          
          {formData().wateringSchedule?.intervalDays && (
            <div class="ml-6">
              <Input
                label="Notes (optional)"
                type="text"
value={formData().wateringSchedule?.notes || ''}
                onInput={(e) => setFormData(prev => ({
                  ...prev,
                  wateringSchedule: {
                    ...prev.wateringSchedule!,
                    notes: e.currentTarget.value || undefined
                  }
                }))}
                placeholder="e.g., Water when soil is dry"
              />
            </div>
          )}
        </div>
      </div>

      {/* Fertilizing Schedule */}
      <div class="space-y-4">
        <h3 class="text-lg font-medium text-gray-900">Fertilizing Schedule</h3>
        <div class="space-y-4">
          <label class="flex items-center space-x-2">
            <input
              type="checkbox"
              class="rounded border-gray-300 text-green-600 focus:ring-green-500"
              checked={!!formData().fertilizingSchedule?.intervalDays}
              onInput={(e) => {
                const checked = e.currentTarget.checked;
                setFormData(prev => ({
                  ...prev,
                  fertilizingSchedule: checked 
                    ? { intervalDays: 14, amount: undefined, unit: 'ml', notes: '' }
                    : undefined
                }));
              }}
            />
            <span class="text-sm font-medium text-gray-700">Enable fertilizing schedule</span>
          </label>
          
          {formData().fertilizingSchedule?.intervalDays && (
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 ml-6">
              <Input
                label="Interval (days)"
                type="number"
                min="1"
                max="365"
value={formData().fertilizingSchedule?.intervalDays || 14}
                onInput={(e) => setFormData(prev => ({
                  ...prev,
                  fertilizingSchedule: {
                    ...prev.fertilizingSchedule!,
                    intervalDays: parseInt(e.currentTarget.value) || 1
                  }
                }))}
                error={errors().fertilizingIntervalDays}
                required
              />
              
              <Input
                label="Amount (optional)"
                type="number"
                min="0.1"
                step="0.1"
value={formData().fertilizingSchedule?.amount || ''}
                onInput={(e) => setFormData(prev => ({
                  ...prev,
                  fertilizingSchedule: {
                    ...prev.fertilizingSchedule!,
                    amount: parseFloat(e.currentTarget.value) || undefined
                  }
                }))}
                placeholder="e.g., 5"
              />
              
              <Input
                label="Unit (optional)"
                type="text"
value={formData().fertilizingSchedule?.unit || ''}
                onInput={(e) => setFormData(prev => ({
                  ...prev,
                  fertilizingSchedule: {
                    ...prev.fertilizingSchedule!,
                    unit: e.currentTarget.value || undefined
                  }
                }))}
                placeholder="e.g., ml, drops"
              />
            </div>
          )}
          
          {formData().fertilizingSchedule?.intervalDays && (
            <div class="ml-6">
              <Input
                label="Notes (optional)"
                type="text"
value={formData().fertilizingSchedule?.notes || ''}
                onInput={(e) => setFormData(prev => ({
                  ...prev,
                  fertilizingSchedule: {
                    ...prev.fertilizingSchedule!,
                    notes: e.currentTarget.value || undefined
                  }
                }))}
                placeholder="e.g., Dilute 1:10 with water"
              />
            </div>
          )}
        </div>
      </div>

      <PreviewUpload
        onFileSelect={handlePreviewSelect}
        onPhotoSelect={props.onPhotoSelect}
        onClearPreview={props.onClearPreview}
        selectedFile={previewFile()}
        existingPreviewUrl={props.existingPreviewUrl}
        existingPhotos={props.existingPhotos}
        error={previewError()}
        loading={props.loading}
      />

      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <h3 class="text-lg font-medium text-gray-900">Custom Metrics</h3>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={addCustomMetric}
          >
            Add Metric
          </Button>
        </div>

        <div class="space-y-4">
          <For each={formData().customMetrics}>
            {(metric, index) => (
              <div class="bg-gray-50 p-4 rounded-lg space-y-4">
                <div class="flex items-center justify-between">
                  <h4 class="text-sm font-medium text-gray-700">Metric {index() + 1}</h4>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => removeCustomMetric(index())}
                  >
                    Remove
                  </Button>
                </div>
                
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <Input
                    label="Name"
                    type="text"
                    value={metric.name}
                    onInput={(e) => updateCustomMetric(index(), 'name', e.currentTarget.value)}
                    error={errors()[`customMetric_${index()}_name`]}
                    placeholder="e.g., Height"
                  />

                  <Input
                    label="Unit"
                    type="text"
                    value={metric.unit}
                    onInput={(e) => updateCustomMetric(index(), 'unit', e.currentTarget.value)}
                    error={errors()[`customMetric_${index()}_unit`]}
                    placeholder="e.g., cm"
                  />

                  <div class="space-y-1">
                    <label class="label">Data Type</label>
                    <select
                      class="input"
                      value={metric.dataType}
                      onChange={(e) => updateCustomMetric(index(), 'dataType', e.currentTarget.value)}
                    >
                      <option value="Number">Number</option>
                      <option value="Text">Text</option>
                      <option value="Boolean">Yes/No</option>
                    </select>
                  </div>
                </div>
              </div>
            )}
          </For>
        </div>
      </div>

      <div class="flex justify-end space-x-4">
        <Button
          type="submit"
          loading={props.loading}
          disabled={!formData().name.trim() || !formData().genus.trim()}
        >
          {props.submitText || 'Save Plant'}
        </Button>
      </div>
    </form>
  );
};