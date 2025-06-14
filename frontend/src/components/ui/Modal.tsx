import { Component, JSX, splitProps, Show, onMount, onCleanup } from 'solid-js';
import { cn } from '@/utils/cn';

interface ModalProps {
  open: boolean;
  onClose: () => void;
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  position?: 'center' | 'top' | 'bottom';
  closeOnOverlayClick?: boolean;
  closeOnEscape?: boolean;
  contentOnly?: boolean; // If true, only covers content area
  children: JSX.Element;
  class?: string;
}

export const Modal: Component<ModalProps> = (props) => {
  const [local, rest] = splitProps(props, [
    'open', 'onClose', 'size', 'position', 'closeOnOverlayClick', 
    'closeOnEscape', 'contentOnly', 'children', 'class'
  ]);

  const sizeClasses = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg',
    xl: 'max-w-xl',
    full: 'max-w-full mx-4',
  };

  const positionClasses = {
    center: 'items-center justify-center',
    top: 'items-start justify-center pt-20',
    bottom: 'items-end justify-center pb-20',
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (local.closeOnEscape !== false && e.key === 'Escape') {
      local.onClose();
    }
  };

  const handleOverlayClick = (e: MouseEvent) => {
    if (local.closeOnOverlayClick !== false && e.target === e.currentTarget) {
      local.onClose();
    }
  };

  const handleOverlayTouchMove = (e: TouchEvent) => {
    // Prevent scrolling through modal on mobile
    if (local.contentOnly) {
      e.preventDefault();
    }
  };

  onMount(() => {
    if (local.open) {
      document.body.style.overflow = 'hidden';
      document.addEventListener('keydown', handleKeyDown);
    }
  });

  onCleanup(() => {
    document.body.style.overflow = '';
    document.removeEventListener('keydown', handleKeyDown);
  });

  return (
    <Show when={local.open}>
      <div
        class={cn(
          local.contentOnly ? 'modal-overlay' : 'modal-overlay-fullscreen',
          'transition-all duration-200 ease-out',
          local.contentOnly ? '' : positionClasses[local.position || 'center']
        )}
        onClick={handleOverlayClick}
        onTouchMove={handleOverlayTouchMove}
        role="dialog"
        aria-modal="true"
      >
        <div
          class={cn(
            'relative w-full transform transition-all duration-200 ease-out',
            'bg-white dark:bg-gray-800 rounded-lg shadow-xl',
            'border border-gray-200 dark:border-gray-700',
            sizeClasses[local.size || 'md'],
            local.class
          )}
          onClick={(e) => e.stopPropagation()}
          onTouchMove={(e) => e.stopPropagation()}
          {...rest}
        >
          {local.children}
        </div>
      </div>
    </Show>
  );
};

interface ModalHeaderProps extends JSX.HTMLAttributes<HTMLDivElement> {
  onClose?: () => void;
  showCloseButton?: boolean;
}

export const ModalHeader: Component<ModalHeaderProps> = (props) => {
  const [local, rest] = splitProps(props, ['onClose', 'showCloseButton', 'children', 'class']);

  return (
    <div
      class={cn(
        'flex items-center justify-between p-6 border-b border-gray-200 dark:border-gray-700',
        local.class
      )}
      {...rest}
    >
      <div class="flex-1">{local.children}</div>
      <Show when={local.showCloseButton && local.onClose}>
        <button
          onClick={local.onClose}
          class="ml-4 p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          aria-label="Close modal"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </Show>
    </div>
  );
};

interface ModalBodyProps extends JSX.HTMLAttributes<HTMLDivElement> {}

export const ModalBody: Component<ModalBodyProps> = (props) => {
  const [local, rest] = splitProps(props, ['children', 'class']);

  return (
    <div
      class={cn('p-6', local.class)}
      {...rest}
    >
      {local.children}
    </div>
  );
};

interface ModalFooterProps extends JSX.HTMLAttributes<HTMLDivElement> {
  justify?: 'start' | 'center' | 'end' | 'between';
}

export const ModalFooter: Component<ModalFooterProps> = (props) => {
  const [local, rest] = splitProps(props, ['justify', 'children', 'class']);

  const justifyClasses = {
    start: 'justify-start',
    center: 'justify-center',
    end: 'justify-end',
    between: 'justify-between',
  };

  return (
    <div
      class={cn(
        'flex items-center p-6 border-t border-gray-200 dark:border-gray-700',
        'bg-gray-50 dark:bg-gray-900',
        justifyClasses[local.justify || 'end'],
        local.class
      )}
      {...rest}
    >
      {local.children}
    </div>
  );
}; 