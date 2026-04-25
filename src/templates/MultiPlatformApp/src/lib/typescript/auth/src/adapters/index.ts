/**
 * Platform-specific adapter implementations
 *
 * @packageDocumentation
 */

import type { IAuthAdapter } from '../interfaces/auth-adapter';
import { SupabaseWebAdapter, type SupabaseWebAdapterConfig } from './web/supabase-web-adapter';
import {
  SupabaseTauriAdapter,
  type SupabaseTauriAdapterConfig,
} from './tauri/supabase-tauri-adapter';
import { DevAuthAdapter, type DevAuthAdapterConfig } from './dev/dev-auth-adapter';

/**
 * Platform type
 */
export type Platform = 'web' | 'tauri' | 'dev' | 'auto';

/**
 * Configuration for creating an auth adapter
 */
export type CreateAuthAdapterConfig =
  | {
      platform: 'web';
      config: SupabaseWebAdapterConfig;
    }
  | {
      platform: 'tauri';
      config: SupabaseTauriAdapterConfig;
    }
  | {
      platform: 'dev';
      config?: DevAuthAdapterConfig;
    }
  | {
      platform: 'auto';
      web: SupabaseWebAdapterConfig;
      tauri: SupabaseTauriAdapterConfig;
    };

/**
 * Detect the current platform
 *
 * @returns 'tauri' if running in Tauri, 'web' otherwise
 */
export function detectPlatform(): 'web' | 'tauri' {
  // Check for Tauri global
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    return 'tauri';
  }
  return 'web';
}

/**
 * Create an auth adapter for the specified platform
 *
 * Automatically detects the platform if 'auto' is specified.
 *
 * @param options - Configuration options
 * @returns Auth adapter instance
 *
 * @example
 * ```typescript
 * // Explicit platform
 * const adapter = createAuthAdapter({
 *   platform: 'web',
 *   config: {
 *     supabase: {
 *       url: process.env.PUBLIC_SUPABASE_URL!,
 *       anonKey: process.env.PUBLIC_SUPABASE_ANON_KEY!,
 *     },
 *   },
 * });
 * ```
 *
 * @example
 * ```typescript
 * // Auto-detect platform
 * const adapter = createAuthAdapter({
 *   platform: 'auto',
 *   web: {
 *     supabase: {
 *       url: import.meta.env.PUBLIC_SUPABASE_URL,
 *       anonKey: import.meta.env.PUBLIC_SUPABASE_ANON_KEY,
 *     },
 *   },
 *   tauri: {
 *     supabase: {
 *       url: import.meta.env.PUBLIC_SUPABASE_URL,
 *       anonKey: import.meta.env.PUBLIC_SUPABASE_ANON_KEY,
 *     },
 *   },
 * });
 * ```
 */
export function createAuthAdapter(options: CreateAuthAdapterConfig): IAuthAdapter {
  if (options.platform === 'auto') {
    const platform = detectPlatform();
    if (platform === 'tauri') {
      return new SupabaseTauriAdapter(options.tauri);
    }
    return new SupabaseWebAdapter(options.web);
  }

  if (options.platform === 'web') {
    return new SupabaseWebAdapter(options.config);
  }

  if (options.platform === 'tauri') {
    return new SupabaseTauriAdapter(options.config);
  }

  if (options.platform === 'dev') {
    return new DevAuthAdapter(options.config);
  }

  const _exhaustive: never = options;
  throw new Error(`Unsupported platform: ${(_exhaustive as any).platform}`);
}

// Platform adapters are also exported from their respective subdirectories
// Import from @multi-platform-app/auth/adapters/web or @multi-platform-app/auth/adapters/tauri
export * from './web';
export * from './tauri';
export * from './dev';
