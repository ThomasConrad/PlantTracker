import { Component, createSignal, createEffect, Show, For } from 'solid-js';
import { A } from '@solidjs/router';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { authStore } from '@/stores/auth';
import { apiClient } from '@/api/client';

interface InviteCode {
  id: string;
  code: string;
  max_uses: number;
  current_uses: number;
  is_active: boolean;
  expires_at?: string;
  created_at: string;
}

export const InviteManagementPage: Component = () => {
  const [invites, setInvites] = createSignal<InviteCode[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [createLoading, setCreateLoading] = createSignal(false);
  const [error, setError] = createSignal('');
  const [maxUses, setMaxUses] = createSignal(1);
  const [expiresInDays, setExpiresInDays] = createSignal<number>();

  createEffect(() => {
    loadInvites();
  });

  const loadInvites = async () => {
    try {
      setLoading(true);
      setError('');
      const response = await apiClient.request<{ invites: InviteCode[] }>('/invites/list');
      setInvites(response.invites || []);
    } catch (err: any) {
      setError('Failed to load invites');
      console.error('Failed to load invites:', err);
    } finally {
      setLoading(false);
    }
  };

  const createInvite = async (e: Event) => {
    e.preventDefault();
    
    try {
      setCreateLoading(true);
      setError('');
      
      const expiresAt = expiresInDays() ? 
        new Date(Date.now() + expiresInDays()! * 24 * 60 * 60 * 1000).toISOString() : 
        undefined;

      const newInvite = await apiClient.request<InviteCode>('/invites/create', {
        method: 'POST',
        body: JSON.stringify({
          max_uses: maxUses(),
          expires_at: expiresAt,
        }),
      });

      setInvites([newInvite, ...invites()]);
      setMaxUses(1);
      setExpiresInDays(undefined);
    } catch (err: any) {
      setError('Failed to create invite');
      console.error('Failed to create invite:', err);
    } finally {
      setCreateLoading(false);
    }
  };

  const copyInviteLink = (code: string) => {
    const url = `${window.location.origin}/invite?code=${code}`;
    navigator.clipboard.writeText(url);
    // You could add a toast notification here
  };

  return (
    <div class="max-w-4xl mx-auto p-6 space-y-6">
      <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold text-gray-900">Invite Management</h1>
        <A href="/plants" class="text-primary-600 hover:text-primary-500">
          ← Back to Plants
        </A>
      </div>

      <Show when={!authStore.user?.can_create_invites}>
        <div class="bg-yellow-50 border border-yellow-200 rounded-md p-4">
          <p class="text-sm text-yellow-800">
            You don't have permission to create invite codes. Contact an administrator for access.
          </p>
        </div>
      </Show>

      <Show when={authStore.user?.can_create_invites}>
        <div class="bg-white shadow rounded-lg p-6">
          <h2 class="text-lg font-medium text-gray-900 mb-4">Create New Invite</h2>
          
          <Show when={authStore.user?.max_invites !== null}>
            <div class="mb-4 p-4 bg-blue-50 rounded-md">
              <p class="text-sm text-blue-700">
                Invites remaining: {authStore.user?.invites_remaining ?? 0} / {authStore.user?.max_invites ?? 0}
              </p>
            </div>
          </Show>

          <Show when={authStore.user?.max_invites === null}>
            <div class="mb-4 p-4 bg-green-50 rounded-md">
              <p class="text-sm text-green-700">
                You have unlimited invite creation privileges.
              </p>
            </div>
          </Show>

          <Show when={(authStore.user?.invites_remaining ?? 0) > 0 || authStore.user?.max_invites === null}>
            <form onSubmit={createInvite} class="space-y-4">
              <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Input
                  label="Maximum Uses"
                  type="number"
                  min="1"
                  max="100"
                  value={maxUses()}
                  onInput={(e) => setMaxUses(parseInt(e.currentTarget.value) || 1)}
                  required
                />
                
                <Input
                  label="Expires in Days (optional)"
                  type="number"
                  min="1"
                  max="365"
                  value={expiresInDays() || ''}
                  onInput={(e) => {
                    const val = e.currentTarget.value;
                    setExpiresInDays(val ? parseInt(val) : undefined);
                  }}
                  placeholder="Never expires"
                />
              </div>

              {error() && (
                <div class="bg-red-50 border border-red-200 rounded-md p-4">
                  <p class="text-sm text-red-600">{error()}</p>
                </div>
              )}

              <Button
                type="submit"
                loading={createLoading()}
                disabled={createLoading()}
              >
                Create Invite Code
              </Button>
            </form>
          </Show>

          <Show when={(authStore.user?.invites_remaining ?? 0) <= 0 && authStore.user?.max_invites !== null}>
            <div class="bg-orange-50 border border-orange-200 rounded-md p-4">
              <p class="text-sm text-orange-800">
                You have reached your invite creation limit. Contact an administrator to increase your limit.
              </p>
            </div>
          </Show>
        </div>
      </Show>

      <div class="bg-white shadow rounded-lg p-6">
        <h2 class="text-lg font-medium text-gray-900 mb-4">Your Invite Codes</h2>
        
        <Show when={loading()}>
          <p class="text-gray-500">Loading invites...</p>
        </Show>

        <Show when={!loading() && invites().length === 0}>
          <p class="text-gray-500">You haven't created any invite codes yet.</p>
        </Show>

        <Show when={!loading() && invites().length > 0}>
          <div class="space-y-4">
            <For each={invites()}>
              {(invite) => (
                <div class="border border-gray-200 rounded-lg p-4">
                  <div class="flex items-center justify-between">
                    <div class="flex-1">
                      <div class="flex items-center space-x-4">
                        <code class="bg-gray-100 px-2 py-1 rounded text-sm font-mono">
                          {invite.code}
                        </code>
                        <span class={`px-2 py-1 rounded-full text-xs font-medium ${
                          invite.is_active ? 'bg-green-100 text-green-800' : 'bg-red-100 text-red-800'
                        }`}>
                          {invite.is_active ? 'Active' : 'Inactive'}
                        </span>
                      </div>
                      
                      <div class="mt-2 text-sm text-gray-600 space-x-4">
                        <span>Uses: {invite.current_uses} / {invite.max_uses}</span>
                        <span>Created: {new Date(invite.created_at).toLocaleDateString()}</span>
                        {invite.expires_at && (
                          <span>Expires: {new Date(invite.expires_at).toLocaleDateString()}</span>
                        )}
                      </div>
                    </div>
                    
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => copyInviteLink(invite.code)}
                    >
                      Copy Link
                    </Button>
                  </div>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};