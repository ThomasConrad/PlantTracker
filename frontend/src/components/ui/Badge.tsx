import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '@/utils/cn';

interface BadgeProps extends JSX.HTMLAttributes<HTMLSpanElement> {
  variant?: 'default' | 'success' | 'warning' | 'error' | 'info';
  careType?: 'watering' | 'fertilizing' | 'pruning' | 'repotting' | 'pest-control';
  size?: 'sm' | 'md' | 'lg';
  pill?: boolean;
}

export const Badge: Component<BadgeProps> = (props) => {
  const [local, rest] = splitProps(props, ['variant', 'careType', 'size', 'pill', 'children', 'class']);

  const variantClasses = {
    default: 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-100',
    success: 'bg-green-100 text-green-800 dark:bg-green-800 dark:text-green-100',
    warning: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-800 dark:text-yellow-100',
    error: 'bg-red-100 text-red-800 dark:bg-red-800 dark:text-red-100',
    info: 'bg-blue-100 text-blue-800 dark:bg-blue-800 dark:text-blue-100',
  };

  const careTypeClasses = {
    watering: 'bg-blue-100 text-blue-800 dark:bg-blue-800 dark:text-blue-100',
    fertilizing: 'bg-green-100 text-green-800 dark:bg-green-800 dark:text-green-100',
    pruning: 'bg-purple-100 text-purple-800 dark:bg-purple-800 dark:text-purple-100',
    repotting: 'bg-orange-100 text-orange-800 dark:bg-orange-800 dark:text-orange-100',
    'pest-control': 'bg-red-100 text-red-800 dark:bg-red-800 dark:text-red-100',
  };

  const sizeClasses = {
    sm: 'px-2 py-0.5 text-xs',
    md: 'px-2.5 py-0.5 text-xs',
    lg: 'px-3 py-1 text-sm',
  };

  const getColorClass = () => {
    if (local.careType) return careTypeClasses[local.careType];
    return variantClasses[local.variant || 'default'];
  };

  return (
    <span
      class={cn(
        'inline-flex items-center font-medium',
        local.pill ? 'rounded-full' : 'rounded',
        sizeClasses[local.size || 'md'],
        getColorClass(),
        local.class
      )}
      {...rest}
    >
      {local.children}
    </span>
  );
}; 