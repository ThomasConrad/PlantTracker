import { Component, createSignal, onMount, Show } from 'solid-js';
import { A } from '@solidjs/router';

interface SystemHealth {
  status: string;
  timestamp: string;
  database: {
    status: string;
    size_bytes: number;
    page_count: number;
    page_size: number;
  };
  activity_24h: {
    new_users: number;
    new_invites: number;
  };
  uptime: {
    note: string;
  };
}

export const AdminHealthPage: Component = () => {
  const [health, setHealth] = createSignal<SystemHealth | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [lastRefresh, setLastRefresh] = createSignal<Date>(new Date());

  const loadHealth = async () => {
    try {
      setLoading(true);
      setError(null);
      
      const response = await fetch('/api/v1/admin/health', {
        credentials: 'include'
      });
      
      if (!response.ok) {
        throw new Error('Failed to load system health');
      }
      
      const data = await response.json();
      setHealth(data);
      setLastRefresh(new Date());
    } catch (err) {
      console.error('Error loading health:', err);
      setError(err instanceof Error ? err.message : 'Failed to load system health');
    } finally {
      setLoading(false);
    }
  };

  const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  };

  const formatTimestamp = (timestamp: string) => {
    return new Date(timestamp).toLocaleString();
  };

  const getStatusColor = (status: string) => {
    switch (status.toLowerCase()) {
      case 'healthy':
        return 'text-green-600';
      case 'unhealthy':
        return 'text-red-600';
      case 'warning':
        return 'text-yellow-600';
      default:
        return 'text-gray-600';
    }
  };

  const getStatusBadge = (status: string) => {
    switch (status.toLowerCase()) {
      case 'healthy':
        return 'bg-green-100 text-green-800';
      case 'unhealthy':
        return 'bg-red-100 text-red-800';
      case 'warning':
        return 'bg-yellow-100 text-yellow-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  onMount(() => {
    loadHealth();
    
    // Auto-refresh every 30 seconds
    const interval = setInterval(loadHealth, 30000);
    
    return () => clearInterval(interval);
  });

  return (
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
      <div class="mb-8">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-3xl font-bold text-gray-900">System Health</h1>
            <p class="mt-2 text-gray-600">Monitor system status and performance metrics</p>
          </div>
          <div class="flex items-center space-x-4">
            <button
              onClick={loadHealth}
              disabled={loading()}
              class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50"
            >
              <svg class="mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Refresh
            </button>
            <A
              href="/admin/dashboard"
              class="inline-flex items-center px-4 py-2 border border-gray-300 rounded-md shadow-sm text-sm font-medium text-gray-700 bg-white hover:bg-gray-50"
            >
              ← Back to Dashboard
            </A>
          </div>
        </div>
      </div>

      {/* Last Refresh Info */}
      <div class="mb-6">
        <div class="bg-blue-50 border border-blue-200 rounded-md p-4">
          <div class="flex items-center">
            <svg class="h-5 w-5 text-blue-400 mr-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span class="text-sm text-blue-800">
              Last updated: {lastRefresh().toLocaleTimeString()} (Auto-refreshes every 30 seconds)
            </span>
          </div>
        </div>
      </div>

      <Show 
        when={!loading() && !error() && health()} 
        fallback={
          <Show 
            when={loading()} 
            fallback={
              <div class="text-center py-12">
                <div class="text-red-600">{error()}</div>
                <button 
                  onClick={loadHealth}
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
        <div class="space-y-8">
          {/* Overall System Status */}
          <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="px-4 py-5 sm:p-6">
              <div class="flex items-center justify-between">
                <div>
                  <h2 class="text-lg font-medium text-gray-900">System Status</h2>
                  <p class="text-sm text-gray-500">Overall system health</p>
                </div>
                <div class="text-right">
                  <span class={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${getStatusBadge(health()?.status || 'unknown')}`}>
                    {health()?.status || 'Unknown'}
                  </span>
                  <p class="text-xs text-gray-500 mt-1">
                    {formatTimestamp(health()?.timestamp || '')}
                  </p>
                </div>
              </div>
            </div>
          </div>

          {/* Database Health */}
          <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="px-4 py-5 sm:p-6">
              <h2 class="text-lg font-medium text-gray-900 mb-4">Database Health</h2>
              
              <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <div class="text-center p-4 bg-gray-50 rounded-lg">
                  <div class={`text-2xl font-bold ${getStatusColor(health()?.database.status || 'unknown')}`}>
                    {health()?.database.status || 'Unknown'}
                  </div>
                  <div class="text-sm text-gray-600">Connection Status</div>
                </div>
                
                <div class="text-center p-4 bg-gray-50 rounded-lg">
                  <div class="text-2xl font-bold text-blue-600">
                    {formatBytes(health()?.database.size_bytes || 0)}
                  </div>
                  <div class="text-sm text-gray-600">Database Size</div>
                </div>
                
                <div class="text-center p-4 bg-gray-50 rounded-lg">
                  <div class="text-2xl font-bold text-purple-600">
                    {health()?.database.page_count?.toLocaleString() || 0}
                  </div>
                  <div class="text-sm text-gray-600">Pages</div>
                </div>
                
                <div class="text-center p-4 bg-gray-50 rounded-lg">
                  <div class="text-2xl font-bold text-indigo-600">
                    {formatBytes(health()?.database.page_size || 0)}
                  </div>
                  <div class="text-sm text-gray-600">Page Size</div>
                </div>
              </div>
            </div>
          </div>

          {/* Activity Metrics */}
          <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="px-4 py-5 sm:p-6">
              <h2 class="text-lg font-medium text-gray-900 mb-4">24-Hour Activity</h2>
              
              <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="text-center p-4 bg-green-50 rounded-lg">
                  <div class="text-3xl font-bold text-green-600">
                    {health()?.activity_24h.new_users || 0}
                  </div>
                  <div class="text-sm text-green-800">New Users</div>
                  <div class="text-xs text-gray-500 mt-1">Last 24 hours</div>
                </div>
                
                <div class="text-center p-4 bg-blue-50 rounded-lg">
                  <div class="text-3xl font-bold text-blue-600">
                    {health()?.activity_24h.new_invites || 0}
                  </div>
                  <div class="text-sm text-blue-800">New Invites</div>
                  <div class="text-xs text-gray-500 mt-1">Last 24 hours</div>
                </div>
              </div>
            </div>
          </div>

          {/* System Information */}
          <div class="bg-white overflow-hidden shadow rounded-lg">
            <div class="px-4 py-5 sm:p-6">
              <h2 class="text-lg font-medium text-gray-900 mb-4">System Information</h2>
              
              <div class="space-y-4">
                <div class="flex justify-between items-center p-3 bg-gray-50 rounded">
                  <span class="text-sm font-medium text-gray-700">Application Status</span>
                  <span class={`text-sm ${getStatusColor(health()?.status || 'unknown')}`}>
                    {health()?.status || 'Unknown'}
                  </span>
                </div>
                
                <div class="flex justify-between items-center p-3 bg-gray-50 rounded">
                  <span class="text-sm font-medium text-gray-700">Database Type</span>
                  <span class="text-sm text-gray-600">SQLite</span>
                </div>
                
                <div class="flex justify-between items-center p-3 bg-gray-50 rounded">
                  <span class="text-sm font-medium text-gray-700">Uptime Tracking</span>
                  <span class="text-sm text-gray-600">{health()?.uptime.note || 'Not implemented'}</span>
                </div>
              </div>
            </div>
          </div>

          {/* Performance Tips */}
          <div class="bg-yellow-50 border border-yellow-200 rounded-md p-4">
            <div class="flex">
              <div class="flex-shrink-0">
                <svg class="h-5 w-5 text-yellow-400" fill="currentColor" viewBox="0 0 20 20">
                  <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                </svg>
              </div>
              <div class="ml-3">
                <h3 class="text-sm font-medium text-yellow-800">Performance Tips</h3>
                <div class="mt-2 text-sm text-yellow-700">
                  <ul class="list-disc pl-5 space-y-1">
                    <li>Monitor database size regularly and consider maintenance if it grows too large</li>
                    <li>High activity might indicate the need for performance optimization</li>
                    <li>Check system resources if database operations become slow</li>
                    <li>Consider backup strategies for important data</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};