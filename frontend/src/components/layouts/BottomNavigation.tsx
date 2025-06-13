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
      icon: (active: boolean) => (
        <svg 
          class={`h-7 w-7 ${active ? 'text-primary-600' : 'text-gray-400'}`} 
          fill={active ? 'currentColor' : 'none'} 
          viewBox="0 0 24 24" 
          stroke="currentColor"
        >
          <path 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            stroke-width={active ? 0 : 2} 
            d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" 
          />
        </svg>
      )
    },
    {
      path: '/calendar',
      icon: (active: boolean) => (
        <svg 
          class={`h-7 w-7 ${active ? 'text-primary-600' : 'text-gray-400'}`} 
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
      icon: (active: boolean) => (
        <svg 
          class={`h-7 w-7 ${active ? 'text-primary-600' : 'text-gray-400'}`} 
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
    <nav class="bg-white border-t border-gray-200 sm:hidden safe-area-bottom">
      <div class="grid grid-cols-4 h-16">
        {navItems.map(item => {
          const active = isActive(item.path);
          return (
            <A
              href={item.path}
              class={`flex items-center justify-center transition-colors duration-200 ${
                active ? 'bg-primary-50' : 'hover:bg-gray-50'
              }`}
              activeClass=""
            >
              {item.icon(active)}
            </A>
          );
        })}
        
        {/* Settings button placeholder - will be implemented in next step */}
        <div class="flex items-center justify-center">
          <svg 
            class="h-7 w-7 text-gray-400" 
            fill="none" 
            viewBox="0 0 24 24" 
            stroke="currentColor"
          >
            <path 
              stroke-linecap="round" 
              stroke-linejoin="round" 
              stroke-width={2} 
              d="M4 6h16M4 12h16M4 18h16" 
            />
          </svg>
        </div>
      </div>
    </nav>
  );
};