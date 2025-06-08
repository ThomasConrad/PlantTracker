import { Component, createSignal } from 'solid-js';
import { A, useNavigate } from '@solidjs/router';
import { authStore } from '@/stores/auth';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';

export const LoginPage: Component = () => {
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
      navigate('/plants');
    } catch (error) {
      console.error('Login failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-gray-900">Sign in to your account</h2>
        <p class="mt-2 text-sm text-gray-600">
          Or{' '}
          <A href="/register" class="font-medium text-primary-600 hover:text-primary-500">
            create a new account
          </A>
        </p>
      </div>

      <form onSubmit={handleSubmit} class="space-y-6">
        <Input
          label="Email address"
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
          <div class="bg-red-50 border border-red-200 rounded-md p-4">
            <p class="text-sm text-red-600">{authStore.error}</p>
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