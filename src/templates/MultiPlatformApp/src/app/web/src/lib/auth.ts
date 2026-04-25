/**
 * Auth library helpers for Next.js web app
 *
 * Provides server-side authentication utilities using @multi-platform-app/auth
 */

import { cookies } from 'next/headers';
import { createSupabaseAdmin, getEnvVar } from '@multi-platform-app/auth/server';
import { createBrowserClient, createServerClient } from '@supabase/ssr';
import type { Database } from '@multi-platform-app/auth/types';

/**
 * Create a Supabase client for server components
 * Uses cookies for session management
 */
export async function createSupabaseServerClient() {
  const cookieStore = await cookies();

  return createServerClient<Database>(
    getEnvVar(process.env.PUBLIC_SUPABASE_URL, 'PUBLIC_SUPABASE_URL'),
    getEnvVar(process.env.PUBLIC_SUPABASE_ANON_KEY, 'PUBLIC_SUPABASE_ANON_KEY'),
    {
      cookies: {
        getAll() {
          return cookieStore.getAll();
        },
        setAll(cookiesToSet) {
          try {
            cookiesToSet.forEach(({ name, value, options }) => {
              cookieStore.set(name, value, options);
            });
          } catch (error) {
            // Handle cookie setting errors (e.g., in middleware)
          }
        },
      },
    }
  );
}

/**
 * Create a Supabase client for client components
 * Uses localStorage for session management
 */
export function createSupabaseBrowserClient() {
  return createBrowserClient<Database>(
    getEnvVar(process.env.PUBLIC_SUPABASE_URL!, 'PUBLIC_SUPABASE_URL'),
    getEnvVar(process.env.PUBLIC_SUPABASE_ANON_KEY!, 'PUBLIC_SUPABASE_ANON_KEY')
  );
}

/**
 * Create a Supabase admin client (server-only)
 * Bypasses RLS - use with caution
 */
export function createSupabaseAdminClient() {
  return createSupabaseAdmin({
    url: getEnvVar(process.env.PUBLIC_SUPABASE_URL, 'PUBLIC_SUPABASE_URL'),
    serviceRoleKey: getEnvVar(process.env.SUPABASE_SERVICE_ROLE_KEY, 'SUPABASE_SERVICE_ROLE_KEY'),
  });
}

/**
 * Get the current session from server context
 */
export async function getSession() {
  const supabase = await createSupabaseServerClient();
  const { data } = await supabase.auth.getSession();
  return data.session;
}

/**
 * Get the current user from server context
 */
export async function getUser() {
  const supabase = await createSupabaseServerClient();
  const { data } = await supabase.auth.getUser();
  return data.user;
}

/**
 * Get the current subscription from server context
 */
export async function getSubscription() {
  const supabase = await createSupabaseServerClient();
  const { data } = await supabase.auth.getSession();

  if (!data.session) {
    return null;
  }

  const { data: subscription } = await supabase
    .from('subscriptions')
    .select('*, prices(*, products(*))')
    .eq('user_id', data.session.user.id)
    .in('status', ['active', 'trialing'])
    .maybeSingle();

  return subscription;
}

/**
 * Get all products with their prices
 */
export async function getProducts() {
  const supabase = await createSupabaseServerClient();

  const { data: products } = await supabase
    .from('products')
    .select('*, prices(*)')
    .eq('active', true)
    .eq('prices.active', true)
    .order('metadata->index')
    .order('unit_amount', { referencedTable: 'prices' });

  return products || [];
}

/**
 * Helper to get the base URL for the application
 */
export function getURL(path: string = ''): string {
  let url =
    process.env.PUBLIC_SITE_URL ||
    process.env.NEXT_PUBLIC_VERCEL_URL ||
    'http://localhost:3000';

  url = url.startsWith('http') ? url : `https://${url}`;
  url = url.endsWith('/') ? url : `${url}/`;
  path = path.startsWith('/') ? path.substring(1) : path;

  return `${url}${path}`;
}
