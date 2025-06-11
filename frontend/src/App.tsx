import { Component, createEffect, Show } from 'solid-js';
import { Routes, Route, Navigate } from '@solidjs/router';
import { authStore } from '@/stores/auth';
import { AuthLayout } from '@/components/layouts/AuthLayout';
import { AppLayout } from '@/components/layouts/AppLayout';
import { LoginPage } from '@/routes/LoginPage';
import { RegisterPage } from '@/routes/RegisterPage';
import { PlantsPage } from '@/routes/PlantsPage';
import { PlantDetailPage } from '@/routes/PlantDetailPage';
import { CreatePlantPage } from '@/routes/CreatePlantPage';
import { EditPlantPage } from '@/routes/EditPlantPage';
import { CalendarPage } from '@/routes/CalendarPage';
import { CalendarSettingsPage } from '@/routes/CalendarSettingsPage';
import { SearchPage } from '@/routes/SearchPage';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

const App: Component = () => {
  createEffect(() => {
    authStore.initializeAuth();
  });

  return (
    <Show
      when={!authStore.loading}
      fallback={
        <div class="min-h-screen flex items-center justify-center">
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
                <CreatePlantPage />
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
                <EditPlantPage />
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
  );
};

export default App;