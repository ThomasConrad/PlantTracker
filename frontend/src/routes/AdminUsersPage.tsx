import { Component, createSignal, onMount, Show, For } from 'solid-js';
import { A } from '@solidjs/router';
import { authStore } from '@/stores/auth';

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

type Role = 'admin' | 'moderator' | 'user';

export const AdminUsersPage: Component = () => {
  const [data, setData] = createSignal<UserListResponse | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [currentPage, setCurrentPage] = createSignal(1);
  const [roleFilter, setRoleFilter] = createSignal('');

  const [editingUser, setEditingUser] = createSignal<User | null>(null);
  const [editingRole, setEditingRole] = createSignal<Role>('user');
  const [editingCanCreateInvites, setEditingCanCreateInvites] = createSignal(false);
  const [editingMaxInvitesMode, setEditingMaxInvitesMode] = createSignal<'unlimited' | 'limited'>('limited');
  const [editingMaxInvites, setEditingMaxInvites] = createSignal('5');
  const [savingUser, setSavingUser] = createSignal(false);
  const [deletingUserId, setDeletingUserId] = createSignal<string | null>(null);
  const [selectedUserIds, setSelectedUserIds] = createSignal<string[]>([]);
  const [bulkRole, setBulkRole] = createSignal<Role>('user');
  const [bulkWorking, setBulkWorking] = createSignal(false);

  const loadUsers = async (page = 1, role = '') => {
    try {
      setLoading(true);
      setError(null);

      const params = new URLSearchParams();
      params.set('page', page.toString());
      params.set('limit', '20');
      if (role) params.set('role', role);

      const response = await fetch(`/api/v1/admin/users?${params}`, {
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to load users');
      }

      const userData = await response.json();
      setData(userData);
      setCurrentPage(page);
      setSelectedUserIds([]);
    } catch (err) {
      console.error('Error loading users:', err);
      setError(err instanceof Error ? err.message : 'Failed to load users');
    } finally {
      setLoading(false);
    }
  };

  const isCurrentUser = (userId: string) => authStore.user?.id === userId;

  const toggleUserSelection = (userId: string, checked: boolean) => {
    setSelectedUserIds((prev) => {
      if (checked) {
        if (prev.includes(userId)) return prev;
        return [...prev, userId];
      }
      return prev.filter((id) => id !== userId);
    });
  };

  const toggleSelectAllVisible = (checked: boolean) => {
    const visibleIds = (data()?.users || [])
      .map((u) => u.id)
      .filter((id) => !isCurrentUser(id));
    setSelectedUserIds(checked ? visibleIds : []);
  };

  const handleBulkAction = async (action: unknown, label: string) => {
    const ids = selectedUserIds();
    if (ids.length === 0) return;

    if (label === 'Delete' && !confirm(`Delete ${ids.length} selected users? This cannot be undone.`)) {
      return;
    }

    try {
      setBulkWorking(true);
      setError(null);

      const response = await fetch('/api/v1/admin/users/bulk', {
        method: 'POST',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          user_ids: ids,
          action,
        }),
      });

      if (!response.ok) {
        const body = await response.json().catch(() => ({}));
        throw new Error(body.message || `Failed bulk action: ${label}`);
      }

      await loadUsers(currentPage(), roleFilter());
    } catch (err) {
      console.error(`Bulk action failed (${label}):`, err);
      setError(err instanceof Error ? err.message : `Failed bulk action: ${label}`);
    } finally {
      setBulkWorking(false);
    }
  };

  const openEditModal = (user: User) => {
    setEditingUser(user);
    setEditingRole((user.role.toLowerCase() as Role) || 'user');
    setEditingCanCreateInvites(user.can_create_invites);
    setEditingMaxInvitesMode(user.max_invites === null ? 'unlimited' : 'limited');
    setEditingMaxInvites(user.max_invites === null ? '5' : String(user.max_invites));
  };

  const handleSaveUser = async () => {
    const user = editingUser();
    if (!user) return;

    try {
      setSavingUser(true);
      setError(null);

      const maxInvites =
        editingMaxInvitesMode() === 'unlimited'
          ? null
          : Math.max(0, Number.parseInt(editingMaxInvites() || '0', 10));

      const response = await fetch(`/api/v1/admin/users/${user.id}`, {
        method: 'PUT',
        credentials: 'include',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          role: editingRole(),
          can_create_invites: editingCanCreateInvites(),
          max_invites: maxInvites,
        }),
      });

      if (!response.ok) {
        const body = await response.json().catch(() => ({}));
        throw new Error(body.message || 'Failed to update user');
      }

      setEditingUser(null);
      await loadUsers(currentPage(), roleFilter());
    } catch (err) {
      console.error('Error updating user:', err);
      setError(err instanceof Error ? err.message : 'Failed to update user');
    } finally {
      setSavingUser(false);
    }
  };

  const handleDeleteUser = async (user: User) => {
    const confirmed = confirm(`Delete user "${user.name}" (${user.email})? This cannot be undone.`);
    if (!confirmed) return;

    try {
      setDeletingUserId(user.id);
      setError(null);

      const response = await fetch(`/api/v1/admin/users/${user.id}`, {
        method: 'DELETE',
        credentials: 'include',
      });

      if (!response.ok) {
        const body = await response.json().catch(() => ({}));
        throw new Error(body.message || 'Failed to delete user');
      }

      await loadUsers(currentPage(), roleFilter());
    } catch (err) {
      console.error('Error deleting user:', err);
      setError(err instanceof Error ? err.message : 'Failed to delete user');
    } finally {
      setDeletingUserId(null);
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
              <h2 class="text-lg font-medium text-gray-900">Users ({data()?.total || 0})</h2>
              <div class="text-sm text-gray-500">
                Page {data()?.page || 1} of {data()?.total_pages || 1}
              </div>
            </div>

            <div class="mb-4 p-3 bg-gray-50 border border-gray-200 rounded-md flex flex-wrap items-center gap-2">
              <label class="inline-flex items-center gap-2 text-sm text-gray-700">
                <input
                  type="checkbox"
                  checked={
                    (data()?.users || []).filter((u) => !isCurrentUser(u.id)).length > 0 &&
                    selectedUserIds().length === (data()?.users || []).filter((u) => !isCurrentUser(u.id)).length
                  }
                  onChange={(e) => toggleSelectAllVisible(e.currentTarget.checked)}
                />
                Select visible
              </label>

              <span class="text-sm text-gray-600 ml-2">{selectedUserIds().length} selected</span>

              <div class="flex-1"></div>

              <button
                class="px-3 py-1.5 text-sm rounded-md border border-gray-300 text-gray-700 hover:bg-gray-100 disabled:opacity-50"
                disabled={selectedUserIds().length === 0 || bulkWorking()}
                onClick={() => handleBulkAction('enable_invites', 'Enable invites')}
              >
                Enable Invites
              </button>

              <button
                class="px-3 py-1.5 text-sm rounded-md border border-gray-300 text-gray-700 hover:bg-gray-100 disabled:opacity-50"
                disabled={selectedUserIds().length === 0 || bulkWorking()}
                onClick={() => handleBulkAction('disable_invites', 'Disable invites')}
              >
                Disable Invites
              </button>

              <select
                value={bulkRole()}
                onChange={(e) => setBulkRole(e.currentTarget.value as Role)}
                class="border border-gray-300 rounded-md px-2 py-1.5 text-sm"
              >
                <option value="user">User</option>
                <option value="moderator">Moderator</option>
                <option value="admin">Admin</option>
              </select>

              <button
                class="px-3 py-1.5 text-sm rounded-md bg-green-600 text-white hover:bg-green-700 disabled:opacity-50"
                disabled={selectedUserIds().length === 0 || bulkWorking()}
                onClick={() => handleBulkAction({ set_role: bulkRole() }, 'Set role')}
              >
                Set Role
              </button>

              <button
                class="px-3 py-1.5 text-sm rounded-md bg-red-600 text-white hover:bg-red-700 disabled:opacity-50"
                disabled={selectedUserIds().length === 0 || bulkWorking()}
                onClick={() => handleBulkAction('delete', 'Delete')}
              >
                Delete
              </button>
            </div>

            <div class="overflow-x-auto">
              <table class="min-w-full divide-y divide-gray-200">
                <thead class="bg-gray-50">
                  <tr>
                    <th class="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Sel
                    </th>
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
                        <td class="px-3 py-4 whitespace-nowrap">
                          <input
                            type="checkbox"
                            disabled={isCurrentUser(user.id)}
                            checked={selectedUserIds().includes(user.id)}
                            onChange={(e) => toggleUserSelection(user.id, e.currentTarget.checked)}
                          />
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <div>
                            <div class="text-sm font-medium text-gray-900">{user.name}</div>
                            <div class="text-sm text-gray-500">{user.email}</div>
                            <Show when={isCurrentUser(user.id)}>
                              <div class="text-xs text-blue-600">Current user</div>
                            </Show>
                          </div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap">
                          <span
                            class={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${getRoleBadgeClass(user.role)}`}
                          >
                            {user.role}
                          </span>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                          <Show when={user.can_create_invites} fallback={<span class="text-gray-400">None</span>}>
                            <div>
                              <div>Can create invites</div>
                              <div class="text-xs text-gray-500">
                                {user.invites_remaining !== null ? `${user.invites_remaining} remaining` : 'Unlimited'}
                              </div>
                            </div>
                          </Show>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">{formatDate(user.created_at)}</td>
                        <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-3">
                          <button
                            class="text-green-700 hover:text-green-900"
                            onClick={() => openEditModal(user)}
                          >
                            Edit
                          </button>
                          <button
                            class="text-red-700 hover:text-red-900 disabled:opacity-50"
                            disabled={deletingUserId() === user.id}
                            onClick={() => handleDeleteUser(user)}
                          >
                            {deletingUserId() === user.id ? 'Deleting...' : 'Delete'}
                          </button>
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>

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

      <Show when={editingUser()}>
        <div class="fixed inset-0 z-50 bg-black/40 flex items-center justify-center p-4">
          <div class="w-full max-w-lg rounded-lg bg-white shadow-xl">
            <div class="px-6 py-4 border-b">
              <h3 class="text-lg font-semibold text-gray-900">Edit User</h3>
              <p class="text-sm text-gray-600">{editingUser()?.name} ({editingUser()?.email})</p>
            </div>

            <div class="px-6 py-4 space-y-4">
              <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Role</label>
                <select
                  value={editingRole()}
                  onChange={(e) => setEditingRole(e.currentTarget.value as Role)}
                  class="w-full border border-gray-300 rounded-md px-3 py-2 text-sm"
                >
                  <option value="user">User</option>
                  <option value="moderator">Moderator</option>
                  <option value="admin">Admin</option>
                </select>
              </div>

              <label class="flex items-center gap-2 text-sm text-gray-700">
                <input
                  type="checkbox"
                  checked={editingCanCreateInvites()}
                  onChange={(e) => setEditingCanCreateInvites(e.currentTarget.checked)}
                />
                Can create invites
              </label>

              <div>
                <label class="block text-sm font-medium text-gray-700 mb-1">Invite limit</label>
                <div class="flex gap-3">
                  <label class="inline-flex items-center gap-2 text-sm">
                    <input
                      type="radio"
                      name="max_invites_mode"
                      checked={editingMaxInvitesMode() === 'limited'}
                      onChange={() => setEditingMaxInvitesMode('limited')}
                    />
                    Limited
                  </label>
                  <label class="inline-flex items-center gap-2 text-sm">
                    <input
                      type="radio"
                      name="max_invites_mode"
                      checked={editingMaxInvitesMode() === 'unlimited'}
                      onChange={() => setEditingMaxInvitesMode('unlimited')}
                    />
                    Unlimited
                  </label>
                </div>

                <Show when={editingMaxInvitesMode() === 'limited'}>
                  <input
                    type="number"
                    min="0"
                    value={editingMaxInvites()}
                    onInput={(e) => setEditingMaxInvites(e.currentTarget.value)}
                    class="mt-2 w-full border border-gray-300 rounded-md px-3 py-2 text-sm"
                  />
                </Show>
              </div>
            </div>

            <div class="px-6 py-4 border-t flex justify-end gap-3">
              <button
                type="button"
                onClick={() => setEditingUser(null)}
                class="px-4 py-2 text-sm rounded-md border border-gray-300 text-gray-700 hover:bg-gray-50"
              >
                Cancel
              </button>
              <button
                type="button"
                disabled={savingUser()}
                onClick={handleSaveUser}
                class="px-4 py-2 text-sm rounded-md bg-green-600 text-white hover:bg-green-700 disabled:opacity-50"
              >
                {savingUser() ? 'Saving...' : 'Save Changes'}
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
