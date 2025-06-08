import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '@/utils/cn';

interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
}

export const Input: Component<InputProps> = (props) => {
  const [local, rest] = splitProps(props, ['label', 'error', 'class', 'id']);

  const inputId = local.id || `input-${Math.random().toString(36).substr(2, 9)}`;

  return (
    <div class="space-y-1">
      {local.label && (
        <label for={inputId} class="label">
          {local.label}
        </label>
      )}
      <input
        id={inputId}
        class={cn(
          'input',
          local.error && 'border-red-500 focus:border-red-500 focus:ring-red-500',
          local.class
        )}
        {...rest}
      />
      {local.error && (
        <p class="error-text">{local.error}</p>
      )}
    </div>
  );
};