/**
 * Environment variable utilities
 *
 * Platform-aware utilities for accessing environment variables across
 * Next.js and Vite environments using a unified PUBLIC_ prefix.
 *
 * @packageDocumentation
 */

/**
 * Environment variable prefixes
 *
 * PUBLIC_ is the canonical prefix for client-safe variables.
 * Legacy NEXT_PUBLIC_ and VITE_ prefixes are checked as fallbacks
 * for backward compatibility during migration.
 */
const PLATFORM_PREFIXES = {
  public: 'PUBLIC_',
  next: 'NEXT_PUBLIC_',
  vite: 'VITE_',
} as const;

/**
 * Detect the current platform based on available global variables
 */
function detectPlatform(): 'next' | 'vite' | 'server' {
  // Check if running in browser
  if (typeof window === 'undefined') {
    return 'server';
  }

  // Check for Vite-specific globals
  if (typeof import.meta !== 'undefined' && 'env' in import.meta) {
    return 'vite';
  }

  // Default to Next.js for browser environment
  return 'next';
}

/**
 * Get environment variable value based on platform
 * Checks PUBLIC_ prefix first, then falls back to legacy prefixes
 *
 * @param name - Base name of the environment variable (without prefix)
 * @returns The environment variable value or undefined
 */
function getEnvValue(name: string): string | undefined {
  const platform = detectPlatform();

  // Server-side: Check process.env
  if (platform === 'server') {
    if (typeof process !== 'undefined' && process.env) {
      // Try unprefixed first (for server-only vars)
      if (process.env[name]) {
        return process.env[name];
      }
      // Try canonical PUBLIC_ prefix
      if (process.env[`${PLATFORM_PREFIXES.public}${name}`]) {
        return process.env[`${PLATFORM_PREFIXES.public}${name}`];
      }
      // Legacy fallback: NEXT_PUBLIC_ prefix
      if (process.env[`${PLATFORM_PREFIXES.next}${name}`]) {
        return process.env[`${PLATFORM_PREFIXES.next}${name}`];
      }
    }
    return undefined;
  }

  // Vite platform: Check import.meta.env
  if (platform === 'vite') {
    if (typeof import.meta !== 'undefined' && 'env' in import.meta) {
      const viteEnv = (import.meta as any).env as Record<string, string | undefined>;
      // Try canonical PUBLIC_ prefix
      if (viteEnv[`${PLATFORM_PREFIXES.public}${name}`]) {
        return viteEnv[`${PLATFORM_PREFIXES.public}${name}`];
      }
      // Legacy fallback: VITE_ prefix
      if (viteEnv[`${PLATFORM_PREFIXES.vite}${name}`]) {
        return viteEnv[`${PLATFORM_PREFIXES.vite}${name}`];
      }
      // Try unprefixed
      if (viteEnv[name]) {
        return viteEnv[name];
      }
    }
    return undefined;
  }

  // Next.js platform: Check process.env (available in browser for PUBLIC_ and NEXT_PUBLIC_ vars)
  if (platform === 'next') {
    if (typeof process !== 'undefined' && process.env) {
      // Try canonical PUBLIC_ prefix
      if (process.env[`${PLATFORM_PREFIXES.public}${name}`]) {
        return process.env[`${PLATFORM_PREFIXES.public}${name}`];
      }
      // Legacy fallback: NEXT_PUBLIC_ prefix
      if (process.env[`${PLATFORM_PREFIXES.next}${name}`]) {
        return process.env[`${PLATFORM_PREFIXES.next}${name}`];
      }
      // Try unprefixed
      if (process.env[name]) {
        return process.env[name];
      }
    }
    return undefined;
  }

  return undefined;
}

/**
 * Get a required environment variable
 *
 * @param name - Base name of the environment variable (without prefix)
 * @param customErrorMessage - Optional custom error message
 * @throws Error if the environment variable is not set
 * @returns The environment variable value
 *
 * @example
 * ```typescript
 * // Looks for PUBLIC_SUPABASE_URL (or legacy NEXT_PUBLIC_/VITE_ variants)
 * const supabaseUrl = getRequiredEnvVar('SUPABASE_URL');
 * ```
 */
export function getRequiredEnvVar(
  name: string,
  customErrorMessage?: string
): string {
  const value = getEnvValue(name);

  if (!value) {
    const platform = detectPlatform();
    const prefixedName = platform === 'server' ? name : `${PLATFORM_PREFIXES.public}${name}`;

    throw new Error(
      customErrorMessage ||
        `Missing required environment variable: ${prefixedName}. ` +
          `Please set it in your .env file. ` +
          `Platform: ${platform}`
    );
  }

  return value;
}

/**
 * Get an optional environment variable with a default value
 *
 * @param name - Base name of the environment variable (without prefix)
 * @param defaultValue - Default value to return if the variable is not set
 * @returns The environment variable value or the default value
 *
 * @example
 * ```typescript
 * // Returns environment value or 'http://localhost:3000'
 * const siteUrl = getOptionalEnvVar('SITE_URL', 'http://localhost:3000');
 * ```
 */
export function getOptionalEnvVar(name: string, defaultValue: string): string {
  const value = getEnvValue(name);
  return value ?? defaultValue;
}

/**
 * Check if an environment variable is set
 *
 * @param name - Base name of the environment variable (without prefix)
 * @returns true if the variable is set, false otherwise
 *
 * @example
 * ```typescript
 * if (hasEnvVar('STRIPE_PUBLISHABLE_KEY')) {
 *   // Enable Stripe features
 * }
 * ```
 */
export function hasEnvVar(name: string): boolean {
  return getEnvValue(name) !== undefined;
}

/**
 * Get the current platform
 *
 * @returns The detected platform ('next', 'vite', or 'server')
 *
 * @example
 * ```typescript
 * const platform = getPlatform();
 * if (platform === 'server') {
 *   // Server-side only code
 * }
 * ```
 */
export function getPlatform(): 'next' | 'vite' | 'server' {
  return detectPlatform();
}

/**
 * Validate that all required environment variables are set
 *
 * @param requiredVars - Array of required environment variable names
 * @throws Error with list of missing variables if any are not set
 *
 * @example
 * ```typescript
 * validateEnvVars([
 *   'SUPABASE_URL',
 *   'SUPABASE_ANON_KEY',
 *   'STRIPE_PUBLISHABLE_KEY'
 * ]);
 * ```
 */
export function validateEnvVars(requiredVars: string[]): void {
  const missing: string[] = [];
  const platform = detectPlatform();

  for (const name of requiredVars) {
    if (!getEnvValue(name)) {
      const prefixedName = platform === 'server' ? name : `${PLATFORM_PREFIXES.public}${name}`;
      missing.push(prefixedName);
    }
  }

  if (missing.length > 0) {
    throw new Error(
      `Missing required environment variables:\n` +
        missing.map((v) => `  - ${v}`).join('\n') +
        `\n\nPlease set these in your .env file. Platform: ${platform}`
    );
  }
}
