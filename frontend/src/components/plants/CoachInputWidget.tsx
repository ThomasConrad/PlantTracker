import { Component, createSignal, For, Show } from "solid-js";
import type { InputRequest, SliderParams, SelectParams, ToggleParams, PhotoParams } from "@/api/coach";
import { Button } from "@/components/ui/Button";
import { compressImage } from "@/utils/imageCompress";

interface CoachInputWidgetProps {
  request: InputRequest;
  onSubmit: (key: string, value: string) => void;
  disabled?: boolean;
}

const ToggleWidget: Component<{ params: ToggleParams; onSubmit: (key: string, value: string) => void; disabled?: boolean }> = (props) => {
  return (
    <div class="flex items-center justify-between gap-3 py-1">
      <span class="text-sm text-gray-700 dark:text-gray-200">{props.params.label}</span>
      <div class="flex gap-2">
        <Button
          size="sm"
          variant="outline"
          onClick={() => props.onSubmit(props.params.key, "yes")}
          disabled={props.disabled}
        >
          Yes
        </Button>
        <Button
          size="sm"
          variant="outline"
          onClick={() => props.onSubmit(props.params.key, "no")}
          disabled={props.disabled}
        >
          No
        </Button>
      </div>
    </div>
  );
};

const SliderWidget: Component<{ params: SliderParams; onSubmit: (key: string, value: string) => void; disabled?: boolean }> = (props) => {
  const [value, setValue] = createSignal(Math.round((props.params.min + props.params.max) / 2));
  const [submitted, setSubmitted] = createSignal(false);

  const handleSubmit = () => {
    setSubmitted(true);
    props.onSubmit(props.params.key, String(value()));
  };

  return (
    <div class="py-1 space-y-2">
      <span class="text-sm text-gray-700 dark:text-gray-200">{props.params.label}</span>
      <div class="flex items-center gap-3">
        <Show when={props.params.min_label}>
          <span class="text-xs text-gray-400 dark:text-gray-500 whitespace-nowrap">{props.params.min_label}</span>
        </Show>
        <input
          type="range"
          min={props.params.min}
          max={props.params.max}
          step={props.params.step}
          value={value()}
          onInput={(e) => setValue(Number(e.currentTarget.value))}
          disabled={props.disabled || submitted()}
          class="flex-1 h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-primary-600"
        />
        <Show when={props.params.max_label}>
          <span class="text-xs text-gray-400 dark:text-gray-500 whitespace-nowrap">{props.params.max_label}</span>
        </Show>
        <span class="text-sm font-medium text-gray-700 dark:text-gray-200 min-w-[2ch] text-center">{value()}</span>
      </div>
      <Show when={!submitted()}>
        <Button size="sm" variant="primary" onClick={handleSubmit} disabled={props.disabled}>
          Confirm
        </Button>
      </Show>
    </div>
  );
};

const SelectWidget: Component<{ params: SelectParams; onSubmit: (key: string, value: string) => void; disabled?: boolean; multiple?: boolean }> = (props) => {
  const [selected, setSelected] = createSignal<string[]>([]);

  const toggle = (val: string) => {
    if (props.multiple) {
      setSelected((prev) => prev.includes(val) ? prev.filter((v) => v !== val) : [...prev, val]);
    } else {
      // Single select: submit immediately
      props.onSubmit(props.params.key, val);
    }
  };

  const handleSubmit = () => {
    props.onSubmit(props.params.key, selected().join(", "));
  };

  return (
    <div class="py-1 space-y-2">
      <span class="text-sm text-gray-700 dark:text-gray-200">{props.params.label}</span>
      <div class="flex flex-wrap gap-2">
        <For each={props.params.options}>
          {(option) => (
            <button
              onClick={() => toggle(option.value)}
              disabled={props.disabled}
              class={`px-3 py-1.5 rounded-full text-sm border transition-colors ${
                selected().includes(option.value)
                  ? "bg-primary-100 dark:bg-primary-900 border-primary-500 text-primary-700 dark:text-primary-300"
                  : "bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:border-primary-400"
              }`}
            >
              {option.label}
            </button>
          )}
        </For>
      </div>
      <Show when={props.multiple && selected().length > 0}>
        <Button size="sm" variant="primary" onClick={handleSubmit} disabled={props.disabled}>
          Confirm
        </Button>
      </Show>
    </div>
  );
};

const PhotoWidget: Component<{ params: PhotoParams; onSubmit: (key: string, value: string) => void; disabled?: boolean }> = (props) => {
  let fileInputRef: HTMLInputElement | undefined;

  const handleFileSelect = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;
    try {
      const dataUrl = await compressImage(file, { maxDimension: 1024, quality: 0.8 });
      props.onSubmit(props.params.key, dataUrl);
    } catch {
      const reader = new FileReader();
      reader.onload = () => props.onSubmit(props.params.key, reader.result as string);
      reader.readAsDataURL(file);
    }
    target.value = "";
  };

  return (
    <div class="py-1 space-y-2">
      <span class="text-sm text-gray-700 dark:text-gray-200">{props.params.label}</span>
      <input ref={fileInputRef} type="file" accept="image/*" class="hidden" onChange={handleFileSelect} />
      <Button size="sm" variant="outline" onClick={() => fileInputRef?.click()} disabled={props.disabled}>
        <svg class="w-4 h-4 mr-1.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 13a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        Take photo
      </Button>
    </div>
  );
};

export const CoachInputWidget: Component<CoachInputWidgetProps> = (props) => {
  return (
    <div class="mt-2 p-3 bg-white dark:bg-gray-700 rounded-lg border border-gray-200 dark:border-gray-600">
      {props.request.template === "toggle" && (
        <ToggleWidget params={props.request.params as ToggleParams} onSubmit={props.onSubmit} disabled={props.disabled} />
      )}
      {props.request.template === "slider" && (
        <SliderWidget params={props.request.params as SliderParams} onSubmit={props.onSubmit} disabled={props.disabled} />
      )}
      {props.request.template === "select" && (
        <SelectWidget params={props.request.params as SelectParams} onSubmit={props.onSubmit} disabled={props.disabled} />
      )}
      {props.request.template === "multi_select" && (
        <SelectWidget params={props.request.params as SelectParams} onSubmit={props.onSubmit} disabled={props.disabled} multiple />
      )}
      {props.request.template === "photo" && (
        <PhotoWidget params={props.request.params as PhotoParams} onSubmit={props.onSubmit} disabled={props.disabled} />
      )}
    </div>
  );
};
