import { Component, JSX, createContext, useContext, createSignal, onMount, createEffect } from 'solid-js';

type Theme = 'light' | 'dark' | 'system';

interface ThemeContextValue {
  theme: () => Theme;
  setTheme: (theme: Theme) => void;
  isDark: () => boolean;
}

const ThemeContext = createContext<ThemeContextValue>();

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
};

interface ThemeProviderProps {
  children: JSX.Element;
  defaultTheme?: Theme;
  storageKey?: string;
}

export const ThemeProvider: Component<ThemeProviderProps> = (props) => {
  const [theme, setTheme] = createSignal<Theme>(props.defaultTheme || 'system');
  
  const getSystemTheme = (): 'light' | 'dark' => {
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  };

  const isDark = () => {
    const currentTheme = theme();
    if (currentTheme === 'system') {
      return getSystemTheme() === 'dark';
    }
    return currentTheme === 'dark';
  };

  const updateTheme = (newTheme: Theme) => {
    setTheme(newTheme);
    
    // Save to localStorage
    if (props.storageKey) {
      localStorage.setItem(props.storageKey, newTheme);
    }
    
    // Update document class and data attribute
    const root = document.documentElement;
    const resolvedTheme = newTheme === 'system' ? getSystemTheme() : newTheme;
    
    root.classList.remove('light', 'dark');
    root.classList.add(resolvedTheme);
    root.setAttribute('data-theme', resolvedTheme);
  };

  // Listen for system theme changes
  onMount(() => {
    // Load saved theme from localStorage
    if (props.storageKey) {
      const savedTheme = localStorage.getItem(props.storageKey) as Theme;
      if (savedTheme && ['light', 'dark', 'system'].includes(savedTheme)) {
        setTheme(savedTheme);
      }
    }

    // Listen for system theme changes
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    const handleChange = () => {
      if (theme() === 'system') {
        updateTheme('system'); // This will trigger the theme update
      }
    };

    mediaQuery.addEventListener('change', handleChange);
    
    // Initial theme application
    updateTheme(theme());

    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  });

  // Update theme when it changes
  createEffect(() => {
    updateTheme(theme());
  });

  const contextValue: ThemeContextValue = {
    theme,
    setTheme: updateTheme,
    isDark,
  };

  return (
    <ThemeContext.Provider value={contextValue}>
      {props.children}
    </ThemeContext.Provider>
  );
}; 