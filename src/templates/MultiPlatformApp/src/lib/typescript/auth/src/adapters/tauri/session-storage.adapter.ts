/**
 * Tauri platform session storage adapter
 *
 * IPC-based secure session storage for Tauri desktop applications.
 *
 * @packageDocumentation
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  ISessionStorageAdapter,
  SessionData,
} from '../../interfaces/session-storage.interface';

/**
 * Storage key for session data
 */
const SESSION_KEY = 'supabase_session';

/**
 * Tauri session storage adapter using IPC commands
 *
 * Uses Tauri's secure storage commands to persist session data.
 * Falls back to localStorage in development mode for easier testing.
 */
export class TauriSessionStorageAdapter implements ISessionStorageAdapter {
  private useFallback: boolean;

  constructor(options?: { useFallback?: boolean }) {
    // Use fallback in development or when explicitly enabled
    this.useFallback = options?.useFallback ?? this.isDevelopment();
  }

  /**
   * Check if running in development mode
   */
  private isDevelopment(): boolean {
    return (import.meta as any).env?.DEV ?? false;
  }

  /**
   * Store session data securely
   */
  async setSession(session: SessionData): Promise<void> {
    const sessionJson = JSON.stringify(session);

    if (this.useFallback) {
      localStorage.setItem(SESSION_KEY, sessionJson);
      return;
    }

    try {
      await invoke('secure_storage_set', {
        key: SESSION_KEY,
        value: sessionJson,
      });
    } catch (error) {
      console.error('Failed to store session securely, falling back to localStorage:', error);
      this.useFallback = true;
      localStorage.setItem(SESSION_KEY, sessionJson);
    }
  }

  /**
   * Retrieve stored session data
   */
  async getSession(): Promise<SessionData | null> {
    let sessionJson: string | null = null;

    if (this.useFallback) {
      sessionJson = localStorage.getItem(SESSION_KEY);
    } else {
      try {
        sessionJson = await invoke<string | null>('secure_storage_get', {
          key: SESSION_KEY,
        });
      } catch (error) {
        console.error('Failed to retrieve session securely, falling back to localStorage:', error);
        this.useFallback = true;
        sessionJson = localStorage.getItem(SESSION_KEY);
      }
    }

    if (!sessionJson) {
      return null;
    }

    try {
      return JSON.parse(sessionJson) as SessionData;
    } catch (error) {
      console.error('Failed to parse session data:', error);
      return null;
    }
  }

  /**
   * Clear stored session data
   */
  async clearSession(): Promise<void> {
    if (this.useFallback) {
      localStorage.removeItem(SESSION_KEY);
      return;
    }

    try {
      await invoke('secure_storage_remove', {
        key: SESSION_KEY,
      });
    } catch (error) {
      console.error('Failed to clear session securely, falling back to localStorage:', error);
      this.useFallback = true;
      localStorage.removeItem(SESSION_KEY);
    }
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
