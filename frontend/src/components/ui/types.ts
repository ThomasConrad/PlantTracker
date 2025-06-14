import { JSX } from 'solid-js';

// Button Component Types
export interface ButtonProps extends JSX.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'primary' | 'secondary' | 'outline' | 'danger';
  size?: 'sm' | 'md' | 'lg';
  loading?: boolean;
}

// Input Component Types
export interface InputProps extends JSX.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
}

// Loading Spinner Types
export interface LoadingSpinnerProps {
  size?: 'sm' | 'md' | 'lg';
  class?: string;
}

// Card Component Types
export interface CardProps extends JSX.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'elevated' | 'outlined' | 'ghost';
  padding?: 'none' | 'sm' | 'md' | 'lg';
  hover?: boolean;
}

// Modal Component Types
export interface ModalProps {
  open: boolean;
  onClose: () => void;
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  position?: 'center' | 'top' | 'bottom';
  closeOnOverlayClick?: boolean;
  closeOnEscape?: boolean;
  children: JSX.Element;
  class?: string;
}

// Badge Component Types
export interface BadgeProps extends JSX.HTMLAttributes<HTMLSpanElement> {
  variant?: 'default' | 'success' | 'warning' | 'error' | 'info';
  careType?: 'watering' | 'fertilizing' | 'pruning' | 'repotting' | 'pest-control';
  size?: 'sm' | 'md' | 'lg';
  pill?: boolean;
}

// Care Icon Types
export interface CareIconProps extends JSX.SvgSVGAttributes<SVGSVGElement> {
  type: 'watering' | 'fertilizing' | 'pruning' | 'repotting' | 'pest-control' | 'note' | 'custom';
  size?: 'xs' | 'sm' | 'md' | 'lg' | 'xl';
  variant?: 'default' | 'solid' | 'outline';
  class?: string;
}

// Navigation Icon Types
export interface NavIconProps {
  isActive: boolean;
  children: JSX.Element;
  strokeWidth?: number;
}

// Theme Types
export type Theme = 'light' | 'dark' | 'system';

export interface ThemeContextValue {
  theme: () => Theme;
  setTheme: (theme: Theme) => void;
  isDark: () => boolean;
}

// Common Layout Types
export interface LayoutProps {
  children: JSX.Element;
}

// Care Type Union
export type CareType = 'watering' | 'fertilizing' | 'pruning' | 'repotting' | 'pest-control' | 'note' | 'custom';

// Size Union
export type Size = 'xs' | 'sm' | 'md' | 'lg' | 'xl';

// Variant Union
export type Variant = 'default' | 'primary' | 'secondary' | 'outline' | 'danger' | 'success' | 'warning' | 'error' | 'info'; 