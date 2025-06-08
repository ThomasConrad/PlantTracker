import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '@/utils/cn';

interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'outline' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  loading?: boolean;
}

export const Button: Component<ButtonProps> = (props) => {
  const [local, rest] = splitProps(props, ['variant', 'size', 'loading', 'children', 'class', 'disabled']);

  const variantClasses = {
    primary: 'btn-primary',
    secondary: 'btn-secondary',
    outline: 'btn-outline',
    danger: 'btn-danger',
  };

  const sizeClasses = {
    sm: 'btn-sm',
    md: 'btn-md',
    lg: 'btn-lg',
  };

  return (
    <button
      class={cn(
        'btn',
        variantClasses[local.variant || 'primary'],
        sizeClasses[local.size || 'md'],
        local.loading && 'opacity-50 cursor-not-allowed',
        local.class
      )}
      disabled={local.disabled || local.loading}
      {...rest}
    >
      {local.loading && (
        <div class="mr-2 h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
      )}
      {local.children}
    </button>
  );
};