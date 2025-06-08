import { Component, JSX, createSignal } from 'solid-js';
import { A } from '@solidjs/router';
import { authStore } from '@/stores/auth';

interface AppLayoutProps {
  children: JSX.Element;
}

export const AppLayout: Component<AppLayoutProps> = (props) => {
  const [showUserMenu, setShowUserMenu] = createSignal(false);

  const handleLogout = async () => {
    await authStore.logout();
    setShowUserMenu(false);
  };

  return (
    <div class="min-h-screen bg-gray-50">
      <nav class="bg-white shadow-sm border-b border-gray-200">
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div class="flex justify-between h-16">
            <div class="flex items-center">
              <A href="/plants" class="flex items-center space-x-2">
                <div class="h-8 w-8 bg-primary-600 rounded-lg flex items-center justify-center">
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
                <span class="text-xl font-bold text-gray-900">Plant Tracker</span>
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
                  <div class="h-8 w-8 bg-gray-300 rounded-full flex items-center justify-center">
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
                  <div class="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg border border-gray-200 z-50">
                    <div class="py-1">
                      <div class="px-4 py-2 text-sm text-gray-700 border-b border-gray-200">
                        <p class="font-medium">{authStore.user?.name}</p>
                        <p class="text-gray-500">{authStore.user?.email}</p>
                      </div>
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

      <main class="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
        {props.children}
      </main>
    </div>
  );
};