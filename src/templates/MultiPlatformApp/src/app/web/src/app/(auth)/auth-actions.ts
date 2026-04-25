'use server';

import { redirect } from 'next/navigation';
import { createSupabaseServerClient, getURL } from '@/lib/auth';

export interface ActionResponse<T = unknown> {
  data: T | null;
  error: Error | null;
}

/**
 * Sign in with OAuth provider (GitHub or Google)
 */
export async function signInWithOAuth(
  provider: 'github' | 'google'
): Promise<ActionResponse<never>> {
  const supabase = await createSupabaseServerClient();

  const { data, error } = await supabase.auth.signInWithOAuth({
    provider,
    options: {
      redirectTo: getURL('/auth/callback'),
    },
  });

  if (error) {
    console.error('OAuth sign in error:', error);
    return { data: null, error };
  }

  return redirect(data.url);
}

/**
 * Sign in with magic link (email OTP)
 */
export async function signInWithEmail(email: string): Promise<ActionResponse<void>> {
  const supabase = await createSupabaseServerClient();

  const { error } = await supabase.auth.signInWithOtp({
    email,
    options: {
      emailRedirectTo: getURL('/auth/callback'),
    },
  });

  if (error) {
    console.error('Email sign in error:', error);
    return { data: null, error };
  }

  return { data: null, error: null };
}

/**
 * Sign in with email and password
 */
export async function signInWithPassword(
  email: string,
  password: string
): Promise<ActionResponse<void>> {
  const supabase = await createSupabaseServerClient();

  const { error } = await supabase.auth.signInWithPassword({
    email,
    password,
  });

  if (error) {
    console.error('Password sign in error:', error);
    return { data: null, error };
  }

  return redirect('/account');
}

/**
 * Sign up with email and password
 */
export async function signUpWithPassword(
  email: string,
  password: string,
  fullName?: string
): Promise<ActionResponse<void>> {
  const supabase = await createSupabaseServerClient();

  const { error } = await supabase.auth.signUp({
    email,
    password,
    options: {
      data: {
        full_name: fullName,
      },
      emailRedirectTo: getURL('/auth/callback'),
    },
  });

  if (error) {
    console.error('Sign up error:', error);
    return { data: null, error };
  }

  return { data: null, error: null };
}

/**
 * Sign out the current user
 */
export async function signOut(): Promise<ActionResponse<void>> {
  const supabase = await createSupabaseServerClient();
  const { error } = await supabase.auth.signOut();

  if (error) {
    console.error('Sign out error:', error);
    return { data: null, error };
  }

  return redirect('/login');
}
