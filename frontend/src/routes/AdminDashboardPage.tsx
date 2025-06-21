import { Component, createSignal, onMount, Show } from 'solid-js';
import { A } from '@solidjs/router';
import { apiClient } from '@/api/client';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

interface SystemStats {
  total_users: number;
  max_total_users: number;
  total_invites: number;
  active_invites: number;
  used_invites: number;
  admin_count: number;
}

interface RecentUser {
  id: string;
  email: string;
  name: string;
  role: string;
  created_at: string;
}

interface RecentInvite {
  id: string;
  code: string;
  created_by_name?: string;
  max_uses: number;
  current_uses: number;
  is_active: boolean;
  created_at: string;
}

interface AdminDashboardData {
  system_stats: SystemStats;
  recent_users: RecentUser[];
  recent_invites: RecentInvite[];
}

export const AdminDashboardPage: Component = () => {
  const [data, setData] = createSignal<AdminDashboardData | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  const loadDashboardData = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await fetch('/api/admin/dashboard', {
        credentials: 'include'
      });
      
      if (!response.ok) {
        throw new Error('Failed to load dashboard data');
      }
      
      const dashboardData = await response.json();
      setData(dashboardData);
    } catch (err) {
      console.error('Error loading dashboard:', err);
      setError(err instanceof Error ? err.message : 'Failed to load dashboard');
    } finally {
      setLoading(false);
    }
  };

  onMount(() => {
    loadDashboardData();
  });

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  return (
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div class="mb-8">
        <h1 class="text-3xl font-bold text-gray-900">Admin Dashboard</h1>
        <p class="mt-2 text-gray-600">Manage your Planty instance</p>
      </div>

      <Show 
        when={!loading() && !error() && data()} 
        fallback={
          <Show 
            when={loading()} 
            fallback={
              <div class="text-center py-12">
                <div class="text-red-600">{error()}</div>
                <button 
                  onClick={loadDashboardData}
                  class="mt-4 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                >
                  Retry
                </button>
              </div>
            }
          >
            <div class="flex justify-center py-12">
              <LoadingSpinner size="lg" />
            </div>
          </Show>
        }
      >
        {() => {
          const dashboardData = data()!;
          return (
            <div class="space-y-8">
              {/* System Statistics */}
              <div class="bg-white overflow-hidden shadow rounded-lg">
                <div class="px-4 py-5 sm:p-6">
                  <h2 class="text-lg font-medium text-gray-900 mb-4">System Statistics</h2>
                  <div class="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
                    <div class="bg-gray-50 overflow-hidden shadow rounded-lg">
                      <div class="p-5">
                        <div class="flex items-center">
                          <div class="flex-shrink-0">
                            <svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z" />
                            </svg>
                          </div>
                          <div class="ml-5 w-0 flex-1">
                            <dl>
                              <dt class="text-sm font-medium text-gray-500 truncate">Total Users</dt>
                              <dd class="text-lg font-medium text-gray-900">
                                {dashboardData.system_stats.total_users} / {dashboardData.system_stats.max_total_users}
                              </dd>
                            </dl>
                          </div>
                        </div>
                      </div>
                    </div>

                    <div class="bg-gray-50 overflow-hidden shadow rounded-lg">
                      <div class="p-5">
                        <div class="flex items-center">
                          <div class="flex-shrink-0">
                            <svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z" />
                            </svg>
                          </div>
                          <div class="ml-5 w-0 flex-1">
                            <dl>
                              <dt class="text-sm font-medium text-gray-500 truncate">Active Invites</dt>
                              <dd class="text-lg font-medium text-gray-900">
                                {dashboardData.system_stats.active_invites} / {dashboardData.system_stats.total_invites}
                              </dd>
                            </dl>
                          </div>
                        </div>
                      </div>
                    </div>

                    <div class="bg-gray-50 overflow-hidden shadow rounded-lg">
                      <div class="p-5">
                        <div class="flex items-center">
                          <div class="flex-shrink-0">
                            <svg class="h-6 w-6 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                            </svg>
                          </div>
                          <div class="ml-5 w-0 flex-1">
                            <dl>
                              <dt class="text-sm font-medium text-gray-500 truncate">Admins</dt>
                              <dd class="text-lg font-medium text-gray-900">{dashboardData.system_stats.admin_count}</dd>
                            </dl>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Quick Actions */}
              <div class="bg-white overflow-hidden shadow rounded-lg">
                <div class="px-4 py-5 sm:p-6">
                  <h2 class="text-lg font-medium text-gray-900 mb-4">Quick Actions</h2>
                  <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
                    <A 
                      href="/admin/users" 
                      class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700"
                    >
                      <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z" />
                      </svg>
                      Manage Users
                    </A>
                    <A 
                      href="/invites" 
                      class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-green-600 hover:bg-green-700"
                    >
                      <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 5v2m0 4v2m0 4v2M5 5a2 2 0 00-2 2v3a2 2 0 110 4v3a2 2 0 002 2h14a2 2 0 002-2v-3a2 2 0 110-4V7a2 2 0 00-2-2H5z" />
                      </svg>
                      Manage Invites
                    </A>
                    <A 
                      href="/admin/settings" 
                      class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-gray-600 hover:bg-gray-700"
                    >
                      <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      </svg>
                      System Settings
                    </A>
                    <A 
                      href="/admin/health" 
                      class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-purple-600 hover:bg-purple-700"
                    >
                      <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
                      </svg>
                      System Health
                    </A>
                  </div>
                </div>
              </div>

              {/* Recent Users and Invites */}
              <div class="grid grid-cols-1 gap-8 lg:grid-cols-2">
                {/* Recent Users */}
                <div class="bg-white overflow-hidden shadow rounded-lg">
                  <div class="px-4 py-5 sm:p-6">
                    <h2 class="text-lg font-medium text-gray-900 mb-4">Recent Users</h2>
                    <div class="space-y-3">
                      {dashboardData.recent_users.map((user) => (
                        <div class="flex items-center justify-between">
                          <div>
                            <p class="text-sm font-medium text-gray-900">{user.name}</p>
                            <p class="text-sm text-gray-500">{user.email}</p>
                          </div>
                          <div class="text-right">
                            <span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                              user.role === 'admin' ? 'bg-red-100 text-red-800' : 
                              user.role === 'moderator' ? 'bg-yellow-100 text-yellow-800' : 
                              'bg-gray-100 text-gray-800'
                            }`}>
                              {user.role}
                            </span>
                            <p class="text-xs text-gray-500 mt-1">{formatDate(user.created_at)}</p>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>

                {/* Recent Invites */}
                <div class="bg-white overflow-hidden shadow rounded-lg">
                  <div class="px-4 py-5 sm:p-6">
                    <h2 class="text-lg font-medium text-gray-900 mb-4">Recent Invites</h2>
                    <div class="space-y-3">
                      {dashboardData.recent_invites.map((invite) => (
                        <div class="flex items-center justify-between">
                          <div>
                            <p class="text-sm font-medium text-gray-900 font-mono">{invite.code}</p>
                            <p class="text-sm text-gray-500">by {invite.created_by_name || 'System'}</p>
                          </div>
                          <div class="text-right">
                            <span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                              invite.is_active ? 'bg-green-100 text-green-800' : 'bg-gray-100 text-gray-800'
                            }`}>
                              {invite.current_uses}/{invite.max_uses} used
                            </span>
                            <p class="text-xs text-gray-500 mt-1">{formatDate(invite.created_at)}</p>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          );
        }}
      </Show>
    </div>
  );
};