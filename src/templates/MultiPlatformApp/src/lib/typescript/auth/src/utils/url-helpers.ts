/**
 * URL helper utilities
 *
 * Utility functions for constructing URLs for authentication flows.
 * Works in both web (Next.js) and desktop (Tauri) environments.
 *
 * @packageDocumentation
 */

/**
 * Get the full URL for the site
 *
 * Detects the current environment (Tauri desktop or web browser/SSR)
 * and returns the appropriate base URL with the given path appended.
 *
 * @param path - Optional path to append to the base URL
 * @returns The full URL with the path
 *
 * @example
 * ```typescript
 * getURL('/auth/callback') // Returns 'http://localhost:3000/auth/callback' in dev
 * getURL() // Returns 'https://myapp.com' in production
 * ```
 */
export function getURL(path: string = ''): string {
  // Check for Tauri desktop environment
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    // In Tauri, use PUBLIC_SITE_URL which points to the web app API
    const baseUrl =
      (import.meta as unknown as { env?: { PUBLIC_SITE_URL?: string } }).env
        ?.PUBLIC_SITE_URL || 'http://localhost:3000';
    return `${baseUrl.replace(/\/$/, '')}${path}`;
  }

  // Browser or SSR environment (Next.js)
  let url: string;

  if (typeof window !== 'undefined') {
    // Client-side: use window.location or env vars
    url =
      (window as unknown as { ENV?: { PUBLIC_SITE_URL?: string } }).ENV
        ?.PUBLIC_SITE_URL ||
      process?.env?.PUBLIC_SITE_URL ||
      window.location.origin;
  } else {
    // Server-side: use environment variables
    url =
      process?.env?.PUBLIC_SITE_URL ??
      process?.env?.NEXT_PUBLIC_VERCEL_URL ??
      'http://localhost:3000';
  }

  // Ensure https in production (Vercel URLs don't include protocol)
  if (!url.startsWith('http')) {
    url = `https://${url}`;
  }

  // Remove trailing slash from base URL
  url = url.replace(/\/$/, '');

  return `${url}${path}`;
}

/**
 * Get the redirect URL for OAuth callbacks
 *
 * @param path - The callback path (defaults to '/auth/callback')
 * @returns The full redirect URL for OAuth providers
 */
export function getRedirectURL(path: string = '/auth/callback'): string {
  return getURL(path);
}
