import { Component, Show } from 'solid-js';
import { A } from '@solidjs/router';

interface SettingsMenuProps {
  isOpen: boolean;
  onClose: () => void;
}

export const SettingsMenu: Component<SettingsMenuProps> = (props) => {
  const menuItems = [
    {
      label: 'Calendar Settings',
      href: '/calendar/settings',
      icon: (
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
      )
    },
    {
      label: 'User Settings',
      href: '/settings',
      icon: (
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
        </svg>
      )
    },
    {
      label: 'Logout',
      href: '/logout',
      icon: (
        <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
        </svg>
      )
    }
  ];

  return (
    <Show when={props.isOpen}>
      {/* Backdrop */}
      <div 
        class="fixed inset-0 bg-black bg-opacity-25 z-40 sm:hidden"
        onClick={props.onClose}
      />
      
      {/* Settings Menu */}
      <div class="fixed bottom-16 right-4 bg-white rounded-lg shadow-lg border border-gray-200 z-50 sm:hidden min-w-[200px]">
        <div class="py-2">
          {menuItems.map((item) => (
            <A
              href={item.href}
              class="flex items-center gap-3 px-4 py-3 text-gray-700 hover:bg-gray-50 transition-colors"
              onClick={props.onClose}
            >
              <span class="text-gray-400">{item.icon}</span>
              <span class="text-sm font-medium">{item.label}</span>
            </A>
          ))}
        </div>
      </div>
    </Show>
  );
};