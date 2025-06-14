import { Component, JSX } from 'solid-js';
import type { components } from '@/types/api-generated';

type EntryType = components['schemas']['EntryType'];

interface CareTypeButtonProps {
  type: EntryType;
  isActive: boolean;
  onClick: () => void;
  children: JSX.Element;
}

export const CareTypeButton: Component<CareTypeButtonProps> = (props) => {
  const getButtonClass = () => {
    if (props.isActive) {
      return props.type === 'watering' ? 'care-type-button-watering' : 'care-type-button-fertilizing';
    }
    return 'care-type-button-inactive';
  };

  return (
    <button
      type="button"
      onClick={props.onClick}
      class={getButtonClass()}
    >
      <div class="care-type-icon-container">
        {props.children}
      </div>
    </button>
  );
}; 