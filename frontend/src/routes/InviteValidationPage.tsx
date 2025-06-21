import { Component, createSignal, onMount } from 'solid-js';
import { A, useNavigate, useSearchParams } from '@solidjs/router';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { apiClient } from '@/api/client';

export const InviteValidationPage: Component = () => {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const [inviteCode, setInviteCode] = createSignal('');
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal('');

  onMount(() => {
    // Check for invite code in URL parameters
    const urlCode = searchParams.invite || searchParams.code;
    if (urlCode) {
      setInviteCode(urlCode);
      // Auto-validate and redirect to register page
      handleInviteValidation(urlCode);
    }
  });

  const handleInviteValidation = async (code: string) => {
    try {
      setLoading(true);
      setError('');
      
      await apiClient.validateInvite({ code: code.trim() });
      
      // Redirect to register page with the validated invite code
      navigate(`/register?code=${encodeURIComponent(code.trim())}`);
    } catch (error: unknown) {
      console.error('Invite validation failed:', error);
      if (error && typeof error === 'object' && 'response' in error && 
          (error as { response?: { status?: number } }).response?.status === 400) {
        setError('Invalid or expired invite code. Please check your code and try again.');
      } else {
        setError('Failed to validate invite code. Please try again.');
      }
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    
    if (!inviteCode().trim()) {
      setError('Please enter an invite code');
      return;
    }

    await handleInviteValidation(inviteCode());
  };

  return (
    <div class="space-y-6">
      <div class="text-center">
        <div class="w-16 h-16 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-4">
          <span class="text-2xl">🎫</span>
        </div>
        <h2 class="text-2xl font-bold text-gray-900">Enter Invite Code</h2>
        <p class="mt-2 text-sm text-gray-600">
          You need a valid invite code to create an account
        </p>
      </div>

      <form onSubmit={handleSubmit} class="space-y-6">
        <Input
          label="Invite Code"
          type="text"
          value={inviteCode()}
          onInput={(e) => setInviteCode(e.currentTarget.value)}
          placeholder="Enter your invite code"
          required
          autocomplete="off"
          class="text-center font-mono tracking-wider"
        />

        {error() && (
          <div class="bg-red-50 border border-red-200 rounded-md p-4">
            <p class="text-sm text-red-600">{error()}</p>
          </div>
        )}

        <Button
          type="submit"
          class="w-full"
          loading={loading()}
          disabled={!inviteCode().trim()}
        >
          Validate Invite Code
        </Button>
      </form>

      <div class="text-center space-y-4">
        <div class="relative">
          <div class="absolute inset-0 flex items-center">
            <div class="w-full border-t border-gray-300" />
          </div>
          <div class="relative flex justify-center text-sm">
            <span class="px-2 bg-white text-gray-500">Already have an account?</span>
          </div>
        </div>
        
        <A 
          href="/login" 
          class="block w-full text-center py-2 text-primary-600 hover:text-primary-500 font-medium"
        >
          Sign in to your account
        </A>
      </div>
    </div>
  );
};