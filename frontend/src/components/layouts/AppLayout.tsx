import { Component, JSX, createSignal, Show } from 'solid-js';
import { A } from '@solidjs/router';
import { authStore } from '@/stores/auth';
import { BottomNavigation } from './BottomNavigation';

interface AppLayoutProps {
  children: JSX.Element;
}

export const AppLayout: Component<AppLayoutProps> = (props) => {
  const [showUserMenu, setShowUserMenu] = createSignal(false);

  // Check if we're on the calendar page and should hide the top nav on mobile

  const handleLogout = async () => {
    await authStore.logout();
    setShowUserMenu(false);
  };

  return (
    <div class="bg-gray-50 mobile-content-container sm:min-h-screen">
      {/* Desktop Navigation */}
      <nav class={`bg-white shadow-sm border-b border-gray-200 sm:block hidden`}>
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div class="flex justify-between h-16">
            <div class="flex items-center">
              <A href="/plants" class="flex items-center space-x-2">
                <div class="logo-container-sm">
                  <svg
                    class="h-5 w-5 text-white"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width={2}
                      d="M12 6v6m0 0v6m0-6h6m-6 0H6"
                    />
                  </svg>
                </div>
                <span class="text-xl font-bold text-gray-900">Planty🪴</span>
              </A>
            </div>

            <div class="hidden sm:flex items-center space-x-8">
              {/* Navigation Links */}
              <A
                href="/plants"
                class="inline-flex items-center px-1 pt-1 text-sm font-medium text-gray-900 border-b-2 border-transparent hover:border-gray-300"
                activeClass="border-blue-500 text-blue-600"
              >
                <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                </svg>
                Plants
              </A>
              
              <A
                href="/calendar"
                class="inline-flex items-center px-1 pt-1 text-sm font-medium text-gray-900 border-b-2 border-transparent hover:border-gray-300"
                activeClass="border-blue-500 text-blue-600"
              >
                <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                Calendar
              </A>
            </div>
            
            <div class="flex items-center space-x-4">
              <A
                href="/plants/new"
                class="btn btn-primary btn-sm"
                activeClass="bg-primary-700"
              >
                Add Plant
              </A>

              <div class="relative">
                <button
                  onClick={() => setShowUserMenu(!showUserMenu())}
                  class="flex items-center space-x-2 p-2 rounded-md text-gray-700 hover:text-gray-900 hover:bg-gray-100"
                  aria-label="User menu"
                >
                  <div class="icon-container-md bg-gray-300">
                    <span class="text-sm font-medium text-gray-700">
                      {authStore.user?.name?.[0]?.toUpperCase() || 'U'}
                    </span>
                  </div>
                  <svg
                    class="h-5 w-5"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width={2}
                      d="M19 9l-7 7-7-7"
                    />
                  </svg>
                </button>

                {showUserMenu() && (
                  <div class="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg border border-gray-200 z-40">
                    <div class="py-1">
                      <div class="px-4 py-2 text-sm text-gray-700 border-b border-gray-200">
                        <p class="font-medium">{authStore.user?.name}</p>
                        <p class="text-gray-500">{authStore.user?.email}</p>
                      </div>
                      <Show when={authStore.user?.role === 'admin'}>
                        <A
                          href="/admin/dashboard"
                          class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                          onClick={() => setShowUserMenu(false)}
                        >
                          <div class="flex items-center">
                            <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                            </svg>
                            Admin Dashboard
                          </div>
                        </A>
                      </Show>
                      <Show when={authStore.user?.canCreateInvites}>
                        <A
                          href="/invites"
                          class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                          onClick={() => setShowUserMenu(false)}
                        >
                          <div class="flex items-center">
                            <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 4v16m8-8H4" />
                            </svg>
                            Manage Invites
                          </div>
                        </A>
                      </Show>
                      <A
                        href="/calendar/settings"
                        class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                        onClick={() => setShowUserMenu(false)}
                      >
                        <div class="flex items-center">
                          <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                          </svg>
                          Calendar Settings
                        </div>
                      </A>
                      <button
                        onClick={handleLogout}
                        class="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                      >
                        Sign out
                      </button>
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      </nav>

      <main class="mobile-content-area sm:max-w-7xl sm:mx-auto sm:py-6 sm:px-6 lg:px-8">
        <div class="h-full '${isMobile() ? '' : 'p-4'}'">
          {props.children}
        </div>
      </main>
      
      {/* Bottom Navigation */}
      <div class="mobile-nav-area">
        <BottomNavigation />
      </div>
    </div>
  );
};