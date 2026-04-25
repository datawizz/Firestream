/**
 * Session storage adapter interface
 *
 * Platform-agnostic interface for session storage operations.
 * Implementations should handle platform-specific storage mechanisms.
 *
 * @packageDocumentation
 */

/**
 * Session data stored in platform-specific storage
 */
export interface SessionData {
  access_token: string;
  refresh_token: string;
  expires_at?: number;
  expires_in?: number;
  token_type?: string;
  user?: {
    id: string;
    email?: string;
    [key: string]: any;
  };
}

/**
 * Base interface for session storage adapters
 *
 * Platform-specific adapters should implement this interface to provide
 * consistent session storage functionality across platforms.
 */
export interface ISessionStorageAdapter {
  /**
   * Store session data
   * @param session - Session data to store
   */
  setSession(session: SessionData): Promise<void>;

  /**
   * Retrieve stored session data
   * @returns Session data or null if not found
   */
  getSession(): Promise<SessionData | null>;

  /**
   * Clear stored session data
   */
  clearSession(): Promise<void>;

  /**
   * Check if a valid session exists
   * @returns true if a valid session exists
   */
  hasValidSession(): Promise<boolean>;

  /**
   * Update partial session data
   * @param partial - Partial session data to update
   */
  updateSession(partial: Partial<SessionData>): Promise<void>;
}

/**
 * Cookie handler interface for framework-agnostic cookie operations
 */
export interface CookieHandler {
  /**
   * Get a cookie value by name
   * @param name - Cookie name
   * @returns Cookie value or undefined
   */
  get(name: string): string | undefined;

  /**
   * Set a cookie
   * @param name - Cookie name
   * @param value - Cookie value
   * @param options - Cookie options
   */
  set(name: string, value: string, options?: CookieOptions): void;

  /**
   * Delete a cookie
   * @param name - Cookie name
   * @param options - Cookie options
   */
  delete(name: string, options?: CookieOptions): void;

  /**
   * Get all cookies
   * @returns Array of cookie objects
   */
  getAll?(): Array<{ name: string; value: string }>;
}

/**
 * Cookie options for setting cookies
 */
export interface CookieOptions {
  maxAge?: number;
  expires?: Date;
  path?: string;
  domain?: string;
  secure?: boolean;
  httpOnly?: boolean;
  sameSite?: 'strict' | 'lax' | 'none';
}
