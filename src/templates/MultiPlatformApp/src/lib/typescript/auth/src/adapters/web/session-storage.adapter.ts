/**
 * Web platform session storage adapter
 *
 * Cookie-based session storage implementation for web platforms.
 * Framework-agnostic implementation that accepts a cookie handler.
 *
 * @packageDocumentation
 */

import type {
  ISessionStorageAdapter,
  SessionData,
  CookieHandler,
} from '../../interfaces/session-storage.interface';

/**
 * Configuration for web session storage
 */
export interface WebSessionStorageConfig {
  /**
   * Cookie handler for reading/writing cookies
   */
  cookieHandler: CookieHandler;

  /**
   * Cookie name prefix (default: 'sb')
   */
  cookiePrefix?: string;

  /**
   * Cookie options
   */
  cookieOptions?: {
    maxAge?: number;
    path?: string;
    domain?: string;
    secure?: boolean;
    sameSite?: 'strict' | 'lax' | 'none';
  };
}

/**
 * Web session storage adapter using cookies
 *
 * Provides framework-agnostic session storage using cookies.
 * Accepts a cookie handler to work with any web framework.
 */
export class WebSessionStorageAdapter implements ISessionStorageAdapter {
  private readonly cookieHandler: CookieHandler;
  private readonly cookiePrefix: string;
  private readonly cookieOptions: WebSessionStorageConfig['cookieOptions'];

  constructor(config: WebSessionStorageConfig) {
    this.cookieHandler = config.cookieHandler;
    this.cookiePrefix = config.cookiePrefix || 'sb';
    this.cookieOptions = config.cookieOptions || {
      path: '/',
      secure: true,
      sameSite: 'lax',
      maxAge: 60 * 60 * 24 * 7, // 7 days
    };
  }

  /**
   * Get cookie name for a specific key
   */
  private getCookieName(key: string): string {
    return `${this.cookiePrefix}-${key}`;
  }

  /**
   * Store session data in cookies
   */
  async setSession(session: SessionData): Promise<void> {
    const cookieValue = JSON.stringify(session);

    this.cookieHandler.set(
      this.getCookieName('session'),
      cookieValue,
      this.cookieOptions
    );

    // Store individual tokens as separate cookies for Supabase SSR compatibility
    if (session.access_token) {
      this.cookieHandler.set(
        this.getCookieName('access-token'),
        session.access_token,
        this.cookieOptions
      );
    }

    if (session.refresh_token) {
      this.cookieHandler.set(
        this.getCookieName('refresh-token'),
        session.refresh_token,
        this.cookieOptions
      );
    }
  }

  /**
   * Retrieve session data from cookies
   */
  async getSession(): Promise<SessionData | null> {
    const sessionCookie = this.cookieHandler.get(this.getCookieName('session'));

    if (!sessionCookie) {
      return null;
    }

    try {
      const session = JSON.parse(sessionCookie) as SessionData;
      return session;
    } catch (error) {
      console.error('Failed to parse session cookie:', error);
      return null;
    }
  }

  /**
   * Clear all session cookies
   */
  async clearSession(): Promise<void> {
    const cookieOptions = {
      ...this.cookieOptions,
      maxAge: 0,
    };

    this.cookieHandler.delete(this.getCookieName('session'), cookieOptions);
    this.cookieHandler.delete(this.getCookieName('access-token'), cookieOptions);
    this.cookieHandler.delete(this.getCookieName('refresh-token'), cookieOptions);
  }

  /**
   * Check if a valid session exists
   */
  async hasValidSession(): Promise<boolean> {
    const session = await this.getSession();

    if (!session) {
      return false;
    }

    // Check if session has required tokens
    if (!session.access_token || !session.refresh_token) {
      return false;
    }

    // Check if session is expired
    if (session.expires_at) {
      const now = Math.floor(Date.now() / 1000);
      if (now >= session.expires_at) {
        return false;
      }
    }

    return true;
  }

  /**
   * Update partial session data
   */
  async updateSession(partial: Partial<SessionData>): Promise<void> {
    const currentSession = await this.getSession();

    if (!currentSession) {
      throw new Error('No session to update');
    }

    const updatedSession = {
      ...currentSession,
      ...partial,
    };

    await this.setSession(updatedSession);
  }
}

/**
 * Create a browser cookie handler using document.cookie
 *
 * This is a simple implementation for client-side usage.
 * For server-side rendering, use framework-specific cookie handlers.
 */
export function createBrowserCookieHandler(): CookieHandler {
  return {
    get(name: string): string | undefined {
      const cookies = document.cookie.split(';');
      for (const cookie of cookies) {
        const [cookieName, cookieValue] = cookie.trim().split('=');
        if (cookieName === name) {
          return decodeURIComponent(cookieValue);
        }
      }
      return undefined;
    },

    set(name: string, value: string, options = {}): void {
      let cookieString = `${name}=${encodeURIComponent(value)}`;

      if (options.maxAge !== undefined) {
        cookieString += `; Max-Age=${options.maxAge}`;
      }

      if (options.expires) {
        cookieString += `; Expires=${options.expires.toUTCString()}`;
      }

      if (options.path) {
        cookieString += `; Path=${options.path}`;
      }

      if (options.domain) {
        cookieString += `; Domain=${options.domain}`;
      }

      if (options.secure) {
        cookieString += '; Secure';
      }

      if (options.httpOnly) {
        // Note: httpOnly cannot be set via document.cookie in browser
        console.warn('httpOnly flag cannot be set via document.cookie');
      }

      if (options.sameSite) {
        cookieString += `; SameSite=${options.sameSite}`;
      }

      document.cookie = cookieString;
    },

    delete(name: string, options = {}): void {
      this.set(name, '', {
        ...options,
        maxAge: 0,
      });
    },

    getAll(): Array<{ name: string; value: string }> {
      const cookies = document.cookie.split(';');
      return cookies
        .map((cookie) => {
          const [name, value] = cookie.trim().split('=');
          return {
            name: name.trim(),
            value: decodeURIComponent(value || ''),
          };
        })
        .filter((cookie) => cookie.name && cookie.value);
    },
  };
}
