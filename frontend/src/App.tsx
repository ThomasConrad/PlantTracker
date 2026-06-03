import {
  Component,
  ParentComponent,
  Show,
  createEffect,
  createMemo,
} from "solid-js";
import { Routes, Route, Navigate, useLocation } from "@solidjs/router";
import { authStore } from "@/stores/auth";
import { AuthLayout } from "@/components/layouts/AuthLayout";
import { AppLayout } from "@/components/layouts/AppLayout";
import { HomePage } from "@/routes/HomePage";
import { LoginPage } from "@/routes/LoginPage";
import { RegisterPage } from "@/routes/RegisterPage";
import { InviteValidationPage } from "@/routes/InviteValidationPage";
import { InviteManagementPage } from "@/routes/InviteManagementPage";
import { AdminDashboardPage } from "@/routes/AdminDashboardPage";
import { AdminUsersPage } from "@/routes/AdminUsersPage";
import { AdminSettingsPage } from "@/routes/AdminSettingsPage";
import { AdminHealthPage } from "@/routes/AdminHealthPage";
import { UserSettingsPage } from "@/routes/UserSettingsPage";
import { PlantsPage } from "@/routes/PlantsPage";
import { PlantDetailPage } from "@/routes/PlantDetailPage";
import { PlantPhotosPage } from "@/routes/PlantPhotosPage";
import { PlantCoachPage } from "@/routes/PlantCoachPage";
import { PlantFormPage } from "@/routes/PlantFormPage";
import { CalendarPage } from "@/routes/CalendarPage";
import { CalendarSettingsPage } from "@/routes/CalendarSettingsPage";
import { RemindersPage } from "@/routes/RemindersPage";
import { SearchPage } from "@/routes/SearchPage";
import { NotFoundPage } from "@/routes/NotFoundPage";
import { PrivacyPage } from "@/routes/PrivacyPage";
import { TermsPage } from "@/routes/TermsPage";
import { ContactPage } from "@/routes/ContactPage";
import { LoadingSpinner } from "@/components/ui/LoadingSpinner";
import { OfflineIndicator } from "@/components/ui/OfflineIndicator";
import { ThemeProvider } from "@/providers/ThemeProvider";
import {
  buildLoginRedirectPath,
  resolvePostLoginPath,
} from "@/utils/authRedirect";

const LoginRoute: Component = () => {
  const location = useLocation();
  const redirectPath = createMemo(() => resolvePostLoginPath(location.search));

  return (
    <Show
      when={!authStore.isAuthenticated}
      fallback={<Navigate href={redirectPath()} />}
    >
      <AuthLayout>
        <LoginPage />
      </AuthLayout>
    </Show>
  );
};

const ProtectedRoute: ParentComponent = (props) => {
  const location = useLocation();
  const loginPath = createMemo(() =>
    buildLoginRedirectPath(location.pathname, location.search, location.hash),
  );

  return (
    <Show
      when={authStore.isAuthenticated}
      fallback={<Navigate href={loginPath()} />}
    >
      {props.children}
    </Show>
  );
};

const AdminRoute: ParentComponent = (props) => {
  const location = useLocation();
  const fallbackPath = createMemo(() => {
    if (!authStore.isAuthenticated) {
      return buildLoginRedirectPath(
        location.pathname,
        location.search,
        location.hash,
      );
    }

    return "/plants";
  });

  return (
    <Show
      when={authStore.isAuthenticated && authStore.user?.role === "admin"}
      fallback={<Navigate href={fallbackPath()} />}
    >
      {props.children}
    </Show>
  );
};

const App: Component = () => {
  createEffect(() => {
    authStore.initializeAuth();
  });

  return (
    <ThemeProvider defaultTheme="system" storageKey="planty-theme">
      <OfflineIndicator />
      <Show
        when={!authStore.loading}
        fallback={
          <div class="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
            <LoadingSpinner size="lg" />
          </div>
        }
      >
        <Routes>
          {/* Public routes */}
          <Route path="/login" component={LoginRoute} />
          <Route
            path="/signup"
            component={() => (
              <Show
                when={!authStore.isAuthenticated}
                fallback={<Navigate href="/plants" />}
              >
                <AuthLayout>
                  <InviteValidationPage />
                </AuthLayout>
              </Show>
            )}
          />
          <Route
            path="/invite"
            component={() => (
              <Show
                when={!authStore.isAuthenticated}
                fallback={<Navigate href="/plants" />}
              >
                <AuthLayout>
                  <InviteValidationPage />
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

          {/* Legal and info pages - accessible to all */}
          <Route path="/404" component={NotFoundPage} />
          <Route path="/privacy" component={PrivacyPage} />
          <Route path="/terms" component={TermsPage} />
          <Route path="/contact" component={ContactPage} />

          {/* Protected routes */}
          <Route
            path="/plants"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <PlantsPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/plants/new"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <PlantFormPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/plants/:id"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <PlantDetailPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/plants/:id/edit"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <PlantFormPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/plants/:id/photos"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <PlantPhotosPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/plants/:id/coach"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <PlantCoachPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/calendar"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <CalendarPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/calendar/settings"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <CalendarSettingsPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/reminders"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <RemindersPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/search"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <SearchPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/invites"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <InviteManagementPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/admin/dashboard"
            component={() => (
              <AdminRoute>
                <AppLayout>
                  <AdminDashboardPage />
                </AppLayout>
              </AdminRoute>
            )}
          />
          <Route
            path="/admin/users"
            component={() => (
              <AdminRoute>
                <AppLayout>
                  <AdminUsersPage />
                </AppLayout>
              </AdminRoute>
            )}
          />
          <Route
            path="/admin/settings"
            component={() => (
              <AdminRoute>
                <AppLayout>
                  <AdminSettingsPage />
                </AppLayout>
              </AdminRoute>
            )}
          />
          <Route
            path="/admin/health"
            component={() => (
              <AdminRoute>
                <AppLayout>
                  <AdminHealthPage />
                </AppLayout>
              </AdminRoute>
            )}
          />
          <Route
            path="/settings"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <UserSettingsPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />
          <Route
            path="/settings/user"
            component={() => (
              <ProtectedRoute>
                <AppLayout>
                  <UserSettingsPage />
                </AppLayout>
              </ProtectedRoute>
            )}
          />

          {/* Home route */}
          <Route
            path="/"
            component={() => (
              <Show when={authStore.isAuthenticated} fallback={<HomePage />}>
                <Navigate href="/plants" />
              </Show>
            )}
          />

          {/* Catch-all 404 route - must be last */}
          <Route path="*" component={NotFoundPage} />
        </Routes>
      </Show>
    </ThemeProvider>
  );
};

export default App;
