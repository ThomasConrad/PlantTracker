import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '@/utils/cn';
import { CareIcon } from './CareIcon';
type EntryType = 'care' | 'measurement' | 'note' | 'photo';

interface CareTypeButtonProps extends Omit<JSX.ButtonHTMLAttributes<HTMLButtonElement>, 'type'> {
  careType: EntryType;
  isActive: boolean;
  size?: 'sm' | 'md' | 'lg';
  variant?: 'default' | 'compact' | 'pill';
  showIcon?: boolean;
  showLabel?: boolean;
  children?: JSX.Element;
}

export const CareTypeButton: Component<CareTypeButtonProps> = (props) => {
  const [local, rest] = splitProps(props, [
    'careType', 'isActive', 'size', 'variant', 'showIcon', 'showLabel', 'children', 'class'
  ]);

  const careTypeClasses = {
    care: local.isActive 
      ? 'bg-blue-100 border-blue-300 text-blue-800 dark:bg-blue-900 dark:border-blue-700 dark:text-blue-100' 
      : 'bg-white border-gray-200 text-gray-700 hover:bg-blue-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-blue-900',
    measurement: local.isActive 
      ? 'bg-purple-100 border-purple-300 text-purple-800 dark:bg-purple-900 dark:border-purple-700 dark:text-purple-100' 
      : 'bg-white border-gray-200 text-gray-700 hover:bg-purple-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-purple-900',
    note: local.isActive 
      ? 'bg-gray-100 border-gray-300 text-gray-800 dark:bg-gray-700 dark:border-gray-500 dark:text-gray-100' 
      : 'bg-white border-gray-200 text-gray-700 hover:bg-gray-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300 dark:hover:bg-gray-700',
    photo: local.isActive 
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

  const careTypeLabels: Record<string, string> = {
    care: 'Care',
    measurement: 'Measurement',
    note: 'Note',
    photo: 'Photo',
  };

  // Map EntryType to CareIcon type
  const mapToCareIconType = (type: EntryType): Parameters<typeof CareIcon>[0]['type'] => {
    switch (type) {
      case 'care':
        return 'watering';
      case 'measurement':
        return 'custom';
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
        careTypeClasses[local.careType as keyof typeof careTypeClasses] || careTypeClasses.care,
        local.class
      )}
      {...rest}
    >
      <div class="care-type-icon-container">
        {local.children || (
          <>
            {(local.showIcon !== false) && (
              <CareIcon 
                type={mapToCareIconType(local.careType)} 
                size={local.size === 'lg' ? 'lg' : local.size === 'sm' ? 'sm' : 'md'}
              />
            )}
            {(local.showLabel !== false) && (
              <span class="care-type-label">
                {careTypeLabels[local.careType] || local.careType}
              </span>
            )}
          </>
        )}
      </div>
    </button>
  );
}; 
