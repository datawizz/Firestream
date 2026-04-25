/**
 * Tauri platform Supabase client adapter
 *
 * Browser Supabase client with custom storage callbacks for Tauri.
 *
 * @packageDocumentation
 */

import { createClient, SupabaseClient } from '@supabase/supabase-js';
import type { Database } from '../../types/database';
import { TauriSessionStorageAdapter } from './session-storage.adapter';

/**
 * Configuration for Tauri Supabase client
 */
export interface TauriSupabaseClientConfig {
  /**
   * Supabase URL
   */
  url: string;

  /**
   * Supabase anon key
   */
  anonKey: string;

  /**
   * Session storage adapter (optional, will create default if not provided)
   */
  sessionStorage?: TauriSessionStorageAdapter;

  /**
   * Use fallback storage (localStorage) in development
   */
  useFallback?: boolean;
}

/**
 * Create a Supabase client for Tauri desktop applications
 *
 * Uses custom storage callbacks to integrate with Tauri's secure storage.
 * Falls back to localStorage in development mode.
 *
 * @param config - Configuration options
 * @returns Supabase client instance
 */
export function createTauriSupabaseClient(
  config: TauriSupabaseClientConfig
): SupabaseClient<Database> {
  const { url, anonKey, sessionStorage, useFallback } = config;

  // Use provided session storage or create a new one
  const storage =
    sessionStorage || new TauriSessionStorageAdapter({ useFallback });

  return createClient<Database>(url, anonKey, {
    auth: {
      // Use custom storage callbacks
      storage: {
        async getItem(_key: string): Promise<string | null> {
          try {
            const session = await storage.getSession();
            if (!session) return null;

            // Return the full session as JSON
            return JSON.stringify(session);
          } catch (error) {
            console.error('Failed to get item from storage:', error);
            return null;
          }
        },

        async setItem(_key: string, value: string): Promise<void> {
          try {
            const session = JSON.parse(value);
            await storage.setSession(session);
          } catch (error) {
            console.error('Failed to set item in storage:', error);
          }
        },

        async removeItem(_key: string): Promise<void> {
          try {
            await storage.clearSession();
          } catch (error) {
            console.error('Failed to remove item from storage:', error);
          }
        },
      },
      // Persist session across app restarts
      persistSession: true,
      // Auto refresh tokens
      autoRefreshToken: true,
      // Detect session in URL (for OAuth redirects)
      detectSessionInUrl: true,
    },
  });
}

/**
 * Helper to check if running in Tauri
 */
export function isTauri(): boolean {
  return '__TAURI__' in window;
}
