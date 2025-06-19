import { Component, createSignal, onMount } from 'solid-js';
import { A, useNavigate, useSearchParams } from '@solidjs/router';
import { authStore } from '@/stores/auth';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';

export const RegisterPage: Component = () => {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [name, setName] = createSignal('');
  const [email, setEmail] = createSignal('');
  const [password, setPassword] = createSignal('');
  const [confirmPassword, setConfirmPassword] = createSignal('');
  const [inviteCode, setInviteCode] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [errors, setErrors] = createSignal<Record<string, string>>({});

  onMount(() => {
    const invite = searchParams.invite;
    if (invite) {
      setInviteCode(invite);
    }
  });

  const validateForm = () => {
    const newErrors: Record<string, string> = {};

    if (!name().trim()) {
      newErrors.name = 'Name is required';
    }

    if (!email().trim()) {
      newErrors.email = 'Email is required';
    } else if (!/\S+@\S+\.\S+/.test(email())) {
      newErrors.email = 'Email is invalid';
    }

    if (!password()) {
      newErrors.password = 'Password is required';
    } else if (password().length < 8) {
      newErrors.password = 'Password must be at least 8 characters';
    }

    if (password() !== confirmPassword()) {
      newErrors.confirmPassword = 'Passwords do not match';
    }

    if (!inviteCode().trim()) {
      newErrors.inviteCode = 'Invite code is required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    
    if (!validateForm()) return;

    try {
      setLoading(true);
      await authStore.register({
        name: name().trim(),
        email: email().trim(),
        password: password(),
        invite_code: inviteCode().trim(),
      });
      navigate('/plants');
    } catch (error) {
      console.error('Registration failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div class="space-y-6">
      <div>
        <h2 class="text-2xl font-bold text-gray-900">Create your account</h2>
        <p class="mt-2 text-sm text-gray-600">
          Or{' '}
          <A href="/login" class="font-medium text-primary-600 hover:text-primary-500">
            sign in to your existing account
          </A>
        </p>
      </div>

      <form onSubmit={handleSubmit} class="space-y-6">
        <Input
          label="Full name"
          type="text"
          value={name()}
          onInput={(e) => setName(e.currentTarget.value)}
          error={errors().name}
          required
          autocomplete="name"
        />

        <Input
          label="Email address"
          type="email"
          value={email()}
          onInput={(e) => setEmail(e.currentTarget.value)}
          error={errors().email}
          required
          autocomplete="email"
        />

        <Input
          label="Password"
          type="password"
          value={password()}
          onInput={(e) => setPassword(e.currentTarget.value)}
          error={errors().password}
          required
          autocomplete="new-password"
        />

        <Input
          label="Confirm password"
          type="password"
          value={confirmPassword()}
          onInput={(e) => setConfirmPassword(e.currentTarget.value)}
          error={errors().confirmPassword}
          required
          autocomplete="new-password"
        />

        <Input
          label="Invite code"
          type="text"
          value={inviteCode()}
          onInput={(e) => setInviteCode(e.currentTarget.value)}
          error={errors().inviteCode}
          required
          placeholder="Enter your invite code"
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
          disabled={!name() || !email() || !password() || !confirmPassword() || !inviteCode()}
        >
          Create account
        </Button>
      </form>
    </div>
  );
};