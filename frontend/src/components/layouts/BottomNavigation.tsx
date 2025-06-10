import { Component } from 'solid-js';
import { A, useLocation } from '@solidjs/router';

export const BottomNavigation: Component = () => {
  const location = useLocation();
  
  const isActive = (path: string) => {
    if (path === '/plants' && (location.pathname === '/plants' || location.pathname === '/')) {
      return true;
    }
    return location.pathname.startsWith(path);
  };

  const navItems = [
    {
      path: '/plants',
      label: 'Plants',
      icon: (active: boolean) => (
        <svg 
          class={`h-6 w-6 ${active ? 'text-primary-600' : 'text-gray-400'}`} 
          fill={active ? 'currentColor' : 'none'} 
          viewBox="0 0 24 24" 
          stroke="currentColor"
        >
          <path 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            stroke-width={active ? 0 : 2} 
            d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" 
          />
        </svg>
      )
    },
    {
      path: '/calendar',
      label: 'Calendar',
      icon: (active: boolean) => (
        <svg 
          class={`h-6 w-6 ${active ? 'text-primary-600' : 'text-gray-400'}`} 
          fill={active ? 'currentColor' : 'none'} 
          viewBox="0 0 24 24" 
          stroke="currentColor"
        >
          <path 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            stroke-width={active ? 0 : 2} 
            d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" 
          />
        </svg>
      )
    },
    {
      path: '/search',
      label: 'Search',
      icon: (active: boolean) => (
        <svg 
          class={`h-6 w-6 ${active ? 'text-primary-600' : 'text-gray-400'}`} 
          fill="none" 
          viewBox="0 0 24 24" 
          stroke="currentColor"
        >
          <path 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            stroke-width={2} 
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" 
          />
        </svg>
      )
    }
  ];

  return (
    <nav class="fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 z-50 sm:hidden">
      <div class="grid grid-cols-3 h-16">
        {navItems.map(item => {
          const active = isActive(item.path);
          return (
            <A
              href={item.path}
              class={`flex flex-col items-center justify-center space-y-1 transition-colors duration-200 ${
                active ? 'bg-primary-50' : 'hover:bg-gray-50'
              }`}
              activeClass=""
            >
              {item.icon(active)}
              <span class={`text-xs font-medium ${active ? 'text-primary-600' : 'text-gray-400'}`}>
                {item.label}
              </span>
            </A>
          );
        })}
      </div>
    </nav>
  );
};