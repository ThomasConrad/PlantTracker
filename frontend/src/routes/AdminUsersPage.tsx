import { Component, createSignal, onMount, Show, For } from 'solid-js';
import { A } from '@solidjs/router';

interface User {
  id: string;
  email: string;
  name: string;
  role: string;
  can_create_invites: boolean;
  max_invites: number | null;
  invites_created: number;
  invites_remaining: number | null;
  created_at: string;
  updated_at: string;
}

interface UserListResponse {
  users: User[];
  total: number;
  page: number;
  limit: number;
  total_pages: number;
}

export const AdminUsersPage: Component = () => {
  const [data, setData] = createSignal<UserListResponse | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [currentPage, setCurrentPage] = createSignal(1);
  const [roleFilter, setRoleFilter] = createSignal('');
  const [editingUser, setEditingUser] = createSignal<User | null>(null);

  const loadUsers = async (page = 1, role = '') => {
    try {
      setLoading(true);
      setError(null);
      
      const params = new URLSearchParams();
      params.set('page', page.toString());
      params.set('limit', '20');
      if (role) params.set('role', role);
      
      const response = await fetch(`/api/v1/admin/users?${params}`, {
        credentials: 'include'
      });
      
      if (!response.ok) {
        throw new Error('Failed to load users');
      }
      
      const userData = await response.json();
      setData(userData);
      setCurrentPage(page);
    } catch (err) {
      console.error('Error loading users:', err);
      setError(err instanceof Error ? err.message : 'Failed to load users');
    } finally {
      setLoading(false);
    }
  };

  const handleRoleFilterChange = (newRole: string) => {
    setRoleFilter(newRole);
    loadUsers(1, newRole);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
  };

  const getRoleBadgeClass = (role: string) => {
    switch (role.toLowerCase()) {
      case 'admin':
        return 'bg-red-100 text-red-800';
      case 'moderator':
        return 'bg-yellow-100 text-yellow-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  onMount(() => {
    loadUsers();
  });

  return (
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div class="mb-8">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-3xl font-bold text-gray-900">User Management</h1>
            <p class="mt-2 text-gray-600">Manage user accounts, roles, and permissions</p>
          </div>
          <A
            href="/admin/dashboard"
            class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
          >
            ← Back to Dashboard
          </A>
        </div>
      </div>

      {/* Filters */}
      <div class="mb-6">
        <div class="bg-white shadow rounded-lg p-4">
          <div class="flex items-center space-x-4">
            <div>
              <label for="role-filter" class="block text-sm font-medium text-gray-700 mb-1">
                Filter by Role
              </label>
              <select
                id="role-filter"
                value={roleFilter()}
                onChange={(e) => handleRoleFilterChange(e.currentTarget.value)}
                class="border border-gray-300 rounded-md px-3 py-2 text-sm focus:ring-green-500 focus:border-green-500"
              >
                <option value="">All Roles</option>
                <option value="admin">Admin</option>
                <option value="moderator">Moderator</option>
                <option value="user">User</option>
              </select>
            </div>
            <div class="flex-1"></div>
            <button
              onClick={() => loadUsers(currentPage(), roleFilter())}
              class="inline-flex items-center px-3 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
            >
              <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Refresh
            </button>
          </div>
        </div>
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
                  onClick={() => loadUsers()}
                  class="mt-4 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
                >
                  Retry
                </button>
              </div>
            }
          >
            <div class="flex justify-center py-12">
              <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-green-600"></div>
            </div>
          </Show>
        }
      >
        <div class="bg-white shadow overflow-hidden sm:rounded-md">
          <div class="px-4 py-5 sm:p-6">
            <div class="mb-4 flex items-center justify-between">
              <h2 class="text-lg font-medium text-gray-900">
                Users ({data()?.total || 0})
              </h2>
              <div class="text-sm text-gray-500">
                Page {data()?.page || 1} of {data()?.total_pages || 1}
              </div>
            </div>

            <div class="overflow-x-auto">
              <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                  <tr>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      User
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Role
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Invite Permissions
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Joined
                    </th>
                    <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody class="bg-white divide-y divide-gray-200">
                  <For each={data()?.users || []}>
                    {(user) => (
                      <tr class="hover:bg-gray-50">
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div>
                            <div class="text-sm font-medium text-gray-900">{user.name}</div>
                            <div class="text-sm text-gray-500">{user.email}</div>
                          </div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <span class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getRoleBadgeClass(user.role)}`}>
                            {user.role}
                          </span>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <Show
                            when={user.can_create_invites}
                            fallback={<span class="text-gray-400">None</span>}
                          >
                            <div>
                              <div>Can create invites</div>
                              <div class="text-xs text-gray-500">
                                {user.invites_remaining !== null 
                                  ? `${user.invites_remaining} remaining` 
                                  : 'Unlimited'
                                }
                              </div>
                            </div>
                          </Show>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {formatDate(user.created_at)}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                          <button
                            onClick={() => setEditingUser(user)}
                            class="text-green-600 hover:text-green-900 mr-3"
                          >
                            Edit
                          </button>
                          <button class="text-red-600 hover:text-red-900">
                            Delete
                          </button>
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>

            {/* Pagination */}
            <Show when={(data()?.total_pages || 0) > 1}>
              <div class="mt-6 flex items-center justify-between">
                <button
                  onClick={() => loadUsers(Math.max(1, currentPage() - 1), roleFilter())}
                  disabled={currentPage() <= 1}
                  class="relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Previous
                </button>
                <span class="text-sm text-gray-700">
                  Page {currentPage()} of {data()?.total_pages || 1}
                </span>
                <button
                  onClick={() => loadUsers(Math.min(data()?.total_pages || 1, currentPage() + 1), roleFilter())}
                  disabled={currentPage() >= (data()?.total_pages || 1)}
                  class="relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  Next
                </button>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* Edit User Modal - Placeholder for now */}
      <Show when={editingUser()}>
        <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
          <div class="relative top-20 mx-auto p-5 border w-96 shadow-lg rounded-md bg-white">
            <div class="mt-3">
              <h3 class="text-lg font-medium text-gray-900 mb-4">
                Edit User: {editingUser()?.name}
              </h3>
              <p class="text-sm text-gray-600 mb-4">
                User editing functionality will be implemented here.
              </p>
              <button
                onClick={() => setEditingUser(null)}
                class="w-full px-4 py-2 bg-gray-600 text-white rounded-md hover:bg-gray-700"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};