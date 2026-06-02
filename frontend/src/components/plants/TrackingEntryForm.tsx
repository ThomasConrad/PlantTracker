import { Component, createSignal, Show, For } from "solid-js";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { PreviewUpload } from "./PreviewUpload";
import { plantsStore } from "@/stores/plants";
import type { Plant } from "@/types";
import type { components } from "@/types/api-generated";

type CreateTrackingEntryRequest =
  components["schemas"]["CreateTrackingEntryRequest"];

type FormMode = "care" | "note" | "photo" | "measurement";

interface TrackingEntryFormProps {
  plant: Plant;
  onClose: () => void;
  onSuccess: () => void;
  onSubmit: (data: CreateTrackingEntryRequest) => Promise<void>;
}

export const TrackingEntryForm: Component<TrackingEntryFormProps> = (props) => {
  const [mode, setMode] = createSignal<FormMode>("care");
  const [notes, setNotes] = createSignal("");
  const [submitting, setSubmitting] = createSignal(false);

  // Care task fields
  const [selectedCareTaskIds, setSelectedCareTaskIds] = createSignal<string[]>(
    [],
  );

  // Measurement fields
  const [selectedMetricId, setSelectedMetricId] = createSignal("");
  const [customValue, setCustomValue] = createSignal("");

  // Photo fields
  const [selectedPhoto, setSelectedPhoto] = createSignal<File | null>(null);
  const [uploadingPhoto, setUploadingPhoto] = createSignal(false);
  const [photoError, setPhotoError] = createSignal("");

  const careTasks = () =>
    (props.plant.careTasks ?? []).filter((t) => !t.archivedAt);

  const toggleCareTask = (taskId: string) => {
    setSelectedCareTaskIds((prev) =>
      prev.includes(taskId)
        ? prev.filter((id) => id !== taskId)
        : [...prev, taskId],
    );
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();

    const data: CreateTrackingEntryRequest = {
      timestamp: new Date().toISOString(),
      notes: notes() || undefined,
    };

    // Care tasks
    if (selectedCareTaskIds().length > 0) {
      data.careTaskIds = selectedCareTaskIds();
    }

    // Measurement
    if (mode() === "measurement" && selectedMetricId() && customValue()) {
      const metric = props.plant.customMetrics.find(
        (m) => m.id === selectedMetricId(),
      );
      if (metric) {
        let parsedValue: unknown;
        if (metric.dataType === "Number") {
          parsedValue = parseFloat(customValue()) || 0;
        } else if (metric.dataType === "Boolean") {
          parsedValue = customValue() === "true";
        } else {
          parsedValue = customValue();
        }
        data.measurements = [
          { metricId: selectedMetricId(), value: parsedValue },
        ];
      }
    }

    // Photo
    if (mode() === "photo") {
      if (!selectedPhoto()) {
        setPhotoError("Please select a photo");
        return;
      }
    }

    try {
      setSubmitting(true);

      if (mode() === "photo" && selectedPhoto()) {
        setUploadingPhoto(true);
        try {
          const photoResponse = await plantsStore.uploadPhoto(
            props.plant.id,
            selectedPhoto()!,
          );
          data.photoIds = [photoResponse.id];
        } catch (err) {
          console.error("Failed to upload photo:", err);
          setPhotoError("Failed to upload photo. Please try again.");
          return;
        } finally {
          setUploadingPhoto(false);
        }
      }

      await props.onSubmit(data);
      props.onSuccess();
      props.onClose();
    } catch (error) {
      console.error("Failed to create tracking entry:", error);
    } finally {
      setSubmitting(false);
    }
  };

  const resetForm = () => {
    setNotes("");
    setSelectedCareTaskIds([]);
    setSelectedMetricId("");
    setCustomValue("");
    setSelectedPhoto(null);
    setPhotoError("");
  };

  const handleClose = () => {
    resetForm();
    props.onClose();
  };

  const handleOverlayTouchMove = (e: TouchEvent) => {
    e.preventDefault();
  };

  const isSubmitDisabled = () => {
    if (submitting() || uploadingPhoto()) return true;
    if (mode() === "care" && selectedCareTaskIds().length === 0) return true;
    if (mode() === "measurement" && (!selectedMetricId() || !customValue()))
      return true;
    if (mode() === "photo" && !selectedPhoto()) return true;
    if (mode() === "note" && !notes()) return true;
    return false;
  };

  return (
    <div class="modal-overlay" onTouchMove={handleOverlayTouchMove}>
      <div class="modal-container" onTouchMove={(e) => e.stopPropagation()}>
        <div class="modal-header">
          <h2 class="modal-title">Create Tracking Entry</h2>
          <button
            type="button"
            onClick={handleClose}
            class="modal-close-button"
          >
            <svg
              class="h-5 w-5"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        <form onSubmit={handleSubmit} class="modal-body">
          <div class="space-y-6">
            {/* Mode Selection */}
            <div class="space-y-2">
              <label class="label">Activity Type</label>
              <div class="grid grid-cols-2 sm:grid-cols-3 gap-3">
                <Show when={careTasks().length > 0}>
                  <button
                    type="button"
                    onClick={() => setMode("care")}
                    class={`care-type-button ${mode() === "care" ? "care-type-button-active" : "care-type-button-inactive"}`}
                  >
                    <svg
                      class="h-5 w-5"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width={2}
                        d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                      />
                    </svg>
                    Care Task
                  </button>
                </Show>
                <button
                  type="button"
                  onClick={() => setMode("note")}
                  class={`care-type-button ${mode() === "note" ? "care-type-button-active" : "care-type-button-inactive"}`}
                >
                  <svg
                    class="h-5 w-5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width={2}
                      d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                    />
                  </svg>
                  Note
                </button>
                <button
                  type="button"
                  onClick={() => setMode("photo")}
                  class={`care-type-button ${mode() === "photo" ? "care-type-button-active" : "care-type-button-inactive"}`}
                >
                  <svg
                    class="h-5 w-5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width={2}
                      d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"
                    />
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width={2}
                      d="M15 13a3 3 0 11-6 0 3 3 0 016 0z"
                    />
                  </svg>
                  Photo
                </button>
                <Show when={props.plant.customMetrics.length > 0}>
                  <button
                    type="button"
                    onClick={() => setMode("measurement")}
                    class={`care-type-button ${mode() === "measurement" ? "care-type-button-active" : "care-type-button-inactive"}`}
                  >
                    <svg
                      class="h-5 w-5"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width={2}
                        d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                      />
                    </svg>
                    Measurement
                  </button>
                </Show>
              </div>
            </div>

            {/* Care Task Selection (multi-select) */}
            <Show when={mode() === "care"}>
              <div class="space-y-4">
                <h3 class="text-sm font-medium text-gray-900">
                  Select Care Tasks
                </h3>
                <div class="grid grid-cols-2 gap-2">
                  <For each={careTasks()}>
                    {(task) => (
                      <button
                        type="button"
                        onClick={() => toggleCareTask(task.id)}
                        class={`p-3 rounded-lg border text-left transition-colors ${
                          selectedCareTaskIds().includes(task.id)
                            ? "border-primary-500 bg-primary-50 ring-1 ring-primary-500"
                            : "border-gray-200 hover:border-gray-300"
                        }`}
                      >
                        <span class="text-lg">{task.icon ?? "🌱"}</span>
                        <p class="text-sm font-medium mt-1">{task.name}</p>
                      </button>
                    )}
                  </For>
                </div>
              </div>
            </Show>

            {/* Measurement Details */}
            <Show when={mode() === "measurement"}>
              <div class="space-y-4">
                <h3 class="text-sm font-medium text-gray-900">Measurement</h3>
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
                        <option value={metric.id}>
                          {metric.name} ({metric.unit})
                        </option>
                      )}
                    </For>
                  </select>
                </div>
                <Show when={selectedMetricId()}>
                  {(() => {
                    const metric = props.plant.customMetrics.find(
                      (m) => m.id === selectedMetricId(),
                    );
                    if (!metric) return null;

                    if (metric.dataType === "Boolean") {
                      return (
                        <div class="space-y-1">
                          <label class="label">Value</label>
                          <select
                            class="input"
                            value={customValue()}
                            onChange={(e) =>
                              setCustomValue(e.currentTarget.value)
                            }
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
                          type={
                            metric.dataType === "Number" ? "number" : "text"
                          }
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

            {/* Photo Details */}
            <Show when={mode() === "photo"}>
              <div class="space-y-4">
                <h3 class="text-sm font-medium text-gray-900">Photo</h3>
                <PreviewUpload
                  onFileSelect={(file) => {
                    setSelectedPhoto(file);
                    setPhotoError("");
                  }}
                  selectedFile={selectedPhoto()}
                  error={photoError()}
                  loading={uploadingPhoto()}
                />
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
            <Button type="button" variant="outline" onClick={handleClose}>
              Cancel
            </Button>
            <Button
              type="submit"
              loading={submitting() || uploadingPhoto()}
              disabled={isSubmitDisabled()}
            >
              {uploadingPhoto() ? "Uploading Photo..." : "Create Entry"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};
