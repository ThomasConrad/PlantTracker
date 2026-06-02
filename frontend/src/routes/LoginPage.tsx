import { Component, createSignal } from 'solid-js';
import { useLocation, useNavigate } from '@solidjs/router';
import { authStore } from '@/stores/auth';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { resolvePostLoginPath } from '@/utils/authRedirect';

export const LoginPage: Component = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [loading, setLoading] = createSignal(false);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    
    if (!email() || !password()) return;

    try {
      setLoading(true);
      await authStore.login({
        email: email(),
        password: password(),
      });
      navigate(resolvePostLoginPath(location.search), { replace: true });
    } catch (error) {
      console.error('Login failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-gray-900 dark:text-gray-100">Sign in</h2>
      </div>

      <form onSubmit={handleSubmit} class="space-y-5">
        <Input
          label="Email"
          type="email"
          value={email()}
          onInput={(e) => setEmail(e.currentTarget.value)}
          required
          autocomplete="email"
        />

        <Input
          label="Password"
          type="password"
          value={password()}
          onInput={(e) => setPassword(e.currentTarget.value)}
          required
          autocomplete="current-password"
        />

        {authStore.error && (
          <div class="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-md p-3">
            <p class="text-sm text-red-600 dark:text-red-400">{authStore.error}</p>
          </div>
        )}

        <Button
          type="submit"
          class="w-full"
          loading={loading()}
          disabled={!email() || !password()}
        >
          Sign in
        </Button>
      </form>
    </div>
  );
};
