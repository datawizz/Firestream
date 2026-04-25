/**
 * Server-side Supabase utilities
 *
 * Utilities for creating Supabase clients in server environments
 *
 * @packageDocumentation
 */

import { createClient, SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '../types/database';

/**
 * Configuration for creating a Supabase admin client
 */
export interface SupabaseAdminConfig {
  url: string;
  serviceRoleKey: string;
  options?: {
    auth?: {
      persistSession?: boolean;
      autoRefreshToken?: boolean;
    };
  };
}

/**
 * Create a Supabase admin client with service role key
 *
 * This client bypasses Row Level Security (RLS) and should only be used in server environments.
 * Never expose the service role key to the client.
 *
 * @param config - Configuration for the admin client
 * @returns Supabase client with admin privileges
 *
 * @example
 * ```typescript
 * const supabase = createSupabaseAdmin({
 *   url: process.env.SUPABASE_URL,
 *   serviceRoleKey: process.env.SUPABASE_SERVICE_ROLE_KEY,
 * });
 * ```
 */
export function createSupabaseAdmin(
  config: SupabaseAdminConfig
): SupabaseClient<Database> {
  return createClient<Database>(config.url, config.serviceRoleKey, {
    auth: {
      persistSession: false,
      autoRefreshToken: false,
      ...config.options?.auth,
    },
    ...config.options,
  });
}

/**
 * Helper to get environment variable with validation
 *
 * @param value - Environment variable value
 * @param name - Name of the environment variable for error messages
 * @returns The validated environment variable value
 * @throws Error if the environment variable is not set
 */
export function getEnvVar(value: string | undefined, name: string): string {
  if (!value) {
    throw new Error(`Missing required environment variable: ${name}`);
  }
  return value;
}
