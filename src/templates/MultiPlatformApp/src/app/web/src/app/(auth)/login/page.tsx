import { redirect } from 'next/navigation';
import { getSession, getSubscription } from '@/lib/auth';
import { hasValidSupabaseConfig } from '@/lib/supabase-config';
import { signInWithEmail, signInWithOAuth, signInWithPassword } from '../auth-actions';
import { AuthUI } from '../auth-ui';
import { DevLoginForm } from '../dev-login-form';

export const metadata = {
  title: 'Sign In',
  description: 'Sign in to your account',
};

export default async function LoginPage() {
  if (!hasValidSupabaseConfig()) {
    return <DevLoginForm />;
  }

  const session = await getSession();
  const subscription = await getSubscription();

  // If user has session and subscription, redirect to account
  if (session && subscription) {
    redirect('/account');
  }

  // If user has session but no subscription, redirect to pricing
  if (session && !subscription) {
    redirect('/pricing');
  }

  return (
    <AuthUI
      mode="login"
      signInWithOAuth={signInWithOAuth}
      signInWithEmail={signInWithEmail}
      signInWithPassword={signInWithPassword}
    />
  );
}
