import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '@/utils/cn';

interface CardProps extends JSX.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'elevated' | 'outlined' | 'ghost';
  padding?: 'none' | 'sm' | 'md' | 'lg';
  hover?: boolean;
}

export const Card: Component<CardProps> = (props) => {
  const [local, rest] = splitProps(props, ['variant', 'padding', 'hover', 'children', 'class']);

  const variantClasses = {
    default: 'bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 shadow-sm',
    elevated: 'bg-white dark:bg-gray-800 shadow-md border border-gray-200 dark:border-gray-700',
    outlined: 'bg-white dark:bg-gray-800 border-2 border-gray-200 dark:border-gray-700',
    ghost: 'bg-transparent border-none shadow-none',
  };

  const paddingClasses = {
    none: 'p-0',
    sm: 'p-4',
    md: 'p-6',
    lg: 'p-8',
  };

  return (
    <div
      class={cn(
        'rounded-lg transition-all duration-200',
        variantClasses[local.variant || 'default'],
        paddingClasses[local.padding || 'md'],
        local.hover && 'hover:shadow-lg hover:scale-[1.02] cursor-pointer',
        local.class
      )}
      {...rest}
    >
      {local.children}
    </div>
  );
};

interface CardHeaderProps extends JSX.HTMLAttributes<HTMLDivElement> {
  bordered?: boolean;
}

export const CardHeader: Component<CardHeaderProps> = (props) => {
  const [local, rest] = splitProps(props, ['bordered', 'children', 'class']);

  return (
    <div
      class={cn(
        'px-6 py-4',
        local.bordered && 'border-b border-gray-200 dark:border-gray-700',
        local.class
      )}
      {...rest}
    >
      {local.children}
    </div>
  );
};

interface CardBodyProps extends JSX.HTMLAttributes<HTMLDivElement> {}

export const CardBody: Component<CardBodyProps> = (props) => {
  const [local, rest] = splitProps(props, ['children', 'class']);

  return (
    <div
      class={cn('px-6 py-4', local.class)}
      {...rest}
    >
      {local.children}
    </div>
  );
};

interface CardFooterProps extends JSX.HTMLAttributes<HTMLDivElement> {
  bordered?: boolean;
  background?: boolean;
}

export const CardFooter: Component<CardFooterProps> = (props) => {
  const [local, rest] = splitProps(props, ['bordered', 'background', 'children', 'class']);

  return (
    <div
      class={cn(
        'px-6 py-4',
        local.bordered && 'border-t border-gray-200 dark:border-gray-700',
        local.background && 'bg-gray-50 dark:bg-gray-900',
        local.class
      )}
      {...rest}
    >
      {local.children}
    </div>
  );
}; 