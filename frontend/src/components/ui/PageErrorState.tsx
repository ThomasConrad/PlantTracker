import { Component } from 'solid-js';

interface PageErrorStateProps {
  title?: string;
  message: string;
  maxWidth?: 'md' | 'lg' | 'xl' | '2xl' | '3xl' | '4xl' | '5xl' | '6xl' | '7xl';
}

export const PageErrorState: Component<PageErrorStateProps> = (props) => {
  const maxWidthClass = () => {
    const maxWidth = props.maxWidth || '7xl';
    return `max-w-${maxWidth}`;
  };

  return (
    <div class="px-4 sm:px-6 mt-6">
      <div class={`${maxWidthClass()} mx-auto`}>
        <div class="bg-red-50 border border-red-200 rounded-xl p-4">
          <div class="flex items-start gap-3">
            <div class="flex-shrink-0">
              <svg class="h-5 w-5 text-red-500 mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-sm font-medium text-red-800">
                {props.title || 'Something went wrong'}
              </h3>
              <p class="text-sm text-red-700 mt-1">
                {props.message}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}; 