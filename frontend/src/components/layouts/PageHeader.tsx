import { Component, JSX, Show } from 'solid-js';
import { A } from '@solidjs/router';

interface PageHeaderProps {
  title: string;
  subtitle?: string;
  backUrl: string;
  backLabel?: string;
  maxWidth?: 'md' | 'lg' | 'xl' | '2xl' | '3xl' | '4xl' | '5xl' | '6xl' | '7xl';
  children?: JSX.Element;
  status?: {
    label: string;
    color: 'blue' | 'green' | 'yellow' | 'red' | 'gray';
  };
  badge?: JSX.Element;
}

export const PageHeader: Component<PageHeaderProps> = (props) => {
  const maxWidthClass = () => {
    const maxWidth = props.maxWidth || '4xl';
    return `max-w-${maxWidth}`;
  };

  const statusColorClass = () => {
    if (!props.status) return '';
    
    const colorMap = {
      blue: 'bg-blue-100 text-blue-800',
      green: 'bg-green-100 text-green-800', 
      yellow: 'bg-yellow-100 text-yellow-800',
      red: 'bg-red-100 text-red-800',
      gray: 'bg-gray-100 text-gray-800'
    };
    
    return colorMap[props.status.color] || colorMap.gray;
  };

  return (
    <div class="px-4 sm:px-6 pt-4 sm:pt-6 pb-6">
      <div class={`${maxWidthClass()} mx-auto`}>
        <div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-4 sm:gap-6">
          <div class="flex-1 min-w-0">
            <div class="flex items-center space-x-3 sm:space-x-4">
              <A
                href={props.backUrl}
                class="flex-shrink-0 p-2 -ml-2 text-gray-400 hover:text-gray-600 hover:bg-gray-50 rounded-lg transition-colors duration-200"
                aria-label={props.backLabel || "Back"}
              >
                <svg class="h-5 w-5 sm:h-6 sm:w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 19l-7-7 7-7" />
                </svg>
              </A>
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-3 mb-1">
                  <h1 class="text-xl sm:text-2xl lg:text-3xl font-bold text-gray-900 truncate">
                    {props.title}
                  </h1>
                  <Show when={props.badge}>
                    {props.badge}
                  </Show>
                </div>
                <Show when={props.subtitle}>
                  <p class="text-sm sm:text-base text-gray-600 line-clamp-2">
                    {props.subtitle}
                  </p>
                </Show>
              </div>
            </div>
          </div>
          
          {/* Status indicator */}
          <Show when={props.status}>
            <div class="flex-shrink-0">
              <div class={`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium ${statusColorClass()}`}>
                <svg class="mr-1.5 h-3 w-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                </svg>
                {props.status?.label}
              </div>
            </div>
          </Show>

          {/* Action buttons */}
          <Show when={props.children}>
            <div class="flex items-center gap-2 sm:gap-3 flex-shrink-0">
              {props.children}
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
}; 