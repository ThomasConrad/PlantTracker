import { Component } from 'solid-js';
import { LoadingSpinner } from './LoadingSpinner';

interface PageLoadingStateProps {
  title?: string;
  subtitle?: string;
  size?: 'sm' | 'md' | 'lg';
}

export const PageLoadingState: Component<PageLoadingStateProps> = (props) => {
  return (
    <div class="flex justify-center py-16 sm:py-20">
      <div class="flex flex-col items-center gap-4">
        <LoadingSpinner size={props.size || 'lg'} />
        <div class="text-center">
          <p class="text-sm sm:text-base text-gray-500 font-medium">
            {props.title || 'Loading...'}
          </p>
          {props.subtitle && (
            <p class="text-xs sm:text-sm text-gray-400 mt-1">
              {props.subtitle}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}; 