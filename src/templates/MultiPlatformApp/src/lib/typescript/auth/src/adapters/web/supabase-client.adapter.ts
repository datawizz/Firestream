/**
 * Web platform Supabase client adapter
 *
 * SSR-compatible Supabase client using @supabase/ssr.
 * Framework-agnostic implementation that accepts a cookie handler.
 *
 * @packageDocumentation
 */

import { createServerClient, createBrowserClient } from '@supabase/ssr';
import type { SupabaseClient } from '@supabase/supabase-js';
import type { CookieHandler, CookieOptions } from '../../interfaces/session-storage.interface';
import type { Database } from '../../types/database';

/**
 * Configuration for web Supabase client
 */
export interface WebSupabaseClientConfig {
  /**
   * Supabase URL
   */
  url: string;

  /**
   * Supabase anon key
   */
  anonKey: string;

  /**
   * Cookie handler (optional, for server-side rendering)
   */
  cookieHandler?: CookieHandler;

  /**
   * Whether this is a server-side client
   */
  isServer?: boolean;
}

/**
 * Create a Supabase client for web platforms
 *
 * Automatically detects whether to create a server or browser client
 * based on the presence of a cookie handler.
 *
 * @param config - Configuration options
 * @returns Supabase client instance
 */
export function createWebSupabaseClient(
  config: WebSupabaseClientConfig
): SupabaseClient<Database> {
  const { url, anonKey, cookieHandler, isServer } = config;

  // If we have a cookie handler or isServer is true, create a server client
  if (cookieHandler || isServer) {
    if (!cookieHandler) {
      throw new Error('Cookie handler is required for server-side Supabase client');
    }

    return createServerClient<Database>(url, anonKey, {
      cookies: {
        get(name: string) {
          return cookieHandler.get(name);
        },
        set(name: string, value: string, options: any) {
          cookieHandler.set(name, value, options as CookieOptions);
        },
        remove(name: string, options: any) {
          cookieHandler.delete(name, options as CookieOptions);
        },
      },
    });
  }

  // Otherwise, create a browser client
  return createBrowserClient<Database>(url, anonKey);
}

/**
 * Create a Supabase server client for middleware
 *
 * Specialized client for use in middleware with getAll/setAll cookie methods.
 * Useful for frameworks like Next.js middleware.
 *
 * @param config - Configuration with enhanced cookie handler
 * @returns Supabase client instance
 */
export function createMiddlewareSupabaseClient(config: {
  url: string;
  anonKey: string;
  cookieHandler: CookieHandler & {
    getAll(): Array<{ name: string; value: string }>;
    setAll(cookies: Array<{ name: string; value: string; options?: any }>): void;
  };
}): SupabaseClient<Database> {
  const { url, anonKey, cookieHandler } = config;

  return createServerClient<Database>(url, anonKey, {
    cookies: {
      getAll() {
        return cookieHandler.getAll();
      },
      setAll(cookiesToSet) {
        for (const { name, value, options } of cookiesToSet) {
          cookieHandler.set(name, value, options as CookieOptions);
        }
      },
    },
  });
}

/**
 * Helper to create cookie handler from Next.js cookies() API
 *
 * Example usage:
 * ```typescript
 * import { cookies } from 'next/headers';
 *
 * const cookieStore = await cookies();
 * const cookieHandler = createNextCookieHandler(cookieStore);
 * const client = createWebSupabaseClient({
 *   url: process.env.PUBLIC_SUPABASE_URL!,
 *   anonKey: process.env.PUBLIC_SUPABASE_ANON_KEY!,
 *   cookieHandler,
 * });
 * ```
 */
export function createNextCookieHandler(cookieStore: {
  get(name: string): { value: string } | undefined;
  set(name: string, value: string, options?: any): void;
  delete(name: string): void;
  getAll(): Array<{ name: string; value: string }>;
}): CookieHandler {
  return {
    get(name: string): string | undefined {
      return cookieStore.get(name)?.value;
    },
    set(name: string, value: string, options?: any): void {
      cookieStore.set(name, value, options);
    },
    delete(name: string, options?: any): void {
      cookieStore.set(name, '', { ...options, maxAge: 0 });
    },
    getAll(): Array<{ name: string; value: string }> {
      return cookieStore.getAll();
    },
  };
}

/**
 * Helper to create cookie handler for Next.js middleware
 *
 * Example usage:
 * ```typescript
 * import { NextRequest, NextResponse } from 'next/server';
 *
 * export async function middleware(request: NextRequest) {
 *   const response = NextResponse.next({ request });
 *   const cookieHandler = createNextMiddlewareCookieHandler(request, response);
 *
 *   const client = createMiddlewareSupabaseClient({
 *     url: process.env.PUBLIC_SUPABASE_URL!,
 *     anonKey: process.env.PUBLIC_SUPABASE_ANON_KEY!,
 *     cookieHandler,
 *   });
 *
 *   await client.auth.getUser();
 *
 *   return response;
 * }
 * ```
 */
export function createNextMiddlewareCookieHandler(
  request: {
    cookies: {
      get(name: string): { value: string } | undefined;
      set(name: string, value: string): void;
      getAll(): Array<{ name: string; value: string }>;
    };
  },
  response: {
    cookies: {
      set(name: string, value: string, options?: any): void;
      getAll(): Array<{ name: string; value: string }>;
    };
  }
): CookieHandler & {
  getAll(): Array<{ name: string; value: string }>;
  setAll(cookies: Array<{ name: string; value: string; options?: any }>): void;
} {
  return {
    get(name: string): string | undefined {
      return request.cookies.get(name)?.value;
    },
    set(name: string, value: string, options?: any): void {
      request.cookies.set(name, value);
      response.cookies.set(name, value, options);
    },
    delete(name: string, options?: any): void {
      request.cookies.set(name, '');
      response.cookies.set(name, '', { ...options, maxAge: 0 });
    },
    getAll(): Array<{ name: string; value: string }> {
      return request.cookies.getAll();
    },
    setAll(cookiesToSet): void {
      for (const { name, value, options } of cookiesToSet) {
        request.cookies.set(name, value);
        response.cookies.set(name, value, options);
      }
    },
  };
}
