import { Component, createSignal, Show } from 'solid-js';
import { cn } from '@/utils/cn';
import { useTheme } from '@/providers/ThemeProvider';

interface ThemeToggleProps {
  size?: 'sm' | 'md' | 'lg';
  variant?: 'button' | 'dropdown';
  class?: string;
}

export const ThemeToggle: Component<ThemeToggleProps> = (props) => {
  const { theme, setTheme, isDark } = useTheme();
  const [showDropdown, setShowDropdown] = createSignal(false);

  const sizeClasses = {
    sm: 'w-8 h-8',
    md: 'w-10 h-10',
    lg: 'w-12 h-12',
  };

  const handleThemeToggle = () => {
    if (props.variant === 'dropdown') {
      setShowDropdown(!showDropdown());
      return;
    }

    // Simple toggle behavior for button variant
    const currentTheme = theme();
    if (currentTheme === 'light') {
      setTheme('dark');
    } else if (currentTheme === 'dark') {
      setTheme('system');
    } else {
      setTheme('light');
    }
  };

  const selectTheme = (newTheme: 'light' | 'dark' | 'system') => {
    setTheme(newTheme);
    setShowDropdown(false);
  };

  const getIcon = () => {
    const currentTheme = theme();
    const size = props.size || 'md';
    const iconSize = size === 'sm' ? 'w-4 h-4' : size === 'lg' ? 'w-6 h-6' : 'w-5 h-5';

    if (currentTheme === 'system') {
      return (
        <svg class={iconSize} fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
        </svg>
      );
    }

    if (isDark()) {
      return (
        <svg class={iconSize} fill="currentColor" viewBox="0 0 24 24">
          <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
        </svg>
      );
    }

    return (
      <svg class={iconSize} fill="currentColor" viewBox="0 0 24 24">
        <path d="M12 2.25a.75.75 0 01.75.75v2.25a.75.75 0 01-1.5 0V3a.75.75 0 01.75-.75zM7.5 12a4.5 4.5 0 119 0 4.5 4.5 0 01-9 0zM18.894 6.166a.75.75 0 00-1.06-1.06l-1.591 1.59a.75.75 0 101.06 1.061l1.591-1.59zM21.75 12a.75.75 0 01-.75.75h-2.25a.75.75 0 010-1.5H21a.75.75 0 01.75.75zM17.834 18.894a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 10-1.061 1.06l1.59 1.591zM12 18a.75.75 0 01.75.75V21a.75.75 0 01-1.5 0v-2.25A.75.75 0 0112 18zM7.758 17.303a.75.75 0 00-1.061-1.06l-1.591 1.59a.75.75 0 001.06 1.061l1.591-1.59zM6 12a.75.75 0 01-.75.75H3a.75.75 0 010-1.5h2.25A.75.75 0 016 12zM6.697 7.757a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 00-1.061 1.06l1.59 1.591z" />
      </svg>
    );
  };

  const getThemeLabel = (themeType: 'light' | 'dark' | 'system') => {
    switch (themeType) {
      case 'light': return 'Light';
      case 'dark': return 'Dark';
      case 'system': return 'System';
    }
  };

  return (
    <div class="relative">
      <button
        onClick={handleThemeToggle}
        class={cn(
          'flex items-center justify-center rounded-lg border border-gray-200 dark:border-gray-700',
          'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300',
          'hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors',
          'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500',
          sizeClasses[props.size || 'md'],
          props.class
        )}
        title={`Current theme: ${getThemeLabel(theme())}`}
      >
        {getIcon()}
      </button>

      <Show when={props.variant === 'dropdown' && showDropdown()}>
        <div class="absolute right-0 mt-2 py-2 w-36 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 z-50">
          <button
            onClick={() => selectTheme('light')}
            class={cn(
              'w-full px-4 py-2 text-left text-sm flex items-center space-x-2',
              'hover:bg-gray-100 dark:hover:bg-gray-700',
              theme() === 'light' ? 'text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900' : 'text-gray-700 dark:text-gray-300'
            )}
          >
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M12 2.25a.75.75 0 01.75.75v2.25a.75.75 0 01-1.5 0V3a.75.75 0 01.75-.75zM7.5 12a4.5 4.5 0 119 0 4.5 4.5 0 01-9 0zM18.894 6.166a.75.75 0 00-1.06-1.06l-1.591 1.59a.75.75 0 101.06 1.061l1.591-1.59zM21.75 12a.75.75 0 01-.75.75h-2.25a.75.75 0 010-1.5H21a.75.75 0 01.75.75zM17.834 18.894a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 10-1.061 1.06l1.59 1.591zM12 18a.75.75 0 01.75.75V21a.75.75 0 01-1.5 0v-2.25A.75.75 0 0112 18zM7.758 17.303a.75.75 0 00-1.061-1.06l-1.591 1.59a.75.75 0 001.06 1.061l1.591-1.59zM6 12a.75.75 0 01-.75.75H3a.75.75 0 010-1.5h2.25A.75.75 0 016 12zM6.697 7.757a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 00-1.061 1.06l1.59 1.591z" />
            </svg>
            <span>Light</span>
          </button>
          
          <button
            onClick={() => selectTheme('dark')}
            class={cn(
              'w-full px-4 py-2 text-left text-sm flex items-center space-x-2',
              'hover:bg-gray-100 dark:hover:bg-gray-700',
              theme() === 'dark' ? 'text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900' : 'text-gray-700 dark:text-gray-300'
            )}
          >
            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 24 24">
              <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" />
            </svg>
            <span>Dark</span>
          </button>
          
          <button
            onClick={() => selectTheme('system')}
            class={cn(
              'w-full px-4 py-2 text-left text-sm flex items-center space-x-2',
              'hover:bg-gray-100 dark:hover:bg-gray-700',
              theme() === 'system' ? 'text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-900' : 'text-gray-700 dark:text-gray-300'
            )}
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
            </svg>
            <span>System</span>
          </button>
        </div>
      </Show>
    </div>
  );
}; 