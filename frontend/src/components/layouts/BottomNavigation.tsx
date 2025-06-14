import { Component, createSignal } from 'solid-js';
import { A, useLocation } from '@solidjs/router';
import { SettingsMenu } from './SettingsMenu';
import { NavIcon } from '@/components/ui/NavIcon';

export const BottomNavigation: Component = () => {
  const location = useLocation();
  const [showSettings, setShowSettings] = createSignal(false);
  
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
        <NavIcon isActive={active}>
          <path 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            stroke-width={active ? 2.5 : 2} 
            d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" 
          />
        </NavIcon>
      )
    },
    {
      path: '/calendar',
      icon: (active: boolean) => (
        <NavIcon isActive={active}>
          <path 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            stroke-width={active ? 2.5 : 2} 
            d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" 
          />
        </NavIcon>
      )
    },
    {
      path: '/search',
      icon: (active: boolean) => (
        <NavIcon isActive={active}>
          <path 
            stroke-linecap="round" 
            stroke-linejoin="round" 
            stroke-width={2} 
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" 
          />
        </NavIcon>
      )
    }
  ];

  return (
    <nav class="bottom-nav">
      <div class="bottom-nav-container">
        {navItems.map(item => {
          const active = isActive(item.path);
          return (
            <A
              href={item.path}
              class="bottom-nav-item"
              activeClass=""
            >
              {item.icon(active)}
            </A>
          );
        })}
        
        {/* Settings button */}
        <button 
          class="bottom-nav-item"
          onClick={() => setShowSettings(!showSettings())}
        >
          <NavIcon isActive={showSettings()}>
            <path 
              stroke-linecap="round" 
              stroke-linejoin="round" 
              stroke-width={2} 
              d="M4 6h16M4 12h16M4 18h16" 
            />
          </NavIcon>
        </button>
      </div>
      
      <SettingsMenu 
        isOpen={showSettings()} 
        onClose={() => setShowSettings(false)} 
      />
    </nav>
  );
};