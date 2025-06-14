// Base Components
export { Button } from './Button';
export { Input } from './Input';
export { LoadingSpinner } from './LoadingSpinner';

// Layout Components
export { Card, CardHeader, CardBody, CardFooter } from './Card';
export { Modal, ModalHeader, ModalBody, ModalFooter } from './Modal';

// Feedback Components
export { Badge } from './Badge';

// Plant Care Components
export { CareTypeButton } from './CareTypeButton';
export { 
  CareIcon, 
  WateringIcon, 
  FertilizingIcon, 
  PruningIcon, 
  RepottingIcon, 
  PestControlIcon, 
  NoteIcon, 
  CustomIcon 
} from './CareIcon';

// Navigation Components
export { NavIcon } from './NavIcon';

// Theme Components
export { ThemeToggle } from './ThemeToggle';

// Component Types
export type { 
  ButtonProps, 
  InputProps, 
  LoadingSpinnerProps, 
  CardProps, 
  ModalProps, 
  BadgeProps, 
  CareIconProps, 
  NavIconProps 
} from './types';

// Re-export for convenience
export { cn } from '@/utils/cn'; 