import { Component, JSX } from 'solid-js';

interface NavIconProps {
  isActive: boolean;
  children: JSX.Element;
  strokeWidth?: number;
}

export const NavIcon: Component<NavIconProps> = (props) => {
  return (
    <svg 
      class={props.isActive ? 'bottom-nav-icon-active' : 'bottom-nav-icon-inactive'} 
      fill="none" 
      viewBox="0 0 24 24" 
      stroke="currentColor"
    >
      {props.children}
    </svg>
  );
}; 