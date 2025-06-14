# 🎨 PlantTracker Design System

## Overview

PlantTracker uses a comprehensive design system built with **design tokens**, **component variants**, and **dark mode** support. This guide helps maintain consistency across the application.

## 🎯 Design Tokens

Design tokens are stored in `frontend/src/styles/design-tokens.css` and provide centralized values for:

### Colors
- **Primary Colors**: Green-based palette for plant care actions
- **Semantic Colors**: Success, warning, error, info states
- **Care Type Colors**: Specific colors for each care activity
- **Gray Scale**: Comprehensive neutral palette

### Care Type Color Mapping
```css
--color-watering-*: Blue palette (💧)
--color-fertilizing-*: Green palette (🌱)
--color-pruning-*: Purple palette (✂️)
--color-repotting-*: Orange palette (🪴)
--color-pest-control-*: Red palette (🐛)
```

### Spacing & Typography
```css
--space-*: Consistent spacing scale (0-24)
--font-size-*: Type scale from xs to 4xl
--font-weight-*: Weight scale (light to bold)
```

## 🌙 Dark Mode

### Implementation
- Uses CSS custom properties with `[data-theme="dark"]` selector
- Automatic system preference detection
- Persistent user preference storage
- Smooth transitions between themes

### Theme Provider Usage
```tsx
import { ThemeProvider, useTheme } from '@/providers/ThemeProvider';

// Wrap your app
<ThemeProvider defaultTheme="system" storageKey="planty-theme">
  <App />
</ThemeProvider>

// Use in components
const { theme, setTheme, isDark } = useTheme();
```

### Theme Toggle Component
```tsx
import { ThemeToggle } from '@/components/ui/ThemeToggle';

// Simple toggle button
<ThemeToggle />

// Dropdown with all options
<ThemeToggle variant="dropdown" />
```

## 🧩 Component Architecture

### Base Pattern
All components follow this consistent pattern:

```tsx
import { Component, JSX, splitProps } from 'solid-js';
import { cn } from '@/utils/cn';

interface ComponentProps extends JSX.HTMLAttributes<HTMLElement> {
  variant?: 'default' | 'primary' | 'secondary';
  size?: 'sm' | 'md' | 'lg';
  // Component-specific props
}

export const Component: Component<ComponentProps> = (props) => {
  const [local, rest] = splitProps(props, ['variant', 'size', 'class', 'children']);

  const variantClasses = {
    default: 'bg-white text-gray-900',
    primary: 'bg-primary-600 text-white',
    secondary: 'bg-gray-100 text-gray-900',
  };

  const sizeClasses = {
    sm: 'px-2 py-1 text-sm',
    md: 'px-3 py-2 text-base',
    lg: 'px-4 py-3 text-lg',
  };

  return (
    <element
      class={cn(
        'base-classes',
        variantClasses[local.variant || 'default'],
        sizeClasses[local.size || 'md'],
        local.class
      )}
      {...rest}
    >
      {local.children}
    </element>
  );
};
```

## 📚 Component Library

### Core Components

#### Button
```tsx
import { Button } from '@/components/ui';

<Button variant="primary" size="md" loading={false}>
  Click me
</Button>
```

**Variants**: `primary`, `secondary`, `outline`, `danger`
**Sizes**: `sm`, `md`, `lg`

#### Card
```tsx
import { Card, CardHeader, CardBody, CardFooter } from '@/components/ui';

<Card variant="elevated" hover>
  <CardHeader bordered>
    <h3>Card Title</h3>
  </CardHeader>
  <CardBody>
    <p>Card content...</p>
  </CardBody>
  <CardFooter background>
    <Button>Action</Button>
  </CardFooter>
</Card>
```

**Variants**: `default`, `elevated`, `outlined`, `ghost`

#### Modal
```tsx
import { Modal, ModalHeader, ModalBody, ModalFooter } from '@/components/ui';

<Modal 
  open={isOpen} 
  onClose={() => setIsOpen(false)}
  size="md"
  position="center"
>
  <ModalHeader showCloseButton onClose={() => setIsOpen(false)}>
    <h2>Modal Title</h2>
  </ModalHeader>
  <ModalBody>
    <p>Modal content...</p>
  </ModalBody>
  <ModalFooter justify="end">
    <Button onClick={() => setIsOpen(false)}>Close</Button>
  </ModalFooter>
</Modal>
```

### Plant Care Components

#### CareTypeButton
```tsx
import { CareTypeButton } from '@/components/ui';

<CareTypeButton
  type="watering"
  isActive={selectedType === 'watering'}
  onClick={() => setSelectedType('watering')}
  size="md"
  variant="default"
/>
```

#### CareIcon
```tsx
import { CareIcon, WateringIcon } from '@/components/ui';

<CareIcon type="watering" size="md" />
<WateringIcon size="lg" class="text-blue-600" />
```

**Types**: `watering`, `fertilizing`, `pruning`, `repotting`, `pest-control`, `note`, `custom`

#### Badge
```tsx
import { Badge } from '@/components/ui';

<Badge careType="watering" size="md" pill>
  Watering
</Badge>

<Badge variant="success">
  Completed
</Badge>
```

## 🎨 Styling Guidelines

### CSS Class Naming
- Use existing Tailwind classes with dark mode variants
- Prefer design token-based custom properties
- Component-specific classes follow BEM-like patterns

### Dark Mode Classes
```tsx
// Always include dark mode variants
'bg-white dark:bg-gray-800'
'text-gray-900 dark:text-gray-100'
'border-gray-200 dark:border-gray-700'
```

### Responsive Design
```tsx
// Mobile-first approach
'p-4 sm:p-6 lg:p-8'
'text-sm sm:text-base lg:text-lg'
```

## 🔧 Development Workflow

### Adding New Components

1. **Create Component File**
   ```tsx
   // /components/ui/NewComponent.tsx
   export const NewComponent: Component<Props> = (props) => {
     // Follow base pattern
   };
   ```

2. **Add to Index**
   ```tsx
   // /components/ui/index.ts
   export { NewComponent } from './NewComponent';
   ```

3. **Add Types**
   ```tsx
   // /components/ui/types.ts
   export interface NewComponentProps {
     // Define props
   }
   ```

### Extending Care Types

1. **Add to Design Tokens**
   ```css
   --color-new-care-50: #color;
   --color-new-care-100: #color;
   /* ... */
   ```

2. **Update CareIcon**
   ```tsx
   // Add icon path to CareIcon component
   'new-care': (
     <>
       <path d="..." />
     </>
   )
   ```

3. **Update CareTypeButton**
   ```tsx
   // Add color mapping
   'new-care': 'bg-newcolor-100 text-newcolor-800'
   ```

## 📱 Mobile-First Patterns

### Responsive Components
```tsx
// Use existing responsive utility classes
'responsive-container'     // Full height on mobile, auto on desktop
'responsive-header'        // Consistent header padding
'responsive-content'       // Appropriate content spacing
'mobile-button'           // Touch-friendly button sizing
```

### Bottom Navigation
```tsx
'bottom-nav'              // Mobile navigation container
'bottom-nav-item'         // Navigation item
'bottom-nav-icon-active'  // Active state icon
```

## 🎯 Best Practices

### Component Development
- ✅ Always use `splitProps` for prop separation
- ✅ Include dark mode variants in all color classes
- ✅ Use design tokens for consistent spacing/sizing
- ✅ Follow the established prop interface patterns
- ✅ Include TypeScript types for all props

### Styling
- ✅ Use `cn()` utility for conditional classes
- ✅ Prefer design tokens over hardcoded values
- ✅ Include hover, focus, and active states
- ✅ Test in both light and dark modes
- ✅ Ensure touch-friendly sizing on mobile

### Accessibility
- ✅ Include proper ARIA attributes
- ✅ Ensure keyboard navigation works
- ✅ Use semantic HTML elements
- ✅ Include focus indicators
- ✅ Test with screen readers

## 🚀 Quick Start Checklist

When creating new components:

- [ ] Follow the base component pattern
- [ ] Include variant and size props
- [ ] Add dark mode support
- [ ] Use design tokens
- [ ] Add to component index
- [ ] Include TypeScript types
- [ ] Test responsive behavior
- [ ] Verify accessibility
- [ ] Document usage examples

## 🔄 Component Updates

To maintain consistency when updating existing components:

1. **Check Design Tokens**: Use centralized values
2. **Dark Mode**: Ensure all states work in dark mode
3. **Mobile**: Test responsive behavior
4. **Types**: Update TypeScript interfaces
5. **Documentation**: Update usage examples

---

This design system ensures **consistency**, **maintainability**, and **developer experience** across the PlantTracker application. All new components should follow these established patterns. 