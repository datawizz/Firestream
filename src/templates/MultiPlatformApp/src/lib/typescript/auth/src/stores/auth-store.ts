/**
 * Authentication state store
 *
 * Zustand store for managing authentication state with platform-appropriate persistence
 *
 * @packageDocumentation
 */

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import type { AuthUser, Session } from '../types/auth';

/**
 * Platform detection helper
 */
function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI__' in window;
}

/**
 * Create platform-appropriate storage
 */
function createPlatformStorage() {
  if (typeof window === 'undefined') {
    // SSR - return a no-op storage
    return {
      getItem: () => null,
      setItem: () => {},
      removeItem: () => {},
    };
  }

  if (isTauri()) {
    // Tauri - use localStorage as store adapter handles the backend
    return {
      getItem: (name: string) => localStorage.getItem(name),
      setItem: (name: string, value: string) => localStorage.setItem(name, value),
      removeItem: (name: string) => localStorage.removeItem(name),
    };
  }

  // Web - use localStorage
  return {
    getItem: (name: string) => localStorage.getItem(name),
    setItem: (name: string, value: string) => localStorage.setItem(name, value),
    removeItem: (name: string) => localStorage.removeItem(name),
  };
}

/**
 * Authentication store state
 */
interface AuthStore {
  user: AuthUser | null;
  session: Session | null;
  isAuthenticated: boolean;
  isInitialized: boolean;

  // Actions
  setUser: (user: AuthUser | null) => void;
  setSession: (session: Session | null) => void;
  clearUser: () => void;
  setInitialized: (initialized: boolean) => void;
}

/**
 * Create the auth store with persistence
 */
export const useAuthStore = create<AuthStore>()(
  persist(
    (set) => ({
      user: null,
      session: null,
      isAuthenticated: false,
      isInitialized: false,

      setUser: (user) =>
        set({
          user,
          isAuthenticated: user !== null,
        }),

      setSession: (session) =>
        set({
          session,
        }),

      clearUser: () =>
        set({
          user: null,
          session: null,
          isAuthenticated: false,
        }),

      setInitialized: (initialized) =>
        set({
          isInitialized: initialized,
        }),
    }),
    {
      name: 'auth-storage',
      storage: createJSONStorage(() => createPlatformStorage()),
      // Only persist user and session, not derived state
      partialize: (state) => ({
        user: state.user,
        session: state.session,
      }),
    }
  )
);
