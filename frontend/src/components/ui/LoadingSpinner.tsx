import { Component } from 'solid-js';
import { cn } from '@/utils/cn';

interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  class?: string;
}

export const LoadingSpinner: Component<LoadingSpinnerProps> = (props) => {
  const sizeClasses = {
    sm: 'h-4 w-4',
    md: 'h-6 w-6',
    lg: 'h-8 w-8',
  };

  return (
    <div
      class={cn(
        'animate-spin rounded-full border-2 border-gray-300 border-t-primary-600',
        sizeClasses[props.size || 'md'],
        props.class
      )}
      role="status"
      aria-label="Loading"
    >
      <span class="sr-only">Loading...</span>
    </div>
  );
};