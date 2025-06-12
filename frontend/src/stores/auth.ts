import { createSignal, createEffect } from 'solid-js';
import { apiClient } from '@/api/client';
import type { User, components } from '@/types';

type LoginRequest = components['schemas']['LoginRequest'];
type RegisterRequest = components['schemas']['CreateUserRequest'];

const [isAuthenticated, setIsAuthenticated] = createSignal(false);
const [user, setUser] = createSignal<User | null>(null);
const [loading, setLoading] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);

const authStore = {
  get isAuthenticated() {
    return isAuthenticated();
  },
  get user() {
    return user();
  },
  get loading() {
    return loading();
  },
  get error() {
    return error();
  },

  async login(credentials: LoginRequest): Promise<void> {
    try {
      setLoading(true);
      setError(null);
      const response = await apiClient.login(credentials);
      setUser(response.user);
      setIsAuthenticated(true);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Login failed';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  },

  async register(userData: RegisterRequest): Promise<void> {
    try {
      setLoading(true);
      setError(null);
      const response = await apiClient.register(userData);
      setUser(response.user);
      setIsAuthenticated(true);
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Registration failed';
      setError(errorMessage);
      throw err;
    } finally {
      setLoading(false);
    }
  },

  async logout(): Promise<void> {
    try {
      await apiClient.logout();
      setUser(null);
      setIsAuthenticated(false);
      setError(null);
    } catch (err: unknown) {
      console.error('Logout error:', err);
    }
  },

  async initializeAuth(): Promise<void> {
    try {
      setLoading(true);
      const userData = await apiClient.getCurrentUser();
      setUser(userData);
      setIsAuthenticated(true);
    } catch (err: unknown) {
      // No session found or session expired
      // No active session found
    } finally {
      setLoading(false);
    }
  },

  clearError(): void {
    setError(null);
  },
};

createEffect(() => {
  authStore.initializeAuth();
});

export { authStore };