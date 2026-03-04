import { Component, For, Show, createMemo, createSignal, onCleanup, onMount, splitProps } from 'solid-js';
import { cn } from '@/utils/cn';
import type { SelectProps } from './types';

export const Select: Component<SelectProps> = (props) => {
  const [local, rest] = splitProps(props, [
    'label',
    'error',
    'class',
    'id',
    'options',
    'placeholder',
    'value',
    'onValueChange',
    'disabled',
  ]);

  const selectId = local.id || `select-${Math.random().toString(36).slice(2, 11)}`;
  let rootRef: HTMLDivElement | undefined;
  const [isOpen, setIsOpen] = createSignal(false);

  const selectedLabel = createMemo(() => {
    const selected = local.options.find((option) => option.value === local.value);
    return selected?.label || local.placeholder || 'Select...';
  });

  const closeMenu = () => setIsOpen(false);

  const handleOutsideClick = (event: MouseEvent | PointerEvent) => {
    if (!rootRef || !event.target) return;
    if (!rootRef.contains(event.target as Node)) {
      closeMenu();
    }
  };

  const handleEscape = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      closeMenu();
    }
  };

  const handleSelect = (value: string) => {
    if (local.disabled) return;
    local.onValueChange?.(value);
    closeMenu();
  };

  onMount(() => {
    document.addEventListener('pointerdown', handleOutsideClick);
    document.addEventListener('keydown', handleEscape);
  });

  onCleanup(() => {
    document.removeEventListener('pointerdown', handleOutsideClick);
    document.removeEventListener('keydown', handleEscape);
  });

  return (
    <div class="space-y-1">
      <Show when={local.label}>
        <label for={selectId} class="label">
          {local.label}
        </label>
      </Show>

      <div class="relative" ref={rootRef}>
        <button
          id={selectId}
          type="button"
          disabled={local.disabled}
          aria-haspopup="listbox"
          aria-expanded={isOpen()}
          onClick={() => setIsOpen((open) => !open)}
          class={cn(
            'input w-full appearance-none pr-10 text-left text-base sm:text-sm',
            local.error && 'border-red-500 focus:border-red-500 focus:ring-red-500',
            local.class
          )}
        >
          {selectedLabel()}
        </button>

        <svg
          class="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-500"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M19 9l-7 7-7-7" />
        </svg>

        <Show when={isOpen()}>
          <div class="absolute z-50 mt-1 max-h-60 w-full overflow-auto rounded-md border border-gray-200 bg-white shadow-lg">
            <div role="listbox" class="py-1">
              <For each={local.options}>
                {(option) => (
                  <button
                    type="button"
                    class={cn(
                      'w-full px-3 py-2 text-left text-sm hover:bg-gray-50',
                      option.value === local.value && 'bg-green-50 text-green-700'
                    )}
                    onClick={() => handleSelect(option.value)}
                  >
                    {option.label}
                  </button>
                )}
              </For>
            </div>
          </div>
        </Show>

        <input type="hidden" name={rest.name} value={local.value as string | undefined} />
      </div>

      <Show when={local.error}>
        <p class="error-text">{local.error}</p>
      </Show>
    </div>
  );
};
