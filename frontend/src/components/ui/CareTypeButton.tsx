import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '@/utils/cn';
import { CareIcon } from './CareIcon';
import type { components } from '@/types/api-generated';

type EntryType = components['schemas']['EntryType'];

interface CareTypeButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  type: EntryType;
  isActive: boolean;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'default' | 'compact' | 'pill';
  showIcon?: boolean;
  showLabel?: boolean;
  children?: JSX.Element;
}

export const CareTypeButton: Component<CareTypeButtonProps> = (props) => {
  const [local, rest] = splitProps(props, [
    'type', 'isActive', 'size', 'variant', 'showIcon', 'showLabel', 'children', 'class'
  ]);

  const careTypeClasses = {
    watering: local.isActive 
      ? 'bg-blue-100 border-blue-300 text-blue-800 dark:bg-blue-900 dark:border-blue-700 dark:text-blue-100' 
      : 'bg-white border-gray-200 text-gray-700 hover:bg-blue-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-blue-900',
    fertilizing: local.isActive 
      ? 'bg-green-100 border-green-300 text-green-800 dark:bg-green-900 dark:border-green-700 dark:text-green-100' 
      : 'bg-white border-gray-200 text-gray-700 hover:bg-green-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-green-900',
    pruning: local.isActive 
      ? 'bg-purple-100 border-purple-300 text-purple-800 dark:bg-purple-900 dark:border-purple-700 dark:text-purple-100' 
      : 'bg-white border-gray-200 text-gray-700 hover:bg-purple-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-purple-900',
    repotting: local.isActive 
      ? 'bg-orange-100 border-orange-300 text-orange-800 dark:bg-orange-900 dark:border-orange-700 dark:text-orange-100' 
      : 'bg-white border-gray-200 text-gray-700 hover:bg-orange-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-orange-900',
    'pest-control': local.isActive 
      ? 'bg-red-100 border-red-300 text-red-800 dark:bg-red-900 dark:border-red-700 dark:text-red-100' 
      : 'bg-white border-gray-200 text-gray-700 hover:bg-red-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-red-900',
    note: local.isActive 
      ? 'bg-gray-100 border-gray-300 text-gray-800 dark:bg-gray-700 dark:border-gray-500 dark:text-gray-100' 
      : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-700',
    custom: local.isActive 
      ? 'bg-indigo-100 border-indigo-300 text-indigo-800 dark:bg-indigo-900 dark:border-indigo-700 dark:text-indigo-100' 
      : 'bg-white border-gray-200 text-gray-700 hover:bg-indigo-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-indigo-900',
  };

  const sizeClasses = {
    sm: 'p-2 text-xs',
    md: 'p-3 text-sm',
    lg: 'p-4 text-base',
  };

  const variantClasses = {
    default: 'rounded-lg border',
    compact: 'rounded-md border',
    pill: 'rounded-full border',
  };

  const careTypeLabels = {
    watering: 'Watering',
    fertilizing: 'Fertilizing', 
    pruning: 'Pruning',
    repotting: 'Repotting',
    'pest-control': 'Pest Control',
    note: 'Note',
    custom: 'Custom',
  };

  // Map EntryType to CareIcon type
  const mapToCareIconType = (type: EntryType): Parameters<typeof CareIcon>[0]['type'] => {
    switch (type) {
      case 'watering':
      case 'fertilizing':
        return type;
      default:
        return 'custom';
    }
  };

  return (
    <button
      class={cn(
        'flex flex-col items-center justify-center space-y-1 transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500',
        variantClasses[local.variant || 'default'],
        sizeClasses[local.size || 'md'],
        careTypeClasses[local.type as keyof typeof careTypeClasses] || careTypeClasses.custom,
        local.class
      )}
      {...rest}
    >
      <div class="care-type-icon-container">
        {local.children || (
          <>
            {(local.showIcon !== false) && (
              <CareIcon 
                type={mapToCareIconType(local.type)} 
                size={local.size === 'lg' ? 'lg' : local.size === 'sm' ? 'sm' : 'md'} 
                class="care-type-icon"
              />
            )}
            {(local.showLabel !== false) && (
              <span class="care-type-label">
                {careTypeLabels[local.type as keyof typeof careTypeLabels] || local.type}
              </span>
            )}
          </>
        )}
      </div>
    </button>
  );
}; 