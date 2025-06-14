import { Component, createEffect, Show } from 'solid-js';
import { Routes, Route, Navigate } from '@solidjs/router';
import { authStore } from '@/stores/auth';
import { AuthLayout } from '@/components/layouts/AuthLayout';
import { AppLayout } from '@/components/layouts/AppLayout';
import { LoginPage } from '@/routes/LoginPage';
import { RegisterPage } from '@/routes/RegisterPage';
import { PlantsPage } from '@/routes/PlantsPage';
import { PlantDetailPage } from '@/routes/PlantDetailPage';
import { PlantFormPage } from '@/routes/PlantFormPage';
import { CalendarPage } from '@/routes/CalendarPage';
import { CalendarSettingsPage } from '@/routes/CalendarSettingsPage';
import { SearchPage } from '@/routes/SearchPage';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';
import { ThemeProvider } from '@/providers/ThemeProvider';

const App: Component = () => {
  createEffect(() => {
    authStore.initializeAuth();
  });

  return (
    <ThemeProvider defaultTheme="system" storageKey="planty-theme">
      <Show
        when={!authStore.loading}
        fallback={
          <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
            <LoadingSpinner size="lg" />
          </div>
        }
      >
      <Routes>
        <Route
          path="/login"
          component={() => (
            <Show
              when={!authStore.isAuthenticated}
              fallback={<Navigate href="/plants" />}
            >
              <AuthLayout>
                <LoginPage />
              </AuthLayout>
            </Show>
          )}
        />
        <Route
          path="/register"
          component={() => (
            <Show
              when={!authStore.isAuthenticated}
              fallback={<Navigate href="/plants" />}
            >
              <AuthLayout>
                <RegisterPage />
              </AuthLayout>
            </Show>
          )}
        />
        <Route
          path="/plants"
          component={() => (
            <Show
              when={authStore.isAuthenticated}
              fallback={<Navigate href="/login" />}
            >
              <AppLayout>
                <PlantsPage />
              </AppLayout>
            </Show>
          )}
        />
        <Route
          path="/plants/new"
          component={() => (
            <Show
              when={authStore.isAuthenticated}
              fallback={<Navigate href="/login" />}
            >
              <AppLayout>
                <PlantFormPage />
              </AppLayout>
            </Show>
          )}
        />
        <Route
          path="/plants/:id"
          component={() => (
            <Show
              when={authStore.isAuthenticated}
              fallback={<Navigate href="/login" />}
            >
              <AppLayout>
                <PlantDetailPage />
              </AppLayout>
            </Show>
          )}
        />
        <Route
          path="/plants/:id/edit"
          component={() => (
            <Show
              when={authStore.isAuthenticated}
              fallback={<Navigate href="/login" />}
            >
              <AppLayout>
                <PlantFormPage />
              </AppLayout>
            </Show>
          )}
        />
        <Route
          path="/calendar"
          component={() => (
            <Show
              when={authStore.isAuthenticated}
              fallback={<Navigate href="/login" />}
            >
              <AppLayout>
                <CalendarPage />
              </AppLayout>
            </Show>
          )}
        />
        <Route
          path="/calendar/settings"
          component={() => (
            <Show
              when={authStore.isAuthenticated}
              fallback={<Navigate href="/login" />}
            >
              <AppLayout>
                <CalendarSettingsPage />
              </AppLayout>
            </Show>
          )}
        />
        <Route
          path="/search"
          component={() => (
            <Show
              when={authStore.isAuthenticated}
              fallback={<Navigate href="/login" />}
            >
              <AppLayout>
                <SearchPage />
              </AppLayout>
            </Show>
          )}
        />
        <Route path="/" component={() => <Navigate href="/plants" />} />
        <Route path="*" component={() => <Navigate href="/plants" />} />
      </Routes>
    </Show>
    </ThemeProvider>
  );
};

export default App;