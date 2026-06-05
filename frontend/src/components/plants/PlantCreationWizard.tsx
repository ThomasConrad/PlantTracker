import {
  Component,
  createSignal,
  Show,
  For,
  Switch,
  Match,
  onCleanup,
} from "solid-js";
import { useNavigate } from "@solidjs/router";
import { Button } from "@/components/ui/Button";
import { LoadingSpinner } from "@/components/ui/LoadingSpinner";
import { apiClient } from "@/api/client";
import type { PlantCandidate, IdentifyPlantResponse } from "@/api/client";
import { plantsStore } from "@/stores/plants";
import type { PlantFormData, CareTaskFormData } from "@/types";

type WizardStep = "capture" | "analyzing" | "candidates" | "form";

interface Props {
  /** If true, the user has no AI configured — show manual-only */
  aiUnavailable?: boolean;
}

export const PlantCreationWizard: Component<Props> = (props) => {
  const navigate = useNavigate();

  const [step, setStep] = createSignal<WizardStep>(
    props.aiUnavailable ? "form" : "capture",
  );
  const [imageDataUrl, setImageDataUrl] = createSignal<string | null>(null);
  const [imageFile, setImageFile] = createSignal<File | null>(null);
  const [identifyResult, setIdentifyResult] =
    createSignal<IdentifyPlantResponse | null>(null);
  const [selectedCandidate, setSelectedCandidate] =
    createSignal<PlantCandidate | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [submitting, setSubmitting] = createSignal(false);

  // Camera refs
  let fileInputRef: HTMLInputElement | undefined;
  let videoRef: HTMLVideoElement | undefined;
  let canvasRef: HTMLCanvasElement | undefined;
  const [showLiveCamera, setShowLiveCamera] = createSignal(false);
  const [cameraStream, setCameraStream] = createSignal<MediaStream | null>(
    null,
  );

  onCleanup(() => {
    stopCamera();
  });

  const isMobile = () =>
    /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(
      navigator.userAgent,
    ) ||
    (navigator.maxTouchPoints && navigator.maxTouchPoints > 2);

  // ─── Camera / Photo Capture ─────────────────────────────────────────────────

  const stopCamera = () => {
    const stream = cameraStream();
    if (stream) {
      stream.getTracks().forEach((t) => t.stop());
      setCameraStream(null);
    }
    setShowLiveCamera(false);
  };

  const startCamera = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: "environment", width: { ideal: 1920 } },
      });
      setCameraStream(stream);
      setShowLiveCamera(true);
      // Wait for DOM update then attach stream
      requestAnimationFrame(() => {
        if (videoRef) {
          videoRef.srcObject = stream;
          videoRef.play();
        }
      });
    } catch (err) {
      console.error("Camera access denied:", err);
      setError(
        "Camera access denied. Please allow camera permissions or use file upload.",
      );
    }
  };

  const captureFromCamera = () => {
    if (!videoRef || !canvasRef) return;
    const canvas = canvasRef;
    const ctx = canvas.getContext("2d")!;
    canvas.width = videoRef.videoWidth;
    canvas.height = videoRef.videoHeight;
    ctx.drawImage(videoRef, 0, 0);
    stopCamera();

    canvas.toBlob(
      (blob) => {
        if (!blob) return;
        const file = new File([blob], "plant-photo.jpg", {
          type: "image/jpeg",
        });
        setImageFile(file);
        const reader = new FileReader();
        reader.onload = () => {
          setImageDataUrl(reader.result as string);
          startIdentification(reader.result as string);
        };
        reader.readAsDataURL(file);
      },
      "image/jpeg",
      0.85,
    );
  };

  const handleFileSelect = (e: Event) => {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    compressAndProcess(file);
  };

  const compressAndProcess = (file: File) => {
    // Compress to max 1024px for identification (don't need full resolution)
    const canvas = document.createElement("canvas");
    const ctx = canvas.getContext("2d")!;
    const img = new Image();
    const objectUrl = URL.createObjectURL(file);

    img.onload = () => {
      URL.revokeObjectURL(objectUrl);
      const maxDim = 1280;
      let w = img.width;
      let h = img.height;
      if (w > maxDim || h > maxDim) {
        if (w > h) {
          h = (h * maxDim) / w;
          w = maxDim;
        } else {
          w = (w * maxDim) / h;
          h = maxDim;
        }
      }
      canvas.width = w;
      canvas.height = h;
      ctx.drawImage(img, 0, 0, w, h);
      canvas.toBlob(
        (blob) => {
          if (!blob) return;
          const compressed = new File([blob], file.name, {
            type: "image/jpeg",
          });
          setImageFile(compressed);
          const reader = new FileReader();
          reader.onload = () => {
            setImageDataUrl(reader.result as string);
            startIdentification(reader.result as string);
          };
          reader.readAsDataURL(compressed);
        },
        "image/jpeg",
        0.82,
      );
    };
    img.src = objectUrl;
  };

  // ─── AI Identification ──────────────────────────────────────────────────────

  const startIdentification = async (dataUrl: string) => {
    setStep("analyzing");
    setError(null);

    try {
      const result = await apiClient.identifyPlant(dataUrl);
      setIdentifyResult(result);

      if (result.auto_select && result.candidates.length > 0) {
        // High confidence single match — go directly to form
        setSelectedCandidate(result.candidates[0]);
        setStep("form");
      } else if (result.candidates.length === 0) {
        // No identification possible
        setError(
          "Could not identify the plant. You can add it manually below.",
        );
        setStep("form");
      } else {
        // Multiple candidates — let user choose
        setStep("candidates");
      }
    } catch (err: unknown) {
      const msg =
        err instanceof Error ? err.message : "Identification failed";
      setError(msg);
      setStep("form"); // Fall through to manual
    }
  };

  // ─── Form Submission ────────────────────────────────────────────────────────

  const handleFormSubmit = async (data: PlantFormData & { previewFile?: File }): Promise<void> => {
    try {
      setSubmitting(true);
      // Use the captured image as the preview if we have one
      const submitData = {
        ...data,
        previewFile: data.previewFile || imageFile() || undefined,
      };
      const plant = await plantsStore.createPlant(submitData);
      navigate(`/plants/${plant.id}`);
    } catch (err) {
      console.error("Failed to create plant:", err);
      setError(err instanceof Error ? err.message : "Failed to create plant");
    } finally {
      setSubmitting(false);
    }
  };

  // ─── Derive form initial data from selected candidate ───────────────────────

  const candidateFormData = (): PlantFormData | undefined => {
    const candidate = selectedCandidate();
    if (!candidate) return undefined;

    const careTasks: CareTaskFormData[] = [];
    const care = candidate.suggested_care;

    if (care.watering_interval_days) {
      careTasks.push({
        name: "Water",
        icon: "\uD83D\uDCA7",
        intervalDays: care.watering_interval_days,
        notes: care.additional_notes || undefined,
      });
    }
    if (care.fertilizing_interval_days) {
      careTasks.push({
        name: "Fertilize",
        icon: "\uD83C\uDF31",
        intervalDays: care.fertilizing_interval_days,
      });
    }

    // Derive a friendly name from the common name or genus
    const displayName =
      candidate.common_name ||
      candidate.scientific_name.split(" ")[0];

    return {
      name: displayName,
      genus: candidate.scientific_name,
      careTasks:
        careTasks.length > 0
          ? careTasks
          : [
              { name: "Water", icon: "\uD83D\uDCA7", intervalDays: 7 },
              { name: "Fertilize", icon: "\uD83C\uDF31", intervalDays: 14 },
            ],
      customMetrics: [],
    };
  };

  // ─── Render ─────────────────────────────────────────────────────────────────

  return (
    <div class="min-h-full pb-20 sm:pb-8">
      {/* Header */}
      <div class="px-4 sm:px-6 pt-4 sm:pt-6 pb-4">
        <div class="max-w-4xl mx-auto">
          <div class="flex items-center space-x-3">
            <button
              onClick={() => navigate("/plants")}
              class="p-2 -ml-2 text-gray-400 hover:text-gray-600 hover:bg-gray-50 rounded-lg transition-colors"
              aria-label="Back"
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
                  d="M15 19l-7-7 7-7"
                />
              </svg>
            </button>
            <div>
              <h1 class="text-xl sm:text-2xl font-bold text-gray-900">
                Add New Plant
              </h1>
              <p class="text-sm text-gray-500 mt-0.5">
                <Switch>
                  <Match when={step() === "capture"}>
                    Take a photo to identify your plant
                  </Match>
                  <Match when={step() === "analyzing"}>
                    Analyzing your plant...
                  </Match>
                  <Match when={step() === "candidates"}>
                    Select the best match
                  </Match>
                  <Match when={step() === "form"}>
                    Review and customize details
                  </Match>
                </Switch>
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Step indicator */}
      <Show when={!props.aiUnavailable}>
        <div class="px-4 sm:px-6 pb-4">
          <div class="max-w-4xl mx-auto">
            <div class="flex items-center gap-2">
              <StepDot
                active={step() === "capture"}
                completed={
                  step() === "analyzing" ||
                  step() === "candidates" ||
                  step() === "form"
                }
                label="Photo"
              />
              <StepLine
                completed={
                  step() === "candidates" || step() === "form"
                }
              />
              <StepDot
                active={step() === "analyzing" || step() === "candidates"}
                completed={step() === "form"}
                label="Identify"
              />
              <StepLine completed={step() === "form"} />
              <StepDot active={step() === "form"} completed={false} label="Details" />
            </div>
          </div>
        </div>
      </Show>

      {/* Content */}
      <div class="px-4 sm:px-6">
        <div class="max-w-4xl mx-auto">
          {/* Error banner */}
          <Show when={error()}>
            <div class="mb-4 bg-amber-50 border border-amber-200 rounded-xl p-4 flex items-start gap-3">
              <svg
                class="h-5 w-5 text-amber-500 flex-shrink-0 mt-0.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"
                />
              </svg>
              <p class="text-sm text-amber-800">{error()}</p>
            </div>
          </Show>

          {/* AI unavailable message */}
          <Show when={props.aiUnavailable && step() === "form"}>
            <div class="mb-4 bg-blue-50 border border-blue-200 rounded-xl p-4 flex items-start gap-3">
              <svg
                class="h-5 w-5 text-blue-500 flex-shrink-0 mt-0.5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
              <p class="text-sm text-blue-800">
                Set up AI in{" "}
                <a href="/settings" class="underline font-medium">
                  Settings
                </a>{" "}
                to identify plants from photos automatically.
              </p>
            </div>
          </Show>

          {/* ─── STEP: Capture ─────────────────────────────────────────────── */}
          <Show when={step() === "capture"}>
            <div class="bg-white shadow-sm rounded-xl border border-gray-200 overflow-hidden">
              {/* Live camera view */}
              <Show when={showLiveCamera()}>
                <div class="relative bg-black aspect-[4/3] sm:aspect-video">
                  <video
                    ref={videoRef}
                    class="w-full h-full object-cover"
                    autoplay
                    playsinline
                    muted
                  />
                  <div class="absolute bottom-4 left-0 right-0 flex justify-center gap-4">
                    <button
                      onClick={captureFromCamera}
                      class="w-16 h-16 bg-white rounded-full border-4 border-gray-300 shadow-lg flex items-center justify-center hover:scale-105 transition-transform"
                      aria-label="Take photo"
                    >
                      <div class="w-12 h-12 bg-primary-500 rounded-full" />
                    </button>
                    <button
                      onClick={stopCamera}
                      class="w-12 h-12 bg-gray-800/70 rounded-full flex items-center justify-center text-white"
                      aria-label="Cancel camera"
                    >
                      <svg
                        class="h-6 w-6"
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
                </div>
              </Show>

              {/* Capture prompt (no camera active) */}
              <Show when={!showLiveCamera()}>
                <div class="p-6 sm:p-10 flex flex-col items-center text-center">
                  <div class="w-20 h-20 sm:w-24 sm:h-24 bg-primary-50 rounded-full flex items-center justify-center mb-6">
                    <svg
                      class="h-10 w-10 sm:h-12 sm:w-12 text-primary-500"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width={1.5}
                        d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z"
                      />
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width={1.5}
                        d="M15 13a3 3 0 11-6 0 3 3 0 016 0z"
                      />
                    </svg>
                  </div>
                  <h2 class="text-lg sm:text-xl font-semibold text-gray-900 mb-2">
                    Take a photo of your plant
                  </h2>
                  <p class="text-sm text-gray-500 mb-8 max-w-md">
                    Our AI will identify the species and suggest care schedules.
                    For best results, capture clear leaves and overall shape.
                  </p>

                  <div class="flex flex-col sm:flex-row gap-3 w-full sm:w-auto">
                    <Show when={isMobile()}>
                      <Button
                        variant="primary"
                        size="lg"
                        onClick={startCamera}
                        class="flex-1 sm:flex-initial"
                      >
                        <svg
                          class="h-5 w-5 mr-2"
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
                        Take Photo
                      </Button>
                    </Show>
                    <Button
                      variant={isMobile() ? "outline" : "primary"}
                      size="lg"
                      onClick={() => fileInputRef?.click()}
                      class="flex-1 sm:flex-initial"
                    >
                      <svg
                        class="h-5 w-5 mr-2"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          stroke-width={2}
                          d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                        />
                      </svg>
                      Choose from Gallery
                    </Button>
                  </div>

                  <input
                    ref={fileInputRef}
                    type="file"
                    accept="image/*"
                    class="hidden"
                    onChange={handleFileSelect}
                  />
                </div>
              </Show>
            </div>

            {/* Skip link */}
            <div class="mt-4 text-center">
              <button
                onClick={() => setStep("form")}
                class="text-sm text-gray-500 hover:text-gray-700 underline transition-colors"
              >
                Skip, add manually
              </button>
            </div>
          </Show>

          {/* ─── STEP: Analyzing ──────────────────────────────────────────── */}
          <Show when={step() === "analyzing"}>
            <div class="bg-white shadow-sm rounded-xl border border-gray-200 p-8 sm:p-12">
              <div class="flex flex-col items-center text-center">
                {/* Show the captured image */}
                <Show when={imageDataUrl()}>
                  <div class="w-32 h-32 sm:w-40 sm:h-40 rounded-xl overflow-hidden mb-6 ring-2 ring-primary-100">
                    <img
                      src={imageDataUrl()!}
                      alt="Captured plant"
                      class="w-full h-full object-cover"
                    />
                  </div>
                </Show>

                <LoadingSpinner size="lg" />
                <h2 class="text-lg font-semibold text-gray-900 mt-4 mb-2">
                  Identifying your plant...
                </h2>
                <p class="text-sm text-gray-500 max-w-sm">
                  Analyzing leaf shape, growth pattern, and other features to
                  find the best match.
                </p>
              </div>
            </div>
          </Show>

          {/* ─── STEP: Candidates ─────────────────────────────────────────── */}
          <Show when={step() === "candidates"}>
            <CandidateSelection
              candidates={identifyResult()?.candidates || []}
              analysisNotes={identifyResult()?.analysis_notes || ""}
              onSelect={(candidate) => {
                setSelectedCandidate(candidate);
                setStep("form");
              }}
              onSkip={() => setStep("form")}
            />
          </Show>

          {/* ─── STEP: Form ───────────────────────────────────────────────── */}
          <Show when={step() === "form"}>
            <PlantFormWithPreFill
              initialData={candidateFormData()}
              previewImageUrl={imageDataUrl()}
              previewFile={imageFile()}
              candidate={selectedCandidate()}
              onSubmit={handleFormSubmit}
              loading={submitting()}
              onBack={() => {
                if (identifyResult()?.candidates?.length) {
                  setStep("candidates");
                } else {
                  setStep("capture");
                }
              }}
            />
          </Show>
        </div>
      </div>

      {/* Hidden canvas for camera capture */}
      <canvas ref={canvasRef} class="hidden" />
    </div>
  );
};

// ─── Sub-components ───────────────────────────────────────────────────────────

const StepDot: Component<{
  active: boolean;
  completed: boolean;
  label: string;
}> = (props) => (
  <div class="flex flex-col items-center gap-1">
    <div
      class={`w-3 h-3 rounded-full transition-colors ${
        props.completed
          ? "bg-primary-500"
          : props.active
            ? "bg-primary-400 ring-4 ring-primary-100"
            : "bg-gray-200"
      }`}
    />
    <span
      class={`text-xs ${props.active || props.completed ? "text-primary-700 font-medium" : "text-gray-400"}`}
    >
      {props.label}
    </span>
  </div>
);

const StepLine: Component<{ completed: boolean }> = (props) => (
  <div
    class={`flex-1 h-0.5 rounded transition-colors ${props.completed ? "bg-primary-400" : "bg-gray-200"}`}
  />
);

// ─── Candidate Selection ──────────────────────────────────────────────────────

const CandidateSelection: Component<{
  candidates: PlantCandidate[];
  analysisNotes: string;
  onSelect: (candidate: PlantCandidate) => void;
  onSkip: () => void;
}> = (props) => {
  return (
    <div class="space-y-4">
      {/* Analysis context */}
      <Show when={props.analysisNotes}>
        <div class="bg-gray-50 rounded-xl p-4 border border-gray-100">
          <p class="text-sm text-gray-600 italic">{props.analysisNotes}</p>
        </div>
      </Show>

      {/* Candidate cards */}
      <div class="grid gap-3 sm:gap-4">
        <For each={props.candidates}>
          {(candidate) => (
            <CandidateCard
              candidate={candidate}
              onSelect={() => props.onSelect(candidate)}
            />
          )}
        </For>
      </div>

      {/* None of these */}
      <div class="text-center pt-2">
        <button
          onClick={props.onSkip}
          class="text-sm text-gray-500 hover:text-gray-700 underline transition-colors"
        >
          None of these — add manually
        </button>
      </div>
    </div>
  );
};

// ─── Individual Candidate Card ────────────────────────────────────────────────

const CandidateCard: Component<{
  candidate: PlantCandidate;
  onSelect: () => void;
}> = (props) => {
  const confidencePercent = () => Math.round(props.candidate.confidence * 100);
  const confidenceColor = () => {
    if (props.candidate.confidence >= 0.8) return "text-green-700 bg-green-50";
    if (props.candidate.confidence >= 0.5)
      return "text-amber-700 bg-amber-50";
    return "text-gray-600 bg-gray-50";
  };

  return (
    <button
      onClick={props.onSelect}
      class="w-full bg-white rounded-xl border border-gray-200 shadow-sm hover:border-primary-300 hover:shadow-md transition-all duration-200 overflow-hidden text-left group"
    >
      <div class="flex gap-3 sm:gap-4 p-3 sm:p-4">
        {/* Reference image */}
        <div class="flex-shrink-0 w-20 h-20 sm:w-24 sm:h-24 rounded-lg overflow-hidden bg-gray-100">
          <Show
            when={props.candidate.reference_images.length > 0}
            fallback={
              <div class="w-full h-full flex items-center justify-center text-gray-300">
                <svg
                  class="h-8 w-8"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width={1.5}
                    d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                  />
                </svg>
              </div>
            }
          >
            <img
              src={props.candidate.reference_images[0]}
              alt={props.candidate.scientific_name}
              class="w-full h-full object-cover"
              loading="lazy"
            />
          </Show>
        </div>

        {/* Info */}
        <div class="flex-1 min-w-0">
          <div class="flex items-start justify-between gap-2">
            <div class="min-w-0">
              <h3 class="font-semibold text-gray-900 truncate group-hover:text-primary-700 transition-colors">
                {props.candidate.common_name || props.candidate.genus}
              </h3>
              <p class="text-sm text-gray-500 italic truncate">
                {props.candidate.scientific_name}
              </p>
            </div>
            <span
              class={`flex-shrink-0 text-xs font-bold px-2 py-0.5 rounded-full ${confidenceColor()}`}
            >
              {confidencePercent()}%
            </span>
          </div>

          <p class="text-xs text-gray-500 mt-1.5 line-clamp-2">
            {props.candidate.reasoning}
          </p>

          {/* Care tags */}
          <div class="flex flex-wrap gap-1.5 mt-2">
            <Show when={props.candidate.suggested_care.light_requirement}>
              <CareTag
                icon="\u2600\uFE0F"
                text={props.candidate.suggested_care.light_requirement!}
              />
            </Show>
            <Show when={props.candidate.suggested_care.watering_interval_days}>
              <CareTag
                icon="\uD83D\uDCA7"
                text={`Every ${props.candidate.suggested_care.watering_interval_days}d`}
              />
            </Show>
          </div>
        </div>

        {/* Chevron */}
        <div class="flex-shrink-0 self-center text-gray-300 group-hover:text-primary-400 transition-colors">
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
              d="M9 5l7 7-7 7"
            />
          </svg>
        </div>
      </div>
    </button>
  );
};

const CareTag: Component<{ icon: string; text: string }> = (props) => (
  <span class="inline-flex items-center gap-0.5 text-xs bg-gray-100 text-gray-600 rounded-full px-2 py-0.5">
    <span>{props.icon}</span>
    <span class="truncate max-w-[8rem]">{props.text}</span>
  </span>
);

// ─── Pre-filled Plant Form Wrapper ────────────────────────────────────────────

import { PlantForm } from "@/components/plants/PlantForm";

const PlantFormWithPreFill: Component<{
  initialData?: PlantFormData;
  previewImageUrl?: string | null;
  previewFile?: File | null;
  candidate?: PlantCandidate | null;
  onSubmit: (data: PlantFormData & { previewFile?: File }) => Promise<void>;
  loading: boolean;
  onBack: () => void;
}> = (props) => {
  return (
    <div class="space-y-4">
      {/* Selected candidate summary */}
      <Show when={props.candidate}>
        <div class="bg-primary-50 border border-primary-100 rounded-xl p-4 flex items-start gap-3">
          <Show when={props.previewImageUrl}>
            <img
              src={props.previewImageUrl!}
              alt="Your plant"
              class="w-14 h-14 rounded-lg object-cover flex-shrink-0"
            />
          </Show>
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <h3 class="font-semibold text-primary-900 truncate">
                {props.candidate!.common_name ||
                  props.candidate!.scientific_name}
              </h3>
              <span class="text-xs font-bold text-primary-700 bg-primary-100 px-1.5 py-0.5 rounded-full">
                {Math.round(props.candidate!.confidence * 100)}%
              </span>
            </div>
            <p class="text-sm text-primary-700 italic">
              {props.candidate!.scientific_name}
            </p>
            <Show when={props.candidate!.suggested_care.light_requirement}>
              <p class="text-xs text-primary-600 mt-1">
                \u2600\uFE0F {props.candidate!.suggested_care.light_requirement}
                <Show when={props.candidate!.suggested_care.humidity_notes}>
                  {" "}
                  &bull; {props.candidate!.suggested_care.humidity_notes}
                </Show>
              </p>
            </Show>
          </div>
          <button
            onClick={props.onBack}
            class="flex-shrink-0 text-primary-600 hover:text-primary-800 text-sm underline"
          >
            Change
          </button>
        </div>
      </Show>

      {/* The actual form */}
      <div class="bg-white shadow-sm rounded-xl border border-gray-200 overflow-hidden">
        <div class="px-4 sm:px-6 py-4 border-b border-gray-100 bg-gray-50/50">
          <div class="flex items-center space-x-3">
            <div class="w-8 h-8 bg-primary-100 rounded-lg flex items-center justify-center">
              <svg
                class="h-4 w-4 text-primary-600"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width={2}
                  d="M12 6v6m0 0v6m0-6h6m-6 0H6"
                />
              </svg>
            </div>
            <div>
              <h2 class="text-base font-semibold text-gray-900">
                Plant Details
              </h2>
              <p class="text-xs text-gray-500">
                {props.candidate
                  ? "Pre-filled from identification — edit anything below"
                  : "Fill in the details for your new plant"}
              </p>
            </div>
          </div>
        </div>
        <div class="p-4 sm:p-6">
          <PlantForm
            initialData={props.initialData}
            isEditing={false}
            existingPreviewUrl={props.previewImageUrl || null}
            onSubmit={props.onSubmit}
            submitText="Create Plant"
            loading={props.loading}
          />
        </div>
      </div>
    </div>
  );
};
