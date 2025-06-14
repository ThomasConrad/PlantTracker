import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '@/utils/cn';

interface CareIconProps extends JSX.SvgSVGAttributes<SVGSVGElement> {
  type: 'watering' | 'fertilizing' | 'pruning' | 'repotting' | 'pest-control' | 'note' | 'custom';
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  variant?: 'default' | 'solid' | 'outline';
  class?: string;
}

export const CareIcon: Component<CareIconProps> = (props) => {
  const [local, rest] = splitProps(props, ['type', 'size', 'variant', 'class']);

  const sizeClasses = {
    xs: 'w-3 h-3',
    sm: 'w-4 h-4',
    md: 'w-5 h-5',
    lg: 'w-6 h-6',
    xl: 'w-8 h-8',
  };

  const variantClasses = {
    default: 'fill-none stroke-current',
    solid: 'fill-current stroke-none',
    outline: 'fill-none stroke-current stroke-2',
  };

  const iconPaths = {
    watering: (
      <>
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 3v17.25l-4.5-4.5L12 16l4.5 4.5V3" />
        <path stroke-linecap="round" stroke-linejoin="round" d="M8 8l8-8M16 8L8 0" />
      </>
    ),
    fertilizing: (
      <>
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
        <circle cx="12" cy="4" r="2" />
        <circle cx="12" cy="20" r="2" />
        <circle cx="20" cy="12" r="2" />
        <circle cx="4" cy="12" r="2" />
      </>
    ),
    pruning: (
      <>
        <path stroke-linecap="round" stroke-linejoin="round" d="M6.75 7.5l3 3M6.75 7.5l-1.5-1.5M6.75 7.5h7.5M17.25 16.5l-3-3M17.25 16.5l1.5 1.5M17.25 16.5H9.75" />
        <circle cx="9" cy="9" r="2" />
        <circle cx="15" cy="15" r="2" />
      </>
    ),
    repotting: (
      <>
        <path stroke-linecap="round" stroke-linejoin="round" d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M10 11.25c0 .621-.504 1.125-1.125 1.125H8.25m1.875-1.125l-.875-.875M14 11.25c0 .621.504 1.125 1.125 1.125H15.75m-1.875-1.125l.875-.875" />
        <path stroke-linecap="round" stroke-linejoin="round" d="M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z" />
      </>
    ),
    'pest-control': (
      <>
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v6m3-3H9m12 0a9 9 0 11-18 0 9 9 0 0118 0z" />
        <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5" />
      </>
    ),
    note: (
      <>
        <path stroke-linecap="round" stroke-linejoin="round" d="M16.862 4.487l1.687-1.688a1.875 1.875 0 112.652 2.652L10.582 16.07a4.5 4.5 0 01-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 011.13-1.897l8.932-8.931zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0115.75 21H5.25A2.25 2.25 0 013 18.75V8.25A2.25 2.25 0 015.25 6H10" />
      </>
    ),
    custom: (
      <>
        <path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.240.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
        <path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
      </>
    ),
  };

  return (
    <svg
      class={cn(
        sizeClasses[local.size || 'md'],
        variantClasses[local.variant || 'default'],
        local.class
      )}
      viewBox="0 0 24 24"
      {...rest}
    >
      {iconPaths[local.type]}
    </svg>
  );
};

// Convenience components for each care type
export const WateringIcon: Component<Omit<CareIconProps, 'type'>> = (props) => (
  <CareIcon type="watering" {...props} />
);

export const FertilizingIcon: Component<Omit<CareIconProps, 'type'>> = (props) => (
  <CareIcon type="fertilizing" {...props} />
);

export const PruningIcon: Component<Omit<CareIconProps, 'type'>> = (props) => (
  <CareIcon type="pruning" {...props} />
);

export const RepottingIcon: Component<Omit<CareIconProps, 'type'>> = (props) => (
  <CareIcon type="repotting" {...props} />
);

export const PestControlIcon: Component<Omit<CareIconProps, 'type'>> = (props) => (
  <CareIcon type="pest-control" {...props} />
);

export const NoteIcon: Component<Omit<CareIconProps, 'type'>> = (props) => (
  <CareIcon type="note" {...props} />
);

export const CustomIcon: Component<Omit<CareIconProps, 'type'>> = (props) => (
  <CareIcon type="custom" {...props} />
); 